import { computed, onScopeDispose, reactive, type Ref, watch } from 'vue'

import {
	getBlockedProjectContentLink,
	getBlockedProjectExternalLink,
} from '../project-links/index.ts'

interface MessageDescriptor {
	id: string
	defaultMessage?: string
	description?: string
}

function defineMessage<T extends MessageDescriptor>(descriptor: T): T {
	return descriptor
}

function defineMessages<T extends Record<string, MessageDescriptor>>(descriptors: T): T {
	return descriptors
}

export interface LinkCheckContext {
	url: string | undefined
	field: string
	generalContent?: boolean

	[key: string]: unknown
}

export interface LinkCheckResult {
	severity: 'valid' | 'warn' | 'error'
	message?: MessageDescriptor
	values?: Record<string, unknown>
}

const valid: LinkCheckResult = { severity: 'valid' }

function warn(message: MessageDescriptor, values?: Record<string, unknown>): LinkCheckResult {
	return { severity: 'warn', message, values }
}

function error(message: MessageDescriptor, values?: Record<string, unknown>): LinkCheckResult {
	return { severity: 'error', message, values }
}

type FieldMatcher = string | string[] | ((field: string, context: LinkCheckContext) => boolean)

function matchesField(matcher: FieldMatcher, context: LinkCheckContext): boolean {
	if (typeof matcher === 'function') return matcher(context.field, context)
	if (Array.isArray(matcher)) return matcher.includes(context.field)
	return matcher === context.field
}

type LinkCheckVerify = (
	match: RegExpMatchArray,
	context: LinkCheckContext,
) => Promise<LinkCheckResult>
type LinkCheckMatcher = RegExp | ((remaining: string) => number | null | Promise<number | null>)

function isAsyncMatcher(when: LinkCheckMatcher): boolean {
	return when instanceof RegExp ? false : when.constructor.name === 'AsyncFunction'
}

interface LinkCheckNode {
	when: LinkCheckMatcher
	label?: string
	unrecognizedSeverity?: 'error' | 'warn'
	unrecognizedMessage?: MessageDescriptor
	forMatchers?: FieldMatcher[]
	verifyMatch?: LinkCheckVerify
	childNodes?: LinkCheckNode[]
	isTransparent?: boolean
	isFallback?: boolean
}

interface LinkCheckBuilder {
	when: LinkCheckMatcher
	label?: string

	for(fields: FieldMatcher): LinkCheckBuilder

	verify(fn: LinkCheckVerify): LinkCheckBuilder

	children(...shapes: LinkCheckChildShape[]): LinkCheckBuilder

	severity(value: 'error' | 'warn'): LinkCheckBuilder

	message(descriptor: MessageDescriptor): LinkCheckBuilder

	transparent(): LinkCheckBuilder

	fallback(): LinkCheckBuilder

	warn(message: MessageDescriptor, values?: Record<string, unknown>): LinkCheckBuilder

	error(message: MessageDescriptor, values?: Record<string, unknown>): LinkCheckBuilder
}

type LinkCheckChildShape =
	| LinkCheckNode
	| LinkCheckBuilder
	| RegExp
	| string
	| ((remaining: string) => number | null | Promise<number | null>)

function anchored(source: string): RegExp {
	return new RegExp(`^${source}`, 'i')
}

function buildNode(when: LinkCheckMatcher, label?: string): LinkCheckBuilder {
	const childNodes: LinkCheckNode[] = []
	const forMatchers: FieldMatcher[] = []
	const node: Record<string, unknown> = { when, label, childNodes, forMatchers }

	node.for = (fields: FieldMatcher) => {
		forMatchers.push(fields)
		return node
	}
	node.verify = (fn: LinkCheckVerify) => {
		node.verifyMatch = fn
		return node
	}
	node.severity = (value: 'error' | 'warn') => {
		node.unrecognizedSeverity = value
		return node
	}
	node.message = (descriptor: MessageDescriptor) => {
		node.unrecognizedMessage = descriptor
		return node
	}
	node.transparent = () => {
		node.isTransparent = true
		return node
	}
	node.fallback = () => {
		node.isFallback = true
		return node
	}
	node.warn = (message: MessageDescriptor, values?: Record<string, unknown>) => {
		node.verifyMatch = async () => warn(message, values)
		return node
	}
	node.error = (message: MessageDescriptor, values?: Record<string, unknown>) => {
		node.verifyMatch = async () => error(message, values)
		return node
	}
	node.children = (...shapes: LinkCheckChildShape[]) => {
		const parentLabel = node.label as string | undefined
		const parentForMatchers = node.forMatchers as FieldMatcher[] | undefined
		for (const shape of shapes) {
			const child = normalizeChild(shape)
			const label = child.label ? [parentLabel, child.label].filter(Boolean).join(' ') : parentLabel
			const inheritedFor = [...(parentForMatchers ?? []), ...(child.forMatchers ?? [])]
			childNodes.push({ ...child, label, forMatchers: inheritedFor })
		}
		return node
	}

	return node as unknown as LinkCheckBuilder
}

function check(
	when: RegExp | string | ((remaining: string) => number | null | Promise<number | null>),
	label?: string,
): LinkCheckBuilder {
	const matcher =
		typeof when === 'function' ? when : typeof when === 'string' ? new RegExp(when) : when
	return buildNode(matcher, label)
}

function fallback(label?: string): LinkCheckBuilder {
	return buildNode(() => 0, label).fallback()
}

function normalizeChild(shape: LinkCheckChildShape): LinkCheckNode {
	if (shape instanceof RegExp || typeof shape === 'function') return { when: shape }
	if (typeof shape === 'string') return { when: new RegExp(shape) }
	return shape as unknown as LinkCheckNode
}

function named(label: string, shapes: LinkCheckChildShape[]): LinkCheckNode[] {
	return shapes.map((shape) => ({ ...normalizeChild(shape), label }))
}

interface MatchResult {
	node: LinkCheckNode
	match: RegExpMatchArray
	expectedChild?: LinkCheckNode
}

async function matchNode(
	node: LinkCheckNode,
	remaining: string,
	context: LinkCheckContext,
	isRoot = false,
): Promise<MatchResult | null> {
	let match: RegExpMatchArray | null
	if (node.when instanceof RegExp) {
		match = node.when.exec(remaining)
	} else {
		const consumed = await node.when(remaining)
		match =
			consumed === null
				? null
				: (Object.assign([remaining.slice(0, consumed)], {
						input: remaining,
						index: 0,
					}) as RegExpMatchArray)
	}

	if (!match) {
		if (!isRoot || !node.unrecognizedMessage) return null
		return {
			node: {
				when: node.when,
				label: node.label,
				unrecognizedMessage: node.unrecognizedMessage,
				unrecognizedSeverity: node.unrecognizedSeverity,
			},
			match: Object.assign([remaining], { input: remaining, index: 0 }) as RegExpMatchArray,
		}
	}

	if (node.childNodes?.length) {
		const rest = remaining.slice(match[0].length)
		const syncChildren = node.childNodes.filter(
			(child) => !isAsyncMatcher(child.when) && !child.isFallback,
		)
		const asyncChildren = node.childNodes.filter(
			(child) =>
				isAsyncMatcher(child.when) &&
				!child.isFallback &&
				!(context.generalContent && hasFieldSpecificDescendant(child)),
		)
		const fallbackChildren = node.childNodes.filter((child) => child.isFallback)
		let expectedChild: LinkCheckNode | undefined
		for (const child of [...syncChildren, ...asyncChildren]) {
			const found = await matchNode(child, rest, context)
			if (found) return found
			if (!expectedChild && child.forMatchers?.some((matcher) => matchesField(matcher, context)))
				expectedChild = child
		}

		const matchingFallback = fallbackChildren.find((child) =>
			child.forMatchers?.some((matcher) => matchesField(matcher, context)),
		)
		if (matchingFallback) {
			return {
				node: matchingFallback,
				match: Object.assign([rest], { input: rest, index: 0 }) as RegExpMatchArray,
			}
		}

		if (node.isTransparent) return null
		return { node, match, expectedChild }
	}

	return { node, match }
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

function validUrlPrefix(remaining: string): number | null {
	let url: URL
	try {
		url = new URL(remaining)
		const hostname = url.hostname

		// https pls
		if (url.protocol !== 'https:') return null

		// ensure there's a domain and TLD
		if (!/[^.]\.[^.]/.test(hostname)) return null

		// reserved TLDs
		if (/(^|\.)(local|localhost|test|example|invalid|onion|arpa|home)$/i.test(hostname)) return null

		// example.com/net/org is reserved (and probably quite likely to be set by AI)
		if (/^example\.(com|net|org)$/i.test(hostname)) return null

		// No IP addresses
		const strippedHost = hostname.replace(/^\[|]$/g, '')
		if (/^\d{1,3}(?:\.\d{1,3}){3}$/.test(strippedHost) || strippedHost.includes(':')) return null

		const protocolPrefix = /^https:\/\//i.exec(remaining)
		return protocolPrefix?.[0].length ?? null
	} catch {
		return null
	}
}

const checks = check(validUrlPrefix).message(invalidUrlMessage).transparent()

const rootNode = checks as unknown as LinkCheckNode

const cache = reactive(new Map<string, 'scheduled' | 'pending' | LinkCheckResult>())

function cacheKey(context: LinkCheckContext): string {
	return JSON.stringify(context)
}

function hasFieldSpecificDescendant(node: LinkCheckNode): boolean {
	return (
		(node.forMatchers?.length ?? 0) > 0 ||
		(node.childNodes?.some((child) => hasFieldSpecificDescendant(child)) ?? false)
	)
}

async function checkLink(context: LinkCheckContext) {
	const url = context.url
	if (!url) return
	const key = cacheKey(context)
	const cached = cache.get(key)
	if (cached === 'pending' || typeof cached === 'object') return

	const normalizedUrl = url.replace(/^(https:\/\/)www\./i, '$1')

	cache.set(key, 'pending')
	const blockedLink = context.generalContent
		? getBlockedProjectContentLink(url)
		: getBlockedProjectExternalLink(url)
	if (blockedLink) {
		cache.set(key, error(coreMessages.neverValid, { label: blockedLink.label }))
		return
	}

	const found = await matchNode(rootNode, normalizedUrl, context, true)
	if (!found) {
		if (context.generalContent && validUrlPrefix(normalizedUrl) !== null) {
			cache.set(key, valid)
			return
		}
		cache.delete(key)
		return
	}
	const { node: matched, match, expectedChild } = found

	const isLeaf = !matched.childNodes?.length
	const applies = isLeaf && matched.forMatchers?.some((matcher) => matchesField(matcher, context))

	if (!applies) {
		if (context.generalContent && hasFieldSpecificDescendant(matched)) {
			cache.set(key, valid)
			return
		}

		const build = matched.unrecognizedSeverity === 'warn' ? warn : error

		if (matched.unrecognizedMessage && isLeaf) {
			const message =
				context.field === 'description' &&
				matched.unrecognizedMessage.id === invalidUrlMessage.id
					? invalidDescriptionUrlMessage
					: matched.unrecognizedMessage
			cache.set(key, build(message, { label: matched.label }))
			return
		}

		if (expectedChild) {
			if (matched.unrecognizedMessage) {
				cache.set(key, build(matched.unrecognizedMessage, { label: matched.label }))
				return
			}

			cache.set(key, build(coreMessages.expectedType, { label: expectedChild.label }))
			return
		}

		const validElsewhere = matched.forMatchers && matched.forMatchers.length > 0
		const message = validElsewhere ? coreMessages.wrongField : coreMessages.neverValid
		cache.set(key, build(message, { label: matched.label }))
		return
	}

	if (!matched.verifyMatch) {
		cache.set(key, valid)
		return
	}

	cache.set(key, 'pending')
	try {
		cache.set(key, await matched.verifyMatch(match, context))
	} catch {
		cache.delete(key)
	}
}

function getLinkCheckState(context: LinkCheckContext): LinkCheckResult | undefined {
	if (!context.url) return undefined
	const state = cache.get(cacheKey(context))
	return typeof state === 'object' ? state : undefined
}

function isLinkCheckPending(context: LinkCheckContext): boolean {
	if (!context.url) return false
	const state = cache.get(cacheKey(context))
	return state === 'scheduled' || state === 'pending'
}

function useLinkCheck(context: Ref<LinkCheckContext>) {
	let timeout: ReturnType<typeof setTimeout>
	let scheduledKey: string | undefined
	watch(
		context,
		(value) => {
			clearTimeout(timeout)
			if (scheduledKey && cache.get(scheduledKey) === 'scheduled') cache.delete(scheduledKey)
			scheduledKey = undefined

			if (!value.url) return
			const key = cacheKey(value)
			if (!cache.has(key)) {
				cache.set(key, 'scheduled')
				scheduledKey = key
			}

			timeout = setTimeout(() => checkLink(value), 500)
		},
		{ deep: true, immediate: true },
	)

	onScopeDispose(() => {
		clearTimeout(timeout)
		if (scheduledKey && cache.get(scheduledKey) === 'scheduled') cache.delete(scheduledKey)
	})

	return computed(() => getLinkCheckState(context.value) ?? null)
}

async function discordInviteVerify(match: RegExpMatchArray): Promise<LinkCheckResult> {
	const res = await fetch(`https://discord.com/api/v10/invites/${match[1]}?with_expiration=true`)

	if (!res.ok)
		return error(
			defineMessage({
				id: 'nags.link.discord.invite.invalid',
				defaultMessage: 'This Discord invite is invalid or has expired.',
			}),
		)

	const invite = await res.json()

	if (!invite.guild)
		return error(
			defineMessage({
				id: 'nags.link.discord.invite.not-guild',
				defaultMessage: 'This Discord invite does not lead to a server.',
			}),
		)

	if (invite.expires_at)
		return warn(
			defineMessage({
				id: 'nags.link.discord.invite.expires',
				defaultMessage: 'This Discord invite is set to expire',
			}),
		)

	//TODO Ideally we could also check if the invite has a max uses and if its temporary but
	// we can't without auth which we can't really do in frontend

	return valid
}

checks.children(
	...named('Discord', [
		check(/^discord\.gg/i).children(
			check(/^\/([\w-]+)/i)
				.for('discord')
				.verify(discordInviteVerify),
		),
		check(/^(?:discord\.com|discordapp\.com)/i).children(
			check(/^\/invite\/([\w-]+)/i)
				.for('discord')
				.verify(discordInviteVerify),
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
	fetchRepo: (path: string) => Promise<Record<string, boolean> | undefined>,
	path: string,
	evaluate: (facts: Record<string, boolean>) => LinkCheckResult,
): Promise<LinkCheckResult> {
	const facts = await fetchRepo(path)
	if (!facts) return error(gitRepoMessages.notFound)

	return evaluate(facts)
}

async function giteaFetchRepo(
	host: string,
	path: string,
): Promise<Record<string, boolean> | undefined> {
	const res = await fetch(`https://${host}/api/v1/repos/${path}`)
	if (!res.ok) return undefined
	const data = await res.json()
	return {
		empty: data.size === 0,
		archived: data.archived,
		issues: data.has_issues,
		wiki: data.has_wiki,
	}
}

function gitHost(
	name: string,
	domain: string,
	fetchRepo: (path: string) => Promise<Record<string, boolean> | undefined>,
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
				.verify(async (match) =>
					checkRepo(fetchRepo, match[1], (facts) =>
						facts.empty ? error(gitRepoMessages.empty) : valid,
					),
				),

			check(anchored(`/(${path})${sep}/issues`), 'issues')
				.for('issues')
				.verify(async (match) =>
					checkRepo(fetchRepo, match[1], (facts) => {
						if (facts.archived) return error(gitRepoMessages.archived)
						if (facts.issues === false) return error(gitRepoMessages.issuesDisabled)
						return valid
					}),
				),

			check(anchored(`/(${path})${sep}/${wikiPath}`), 'wiki')
				.for('wiki')
				.verify(async (match) =>
					checkRepo(fetchRepo, match[1], (facts) =>
						facts.wiki === false ? error(gitRepoMessages.wikiDisabled) : valid,
					),
				),
		)
}

// Repo Platforms, includes most source/issues/wiki + Github Sponsor
checks.children(
	gitHost('GitHub', 'github\\.com', async (path) => {
		const res = await fetch(`https://api.github.com/repos/${path}`)
		if (!res.ok) return undefined
		const data = await res.json()
		return {
			empty: data.size === 0,
			archived: data.archived,
			issues: data.has_issues,
			wiki: data.has_wiki,
		}
	})
		// Github sponsor is here
		//TODO: we can't actually check if sponsors is setup with auth or cuz of cors im not really sure but regardless it doesn't works from browser
		.children(check(/^\/sponsors\/[^/]+/i, 'sponsors').for('github')),

	gitHost('Codeberg', 'codeberg\\.org', (path) => giteaFetchRepo('codeberg.org', path)),

	gitHost(
		'GitLab',
		'gitlab\\.com',
		async (path) => {
			const res = await fetch(`https://gitlab.com/api/v4/projects/${encodeURIComponent(path)}`)
			if (!res.ok) return undefined
			//TODO unauthed gitlab doesn't give us like any info, so... yeah that sucks I guess
			return {}
		},
		{
			pathPattern: /[^/]+(?:\/[^/]+)+/.source,
			subPageSeparator: '/-',
			wikiPath: 'wikis',
		},
	),

	gitHost('Bitbucket', 'bitbucket\\.org', async (path) => {
		const res = await fetch(`https://api.bitbucket.org/2.0/repositories/${path}`)
		if (!res.ok) return undefined
		const data = await res.json()
		return {
			empty: data.size === 0,
			issues: data.has_issues,
			wiki: data.has_wiki,
		}
	}),

	gitHost('Gitee', 'gitee\\.com', async (path) => {
		const res = await fetch(`https://gitee.com/api/v5/repos/${path}`)
		if (!res.ok) return undefined
		const data = await res.json()
		return {
			issues: data.has_issues,
			wiki: data.has_wiki,
		}
	}),
)

const giteaHostCache = new Map<string, boolean>()

async function probeGiteaHost(hostname: string): Promise<boolean> {
	if (giteaHostCache.has(hostname)) return giteaHostCache.get(hostname)!
	try {
		const res = await fetch(`https://${hostname}/api/v1/version`)
		giteaHostCache.set(hostname, res.ok)
		return res.ok
	} catch {
		giteaHostCache.set(hostname, false)
		return false
	}
}

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
				.verify(async (match) =>
					checkRepo(
						(path) => giteaFetchRepo(match[1], path),
						match[2],
						(facts) => (facts.empty ? error(gitRepoMessages.empty) : valid),
					),
				),

			check(/^([^/]+)\/([^/]+\/[^/]+)\/issues/i, 'issues')
				.for('issues')
				.verify(async (match) =>
					checkRepo(
						(path) => giteaFetchRepo(match[1], path),
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
				.verify(async (match) =>
					checkRepo(
						(path) => giteaFetchRepo(match[1], path),
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
		.verify(async (_match, context) =>
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
			.verify(async (match, ctx) => licenseVerify(match[1], ctx)),
	),
	check(anchored('opensource\\.org'), 'OSI').children(
		check(anchored('/licenses?/([\\w.-]+)'))
			.for('license')
			.verify(async (match, ctx) => licenseVerify(match[1], ctx)),
	),
	check(anchored('choosealicense\\.com'), 'choosealicense.com').children(
		check(anchored('/licenses/([\\w.-]+)'))
			.for('license')
			.verify(async (match, ctx) => licenseVerify(match[1], ctx)),
	),
	check(anchored('(?:www\\.)?gnu\\.org'), 'GNU').children(
		check(anchored('/licenses/[\\w.-]+'))
			.for('license')
			.verify(async (_match, ctx) => licenseVerify(null, ctx)),
	),
	check(anchored('(?:www\\.)?apache\\.org'), 'Apache').children(
		check(anchored('/licenses/[\\w.-]+'))
			.for('license')
			.verify(async (_match, ctx) => licenseVerify(null, ctx)),
	),
	check(anchored('creativecommons\\.org'), 'Creative Commons').children(
		check(anchored('/(?:licenses/[\\w-]+|publicdomain/zero)/[\\d.]+/?'))
			.for('license')
			.verify(async (_match, ctx) => licenseVerify(null, ctx)),
	),
)

// Google Forms for issues and Docs for Wiki
checks.children(
	check(/^docs\.google\.com/i, 'Google').children(
		check(/^\/forms\//i, 'Forms').for('issues'),
		check(/^\/document\//i, 'Documents').for('wiki'),
	),
)

export { checkLink, getLinkCheckState, isLinkCheckPending, useLinkCheck }
