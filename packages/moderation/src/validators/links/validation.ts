import {
	checkDiscordInvite,
	fetchBitbucketRepo,
	fetchGiteaRepo,
	fetchGiteeRepo,
	fetchGitHubRepo,
	fetchGitLabRepo,
	type GitRepoFacts,
	probeGiteaHost,
} from './remote-checks.ts'
import {
	anchored,
	check,
	fallback,
	getBlockedProjectContentLink,
	getBlockedProjectExternalLink,
	hasFieldSpecificDescendant,
	matchesField,
	matchNode,
	matchNodeSyntax,
	named,
	validUrlPrefix,
} from './syntax-checks.ts'
import type {
	LinkCheckBuilder,
	LinkCheckContext,
	LinkCheckNode,
	LinkCheckResult,
	MatchResult,
	MessageDescriptor,
} from './types.ts'

function defineMessage<T extends MessageDescriptor>(descriptor: T): T {
	return descriptor
}

function defineMessages<T extends Record<string, MessageDescriptor>>(descriptors: T): T {
	return descriptors
}

const valid: LinkCheckResult = { severity: 'valid' }

function warn(message: MessageDescriptor, values?: Record<string, unknown>): LinkCheckResult {
	return { severity: 'warn', message, values }
}

function error(message: MessageDescriptor, values?: Record<string, unknown>): LinkCheckResult {
	return { severity: 'error', message, values }
}

const coreMessages = defineMessages({
	wrongField: {
		id: 'nags.link.wrong-field',
		defaultMessage: "{label} links aren't valid for this field.",
	},
	neverValid: {
		id: 'nags.link.never-valid',
		defaultMessage: "{label} links aren't allowed here.",
	},
	expectedType: {
		id: 'nags.link.expected-type',
		defaultMessage: "This isn't a valid {label} link.",
	},
})

//TODO: we should probably just let you not provide https but backend currently requires it
const invalidUrlMessage = defineMessage({
	id: 'nags.link.invalid-url',
	defaultMessage: 'This URL is invalid',
})

const invalidDescriptionUrlMessage = defineMessage({
	id: 'nags.link.description.invalid-url',
	defaultMessage: 'The description has an invalid link',
})

const checks = check(validUrlPrefix).message(invalidUrlMessage).transparent()

const rootNode = checks as unknown as LinkCheckNode

type PreparedLinkValidation =
	| LinkCheckResult
	| (() => LinkCheckResult | Promise<LinkCheckResult>)
	| undefined

function prepareMatchedLinkValidation(
	context: LinkCheckContext,
	found: MatchResult,
	includeRemoteChecks: boolean,
): PreparedLinkValidation {
	const { node: matched, match, expectedChild } = found
	const isLeaf = !matched.childNodes?.length
	const applies = isLeaf && matched.forMatchers?.some((matcher) => matchesField(matcher, context))

	if (!applies) {
		if (context.generalContent && hasFieldSpecificDescendant(matched)) return valid

		const build = matched.unrecognizedSeverity === 'warn' ? warn : error
		if (matched.unrecognizedMessage && isLeaf) {
			const message =
				context.field === 'description' && matched.unrecognizedMessage.id === invalidUrlMessage.id
					? invalidDescriptionUrlMessage
					: matched.unrecognizedMessage
			return build(message, { label: matched.label })
		}

		if (expectedChild) {
			if (matched.unrecognizedMessage) {
				return build(matched.unrecognizedMessage, { label: matched.label })
			}

			return build(coreMessages.expectedType, { label: expectedChild.label })
		}

		const validElsewhere = matched.forMatchers && matched.forMatchers.length > 0
		const message = validElsewhere ? coreMessages.wrongField : coreMessages.neverValid
		return build(message, { label: matched.label })
	}

	if (!matched.verifyMatch || (matched.isRemoteVerification && !includeRemoteChecks)) return valid
	return () => matched.verifyMatch!(match, context)
}

function getBlockedLinkResult(context: LinkCheckContext): LinkCheckResult | undefined {
	const url = context.url
	if (!url) return
	const blockedLink = context.generalContent
		? getBlockedProjectContentLink(url)
		: getBlockedProjectExternalLink(url)
	return blockedLink ? error(coreMessages.neverValid, { label: blockedLink.label }) : undefined
}

export function validateLinkSyntax(context: LinkCheckContext): LinkCheckResult | undefined {
	const url = context.url
	if (!url) return

	const blockedResult = getBlockedLinkResult(context)
	if (blockedResult) return blockedResult

	const normalizedUrl = url.replace(/^(https:\/\/)www\./i, '$1')
	const found = matchNodeSyntax(rootNode, normalizedUrl, context, true)
	if (!found)
		return context.generalContent && validUrlPrefix(normalizedUrl) !== null ? valid : undefined

	const prepared = prepareMatchedLinkValidation(context, found, false)
	if (typeof prepared !== 'function') return prepared

	const result = prepared()
	return result instanceof Promise ? undefined : result
}

export async function validateLink(
	context: LinkCheckContext,
): Promise<LinkCheckResult | undefined> {
	const url = context.url
	if (!url) return

	const blockedResult = getBlockedLinkResult(context)
	if (blockedResult) return blockedResult

	const normalizedUrl = url.replace(/^(https:\/\/)www\./i, '$1')
	const found = await matchNode(rootNode, normalizedUrl, context, true)
	if (!found) {
		return context.generalContent && validUrlPrefix(normalizedUrl) !== null ? valid : undefined
	}

	const prepared = prepareMatchedLinkValidation(context, found, true)
	if (typeof prepared !== 'function') return prepared
	try {
		return await prepared()
	} catch {
		return undefined
	}
}

checks.children(
	...named('Discord', [
		check(/^discord\.gg/i).children(
			check(/^\/([\w-]+)/i)
				.for('discord')
				.verifyRemotely((match) => checkDiscordInvite(match[1])),
		),
		check(/^(?:discord\.com|discordapp\.com)/i).children(
			check(/^\/invite\/([\w-]+)/i)
				.for('discord')
				.verifyRemotely((match) => checkDiscordInvite(match[1])),
			check(/^\/channels\//i).message(
				defineMessage({
					id: 'nags.link.discord.channel',
					defaultMessage: 'This is a link to a Discord channel, not a server invite.',
				}),
			),
			check(/^\/users\//i).message(
				defineMessage({
					id: 'nags.link.discord.user',
					defaultMessage: 'This is a link to a Discord user, not a server invite.',
				}),
			),
		),
	]),
)

const gitRepoMessages = defineMessages({
	notFound: {
		id: 'nags.link.git.not-found',
		defaultMessage: 'This repository could not be found (it may be private or deleted).',
	},
	empty: {
		id: 'nags.link.git.empty',
		defaultMessage: 'This repository appears to be empty.',
	},
	archived: {
		id: 'nags.link.git.archived',
		defaultMessage: 'This repository is archived, which disables issues.',
	},
	issuesDisabled: {
		id: 'nags.link.git.issues-disabled',
		defaultMessage: 'Issues are disabled on this repository.',
	},
	wikiDisabled: {
		id: 'nags.link.git.wiki-disabled',
		defaultMessage: 'The wiki is disabled on this repository.',
	},
})

async function checkRepo(
	fetchRepo: (path: string) => Promise<GitRepoFacts | undefined>,
	path: string,
	evaluate: (facts: GitRepoFacts) => LinkCheckResult,
): Promise<LinkCheckResult> {
	const facts = await fetchRepo(path)
	if (!facts) return error(gitRepoMessages.notFound)

	return evaluate(facts)
}

function gitHost(
	name: string,
	domain: string,
	fetchRepo: (path: string) => Promise<GitRepoFacts | undefined>,
	options: {
		pathPattern?: string
		subPageSeparator?: string
		wikiPath?: string
	} = {},
): LinkCheckBuilder {
	const path = options.pathPattern ?? /[^/]+\/[^/]+/.source
	const sep = options.subPageSeparator ?? ''
	const wikiPath = options.wikiPath ?? 'wiki'

	return check(anchored(domain), name)
		.severity('warn')
		.children(
			check(anchored(`/(${path})/?$`), 'repo')
				.for('source')
				.verifyRemotely((match) =>
					checkRepo(fetchRepo, match[1], (facts) =>
						facts.empty ? error(gitRepoMessages.empty) : valid,
					),
				),

			check(anchored(`/(${path})${sep}/issues`), 'issues')
				.for('issues')
				.verifyRemotely((match) =>
					checkRepo(fetchRepo, match[1], (facts) => {
						if (facts.archived) return error(gitRepoMessages.archived)
						if (facts.issues === false) return error(gitRepoMessages.issuesDisabled)
						return valid
					}),
				),

			check(anchored(`/(${path})${sep}/${wikiPath}`), 'wiki')
				.for('wiki')
				.verifyRemotely((match) =>
					checkRepo(fetchRepo, match[1], (facts) =>
						facts.wiki === false ? error(gitRepoMessages.wikiDisabled) : valid,
					),
				),
		)
}

// Repo Platforms, includes most source/issues/wiki + Github Sponsor
checks.children(
	gitHost('GitHub', 'github\\.com', fetchGitHubRepo)
		// Github sponsor is here
		//TODO: we can't actually check if sponsors is setup with auth or cuz of cors im not really sure but regardless it doesn't works from browser
		.children(check(/^\/sponsors\/[^/]+/i, 'sponsors').for('github')),

	gitHost('Codeberg', 'codeberg\\.org', (path) => fetchGiteaRepo('codeberg.org', path)),

	gitHost('GitLab', 'gitlab\\.com', fetchGitLabRepo, {
		pathPattern: /[^/]+(?:\/[^/]+)+/.source,
		subPageSeparator: '/-',
		wikiPath: 'wikis',
	}),

	gitHost('Bitbucket', 'bitbucket\\.org', fetchBitbucketRepo),

	gitHost('Gitee', 'gitee\\.com', fetchGiteeRepo),
)

checks.children(
	check(async (remaining) => {
		const hostMatch = /^[^/]+/.exec(remaining)
		if (!hostMatch) return null
		return (await probeGiteaHost(hostMatch[0])) ? 0 : null
	}, 'Self-hosted Gitea/Forgejo')
		.severity('warn')
		.children(
			check(/^([^/]+)\/([^/]+\/[^/]+)\/?$/i, 'repo')
				.for('source')
				.verifyRemotely((match) =>
					checkRepo(
						(path) => fetchGiteaRepo(match[1], path),
						match[2],
						(facts) => (facts.empty ? error(gitRepoMessages.empty) : valid),
					),
				),

			check(/^([^/]+)\/([^/]+\/[^/]+)\/issues/i, 'issues')
				.for('issues')
				.verifyRemotely((match) =>
					checkRepo(
						(path) => fetchGiteaRepo(match[1], path),
						match[2],
						(facts) => {
							if (facts.archived) return error(gitRepoMessages.archived)
							if (facts.issues === false) return error(gitRepoMessages.issuesDisabled)
							return valid
						},
					),
				),

			check(/^([^/]+)\/([^/]+\/[^/]+)\/wiki/i, 'wiki')
				.for('wiki')
				.verifyRemotely((match) =>
					checkRepo(
						(path) => fetchGiteaRepo(match[1], path),
						match[2],
						(facts) => (facts.wiki === false ? error(gitRepoMessages.wikiDisabled) : valid),
					),
				),
		),
)

// Donation
checks.children(
	check(/^patreon\.com/i, 'Patreon').children(check(/^\/(?:user\?u=\d+|[\w.-]+)/i).for('patreon')),

	check(/^(?:buymeacoffee\.com|buymeacoff\.ee)/i, 'Buy Me a Coffee').children(
		check(/^\/([\w-]+)/i).for('bmac'),
	),

	check(/^paypal\.[a-z.]{2,}/i, 'PayPal')
		.for('paypal')
		.children(
			check(/^\/paypalme\/[\w.-]+/i),
			check(/^\/donate/i),
			check(/^\/cgi-bin\/webscr\?cmd=_donations/i),
		),
	check(/^paypal\.me/i, 'PayPal').children(check(/^\/([\w.-]+)/i).for('paypal')),

	// Github sponsor is with the rest of github.

	check(/^ko-fi\.com/i, 'Ko-fi').children(check(/^\/([\w-]+)/i).for('ko-fi')),

	(() => {
		const YOUTUBE_CHANNEL = '(?:@[\\w.-]+|channel/[\\w-]+|c/[\\w-]+|user/[\\w-]+)'

		return check(/^(?:youtube\.com|youtu\.be)/i, 'YouTube')
			.message(
				defineMessage({
					id: 'nags.link.youtube.unrecognized',
					defaultMessage: "This doesn't look like a YouTube donation link.",
				}),
			)
			.for('other')
			.children(
				check(anchored(`/${YOUTUBE_CHANNEL}/join`)),
				check(anchored(`/${YOUTUBE_CHANNEL}/store`)),
			)
	})(),
)

//TODO: remove this if/when we move this to the backend as we can know this if its backend
// tho actually we will probably still need it even then if we're fine with non immediate redirects
// we at the very least need to reword it in that case idk man
checks.children(
	fallback('Unrecognized redirect link')
		.for(['discord', 'github', 'patreon', 'ko-fi', 'paypal', 'bmac'])
		.verify((_match, context) =>
			warn(
				defineMessage({
					id: 'nags.link.unverifiable-redirect',
					defaultMessage: "This doesn't look like a {platform} link.",
				}),
				{ platform: context.platformName ?? context.field },
			),
		),
)

const licenseCheckMessages = defineMessages({
	urlMismatch: {
		id: 'nags.link.license.url-mismatch',
		defaultMessage:
			'This link points to the {detected} license, but your project is set to {selected}.',
	},
	urlRedundant: {
		id: 'nags.link.license.url-redundant',
		defaultMessage:
			"You don't need to link to a generic license page for a supported license — consider linking to your repository's own license file instead, or leaving this blank.",
	},
})

function licenseVerify(detected: string | null, context: Record<string, unknown>): LinkCheckResult {
	const expectedLicense = context.expectedLicense as string | undefined
	const isCustom = context.isCustom as boolean | undefined

	if (detected && expectedLicense && !isCustom) {
		return detected.toLowerCase() === expectedLicense.toLowerCase()
			? valid
			: warn(licenseCheckMessages.urlMismatch, { detected, selected: expectedLicense })
	}

	return isCustom ? valid : warn(licenseCheckMessages.urlRedundant)
}

checks.children(
	check(anchored('spdx\\.org'), 'SPDX').children(
		check(anchored('/licenses/([\\w.-]+)\\.html'))
			.for('license')
			.verify((match, ctx) => licenseVerify(match[1], ctx)),
	),
	check(anchored('opensource\\.org'), 'OSI').children(
		check(anchored('/licenses?/([\\w.-]+)'))
			.for('license')
			.verify((match, ctx) => licenseVerify(match[1], ctx)),
	),
	check(anchored('choosealicense\\.com'), 'choosealicense.com').children(
		check(anchored('/licenses/([\\w.-]+)'))
			.for('license')
			.verify((match, ctx) => licenseVerify(match[1], ctx)),
	),
	check(anchored('(?:www\\.)?gnu\\.org'), 'GNU').children(
		check(anchored('/licenses/[\\w.-]+'))
			.for('license')
			.verify((_match, ctx) => licenseVerify(null, ctx)),
	),
	check(anchored('(?:www\\.)?apache\\.org'), 'Apache').children(
		check(anchored('/licenses/[\\w.-]+'))
			.for('license')
			.verify((_match, ctx) => licenseVerify(null, ctx)),
	),
	check(anchored('creativecommons\\.org'), 'Creative Commons').children(
		check(anchored('/(?:licenses/[\\w-]+|publicdomain/zero)/[\\d.]+/?'))
			.for('license')
			.verify((_match, ctx) => licenseVerify(null, ctx)),
	),
)

// Google Forms for issues and Docs for Wiki
checks.children(
	check(/^docs\.google\.com/i, 'Google').children(
		check(/^\/forms\//i, 'Forms').for('issues'),
		check(/^\/document\//i, 'Documents').for('wiki'),
	),
)
