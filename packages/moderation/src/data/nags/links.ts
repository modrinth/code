import { defineMessage, useVIntl } from '@vintl/vintl'

import type { Nag, NagContext } from '../../types/nags'

export const commonLinkDomains = {
	source: ['github.com', 'gitlab.com', 'bitbucket.org', 'codeberg.org', 'git.sr.ht'],
	issues: ['github.com', 'gitlab.com', 'bitbucket.org', 'codeberg.org', 'docs.google.com'],
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
	linkShorteners: ['bit.ly', 'adf.ly', 'tinyurl.com', 'short.io', 'is.gd'],
}

export function isCommonUrl(url: string | null, commonDomains: string[]): boolean {
	if (url === null || url === '') return true
	try {
		const domain = new URL(url).hostname.toLowerCase()
		return commonDomains.some((allowed) => domain.includes(allowed))
	} catch {
		return false
	}
}

export function isCommonUrlOfType(url: string | null, commonDomains: string[]): boolean {
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

export const linksNags: Nag[] = [
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
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-links',
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
			isDiscordUrl(context.project.wiki_url ?? null),
		link: {
			path: 'settings/links',
			title: defineMessage({
				id: 'nags.visit-links-settings.title',
				defaultMessage: 'Visit links settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-links',
		},
	},
	{
		id: 'link-shortener-usage',
		title: defineMessage({
			id: 'nags.link-shortener-usage.title',
			defaultMessage: "Don't use link shorteners",
		}),
		description: defineMessage({
			id: 'nags.link-shortener-usage.description',
			defaultMessage:
				'Use of link shorteners or other methods to obscure where a link may lead in your external links or license link is prohibited, please only use appropriate full length links.',
		}),
		status: 'required',
		shouldShow: (context: NagContext) => {
			if (context.project.donation_urls) {
				for (const donation of context.project.donation_urls) {
					if (isLinkShortener(donation.url ?? null)) {
						return true
					}
				}
			}

			return (
				isLinkShortener(context.project.source_url ?? null) ||
				isLinkShortener(context.project.issues_url ?? null) ||
				isLinkShortener(context.project.wiki_url ?? null) ||
				isLinkShortener(context.project.discord_url ?? null) ||
				Boolean(context.project.license.url && isLinkShortener(context.project.license.url ?? null))
			)
		},
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
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
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
						'Your {type, select, mod {mod} plugin {plugin} other {project}} uses a license which requires source code to be available. Please provide a source code link or sources file for each additional version, or consider using a different license.',
				}),
				{
					type: context.project.project_type,
				},
			)
		},
		status: 'required',
		shouldShow: (context: NagContext) => {
			const gplLicenses = [
				'GPL-2.0',
				'GPL-2.0+',
				'GPL-2.0-only',
				'GPL-2.0-or-later',
				'GPL-3.0',
				'GPL-3.0+',
				'GPL-3.0-only',
				'GPL-3.0-or-later',
				'LGPL-2.1',
				'LGPL-2.1+',
				'LGPL-2.1-only',
				'LGPL-2.1-or-later',
				'LGPL-3.0',
				'LGPL-3.0+',
				'LGPL-3.0-only',
				'LGPL-3.0-or-later',
				'AGPL-3.0',
				'AGPL-3.0+',
				'AGPL-3.0-only',
				'AGPL-3.0-or-later',
				'MPL-2.0',
			]

			const isGplLicense = gplLicenses.includes(context.project.license.id)
			const hasSourceUrl = !!context.project.source_url
			const hasAdditionalFiles = (context: NagContext) => {
				let hasAdditional = true
				context.versions.forEach((version) => {
					if (version.files.length < 2) hasAdditional = false
				})
				return hasAdditional
			}
			const notSourceAsDistributed = (context: NagContext) =>
				context.project.project_type === 'mod' || context.project.project_type === 'plugin'

			return (
				isGplLicense &&
				notSourceAsDistributed(context) &&
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
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-links',
		},
	},
]
