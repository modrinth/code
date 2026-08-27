import { defineMessage, formatProjectTypeSentence, useVIntl } from '@modrinth/ui'

import type { Nag, NagContext } from '../../types/nags'
import { licenseRequiresSource, notSourceAsDistributed } from '../../utils'
import {
	getBlockedProjectContentLink,
	PROJECT_CONTENT_LINK_SHORTENERS,
} from '../../validators/project-links'

export const commonLinkDomains = {
	source: [
		'github.com',
		'gitlab.com',
		'bitbucket.org',
		'codeberg.org',
		'git.sr.ht',
		'tangled.org',
		'git.gay',
	],
	issues: [
		'github.com',
		'gitlab.com',
		'bitbucket.org',
		'codeberg.org',
		'docs.google.com',
		'tangled.org',
		'git.gay',
	],
	discord: ['discord.gg', 'discord.com', 'dsc.gg'],
	licenseBlocklist: [
		'youtube.com',
		'youtu.be',
		'modrinth.com',
		'curseforge.com',
		'twitter.com',
		'x.com',
		'discord.gg',
		'discord.com',
		'instagram.com',
		'facebook.com',
		'tiktok.com',
		'reddit.com',
		'twitch.tv',
		'patreon.com',
		'ko-fi.com',
		'paypal.com',
		'buymeacoffee.com',
		'google.com',
		'example.com',
		't.me',
	],
	linkShorteners: PROJECT_CONTENT_LINK_SHORTENERS,
}

export function isCommonUrl(url: string | null, commonDomains: readonly string[]): boolean {
	if (url === null || url === '') return true
	try {
		const domain = new URL(url).hostname.toLowerCase()
		return commonDomains.some((allowed) => domain.includes(allowed))
	} catch {
		return false
	}
}

export function isCommonUrlOfType(url: string | null, commonDomains: readonly string[]): boolean {
	if (url === null || url === '') return false
	return isCommonUrl(url, commonDomains)
}

export function isDiscordUrl(url: string | null): boolean {
	return isCommonUrlOfType(url, commonLinkDomains.discord)
}

export function isLinkShortener(url: string | null): boolean {
	return isCommonUrlOfType(url, commonLinkDomains.linkShorteners)
}

export function isUncommonLicenseUrl(url: string | null): boolean {
	return isCommonUrlOfType(url, commonLinkDomains.licenseBlocklist)
}

export function findBlockedProjectExternalLink(context: Pick<NagContext, 'project' | 'projectV3'>) {
	const urls = [
		context.project.source_url,
		context.project.issues_url,
		context.project.wiki_url,
		context.project.discord_url,
		context.project.license.url,
		...(context.project.donation_urls ?? []).map(({ url }) => url),
		...Object.values(context.projectV3?.link_urls ?? {}).map(({ url }) => url),
	]

	for (const url of urls) {
		if (!url) continue
		const blockedLink = getBlockedProjectContentLink(url)
		if (blockedLink) return blockedLink
	}

	return null
}

export const linksNags: Nag[] = [
	{
		id: 'add-links',
		title: defineMessage({
			id: 'nags.add-links.title',
			defaultMessage: 'Add external links',
		}),
		description: defineMessage({
			id: 'nags.add-links.description',
			defaultMessage:
				'Add any relevant links targeted outside of Modrinth, such as source code, an issue tracker, or a Discord invite.',
		}),
		status: 'suggestion',
		shouldShow: (context: NagContext) => {
			return (
				!context.projectV3?.minecraft_server &&
				Object.keys(context.projectV3?.link_urls ?? {}).length === 0
			)
		},
		link: {
			path: 'settings/links',
			title: defineMessage({
				id: 'nags.settings.links.title',
				defaultMessage: 'Visit links settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings-links',
		},
	},
	{
		id: 'add-links-server',
		title: defineMessage({
			id: 'nags.add-links-server.title',
			defaultMessage: 'Add external links',
		}),
		description: defineMessage({
			id: 'nags.add-links-server.description',
			defaultMessage:
				'Add any relevant links targeted outside of Modrinth, such as a website, store, or a Discord invite.',
		}),
		status: 'suggestion',
		shouldShow: (context: NagContext) => {
			return (
				!!context.projectV3?.minecraft_server &&
				Object.keys(context.projectV3?.link_urls ?? {}).length === 0
			)
		},
		link: {
			path: 'settings/links',
			title: defineMessage({
				id: 'nags.settings.links.title',
				defaultMessage: 'Visit links settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings-links',
		},
	},
	{
		id: 'identical-links',
		title: defineMessage({
			id: 'nags.identical-links.title',
			defaultMessage: 'Clean up identical links',
		}),
		description: defineMessage({
			id: 'nags.identical-links.description',
			defaultMessage:
				'Some of your external links appear to be identical. Each link should be entered only once and with the appropriate link type.',
		}),
		status: 'required',
		shouldShow: (context: NagContext) =>
			new Set(Object.values(context.projectV3?.link_urls ?? {}).map((link) => link.url)).size !==
			Object.values(context.projectV3?.link_urls ?? {}).map((link) => link.url).length,
		link: {
			path: 'settings/links',
			title: defineMessage({
				id: 'nags.settings.links.title',
				defaultMessage: 'Visit links settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings-links',
		},
	},
	{
		id: 'verify-external-links',
		title: defineMessage({
			id: 'nags.verify-external-links.title',
			defaultMessage: 'Verify external links',
		}),
		description: defineMessage({
			id: 'nags.verify-external-links.description',
			defaultMessage:
				'Some of your external links may be using domains that are inappropriate for that type of link.',
		}),
		status: 'warning',
		shouldShow: (context: NagContext) => {
			return (
				!isCommonUrl(context.project.source_url ?? null, commonLinkDomains.source) ||
				!isCommonUrl(context.project.issues_url ?? null, commonLinkDomains.issues) ||
				!isCommonUrl(context.project.discord_url ?? null, commonLinkDomains.discord)
			)
		},
		link: {
			path: 'settings/links',
			title: defineMessage({
				id: 'nags.visit-links-settings.title',
				defaultMessage: 'Visit links settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings-links',
		},
	},
	{
		id: 'misused-discord-link',
		title: defineMessage({
			id: 'nags.misused-discord-link.title',
			defaultMessage: 'Move Discord invite',
		}),
		description: defineMessage({
			id: 'nags.misused-discord-link-description',
			defaultMessage:
				'Discord invites can not be used for other link types. Please put your Discord link in the Discord Invite link field only.',
		}),
		status: 'required',
		shouldShow: (context: NagContext) =>
			isDiscordUrl(context.project.source_url ?? null) ||
			isDiscordUrl(context.project.issues_url ?? null) ||
			isDiscordUrl(context.project.wiki_url ?? null) ||
			isDiscordUrl(context.projectV3?.link_urls?.site?.url ?? null) ||
			isDiscordUrl(context.projectV3?.link_urls?.store?.url ?? null),
		link: {
			path: 'settings/links',
			title: defineMessage({
				id: 'nags.visit-links-settings.title',
				defaultMessage: 'Visit links settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings-links',
		},
	},
	{
		id: 'banned-link-usage',
		title: defineMessage({
			id: 'nags.banned-link-usage.title',
			defaultMessage: 'Remove prohibited links',
		}),
		description: (context: NagContext) => {
			const blockedLink = findBlockedProjectExternalLink(context)
			if (!blockedLink) return ''

			const { formatMessage } = useVIntl()
			return formatMessage(
				defineMessage({
					id: 'nags.banned-link-usage.description',
					defaultMessage: '“{url}” is not allowed in project links.',
				}),
				blockedLink,
			)
		},
		status: 'required',
		shouldShow: (context: NagContext) => findBlockedProjectExternalLink(context) !== null,
	},
	{
		id: 'invalid-license-url',
		title: defineMessage({
			id: 'nags.invalid-license-url.title',
			defaultMessage: 'Add a valid license link',
		}),
		description: (context: NagContext) => {
			const { formatMessage } = useVIntl()
			const licenseUrl = context.project.license.url

			if (!licenseUrl) {
				return formatMessage(
					defineMessage({
						id: 'nags.invalid-license-url.description.default',
						defaultMessage: 'License URL is invalid.',
					}),
				)
			}

			try {
				const domain = new URL(licenseUrl).hostname.toLowerCase()
				return formatMessage(
					defineMessage({
						id: 'nags.invalid-license-url.description.domain',
						defaultMessage:
							'Your license URL points to {domain}, which is not appropriate for license information. License URLs should link directly to your license file, not social media, gaming platforms, etc.',
					}),
					{ domain },
				)
			} catch {
				return formatMessage(
					defineMessage({
						id: 'nags.invalid-license-url.description.malformed',
						defaultMessage:
							'Your license URL appears to be malformed. Please provide a valid URL to your license text.',
					}),
				)
			}
		},
		status: 'required',
		shouldShow: (context: NagContext) => {
			const licenseUrl = context.project.license.url
			if (!licenseUrl) return false

			const isBlocklisted = isUncommonLicenseUrl(licenseUrl)

			try {
				new URL(licenseUrl)
				return isBlocklisted
			} catch {
				return true
			}
		},
		link: {
			path: 'settings',
			title: defineMessage({
				id: 'nags.edit-license.title',
				defaultMessage: 'Edit license',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings',
		},
	},
	{
		id: 'gpl-license-source-required',
		title: defineMessage({
			id: 'nags.gpl-license-source-required.title',
			defaultMessage: 'Provide source code',
		}),
		description: (context: NagContext) => {
			const { formatMessage } = useVIntl()

			return formatMessage(
				defineMessage({
					id: 'nags.gpl-license-source-required.description',
					defaultMessage:
						'Your {type} uses a license which requires source code to be available. Please provide a source code link or sources file for each additional version, or consider using a different license.',
				}),
				{
					type: formatProjectTypeSentence(formatMessage, context.project.project_type),
				},
			)
		},
		status: 'required',
		shouldShow: (context: NagContext) => {
			if (context.projectV3.project_types.includes('datapack')) return false

			const hasSourceUrl = !!context.project.source_url
			const hasAdditionalFiles = (context: NagContext) => {
				let hasAdditional = true
				context.versions.forEach((version) => {
					if (version.files.length < 2) hasAdditional = false
				})
				return hasAdditional
			}

			return (
				licenseRequiresSource(context.projectV3.license.id) &&
				notSourceAsDistributed(context.projectV3.project_types) &&
				!hasSourceUrl &&
				!hasAdditionalFiles(context)
			)
		},
		link: {
			path: 'settings/links',
			title: defineMessage({
				id: 'nags.visit-links-settings.title',
				defaultMessage: 'Visit links settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings-links',
		},
	},
]
