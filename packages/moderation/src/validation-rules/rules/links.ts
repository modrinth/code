import { defineMessages } from '@modrinth/ui/i18n'
import { formatProjectTypeSentence } from '@modrinth/ui/src/utils/common-messages.ts'

import type { Nag, ProjectValidationContext } from '../../types/nags.ts'
import { licenseRequiresSource, notSourceAsDistributed } from '../../utils.ts'
import {
	getBlockedProjectExternalLink,
	isCommonProjectLink,
	isDiscordLink,
} from '../../validators/links/index.ts'
import { evaluateRules } from '../evaluate-rules.ts'
import { toNags } from '../to-nags.ts'
import type { ValidationRuleSet } from '../types.ts'

const messages = defineMessages({
	addLinks: {
		id: 'nags.add-links.title',
		defaultMessage: 'Add external links',
	},
	addServerLinks: {
		id: 'nags.add-links-server.title',
		defaultMessage: 'Add external links',
	},
	addLinksDescription: {
		id: 'nags.add-links.description',
		defaultMessage:
			'Add any relevant links targeted outside of Modrinth, such as source code, an issue tracker, or a Discord invite.',
	},
	addServerLinksDescription: {
		id: 'nags.add-links-server.description',
		defaultMessage:
			'Add any relevant links targeted outside of Modrinth, such as a website, store, or a Discord invite.',
	},
	identicalLinks: {
		id: 'nags.identical-links.title',
		defaultMessage: 'Clean up identical links',
	},
	identicalLinksDescription: {
		id: 'nags.identical-links.description',
		defaultMessage:
			'Some of your external links appear to be identical. Each link should be entered only once and with the appropriate link type.',
	},
	verifyLinks: {
		id: 'nags.verify-external-links.title',
		defaultMessage: 'Verify external links',
	},
	verifyLinksDescription: {
		id: 'nags.verify-external-links.description',
		defaultMessage:
			'Some of your external links may be using domains that are inappropriate for that type of link.',
	},
	moveDiscordInvite: {
		id: 'nags.misused-discord-link.title',
		defaultMessage: 'Move Discord invite',
	},
	moveDiscordInviteDescription: {
		id: 'nags.misused-discord-link-description',
		defaultMessage:
			'Discord invites can not be used for other link types. Please put your Discord link in the Discord Invite link field only.',
	},
	removeBannedLinks: {
		id: 'nags.banned-link-usage.title',
		defaultMessage: 'Remove prohibited links',
	},
	removeBannedLinksDescription: {
		id: 'nags.banned-link-usage.description',
		defaultMessage: 'Remove the prohibited external link: “{url}"',
	},
	provideSource: {
		id: 'nags.gpl-license-source-required.title',
		defaultMessage: 'Provide source code',
	},
	provideSourceDescription: {
		id: 'nags.gpl-license-source-required.description',
		defaultMessage:
			'Your {type} uses a license which requires source code to be available. Please provide a source code link or sources file for each additional version, or consider using a different license.',
	},
	visitLinks: {
		id: 'nags.visit-links-settings.title',
		defaultMessage: 'Visit links settings',
	},
})

export function findBlockedProjectExternalLink(
	context: Pick<ProjectValidationContext, 'project' | 'projectV3'>,
) {
	const urls = [
		context.project.source_url,
		context.project.issues_url,
		context.project.wiki_url,
		context.project.discord_url,
		context.project.license.url,
		...(context.project.donation_urls ?? []).map(({ url }) => url),
		...Object.values(context.projectV3.link_urls ?? {}).map(({ url }) => url),
	]

	for (const url of urls) {
		if (!url) continue
		const blockedLink = getBlockedProjectExternalLink(url)
		if (blockedLink) return blockedLink
	}

	return null
}

export const projectLinksValidationRules = {
	'add-links': {
		severity: 'suggestion',
		evaluate: (context) => ({
			valid:
				Boolean(context.projectV3.minecraft_server) ||
				Object.keys(context.projectV3.link_urls ?? {}).length > 0,
		}),
		presentation: {
			message: messages.addLinksDescription,
			nag: { title: messages.addLinks, destination: 'links' },
		},
	},
	'add-links-server': {
		severity: 'suggestion',
		evaluate: (context) => ({
			valid:
				!context.projectV3.minecraft_server ||
				Object.keys(context.projectV3.link_urls ?? {}).length > 0,
		}),
		presentation: {
			message: messages.addServerLinksDescription,
			nag: { title: messages.addServerLinks, destination: 'links' },
		},
	},
	'identical-links': {
		severity: 'error',
		evaluate: (context) => {
			const links = Object.values(context.projectV3.link_urls ?? {}).map(({ url }) => url)
			return { valid: new Set(links).size === links.length }
		},
		presentation: {
			message: messages.identicalLinksDescription,
			nag: { title: messages.identicalLinks, destination: 'links' },
		},
	},
	'verify-external-links': {
		severity: 'warning',
		evaluate: (context) => {
			const sourceUrl = context.project.source_url
			const issuesUrl = context.project.issues_url
			const discordUrl = context.project.discord_url
			return {
				valid: !(
					(sourceUrl && !isCommonProjectLink(sourceUrl, 'source')) ||
					(issuesUrl && !isCommonProjectLink(issuesUrl, 'issues')) ||
					(discordUrl && !isCommonProjectLink(discordUrl, 'discord'))
				),
			}
		},
		presentation: {
			message: messages.verifyLinksDescription,
			nag: {
				title: messages.verifyLinks,
				destination: 'links',
				linkTitle: messages.visitLinks,
			},
		},
	},
	'misused-discord-link': {
		severity: 'error',
		evaluate: (context) => ({
			valid: !(
				isDiscordLink(context.project.source_url) ||
				isDiscordLink(context.project.issues_url) ||
				isDiscordLink(context.project.wiki_url) ||
				isDiscordLink(context.projectV3.link_urls?.site?.url) ||
				isDiscordLink(context.projectV3.link_urls?.store?.url)
			),
		}),
		presentation: {
			message: messages.moveDiscordInviteDescription,
			nag: {
				title: messages.moveDiscordInvite,
				destination: 'links',
				linkTitle: messages.visitLinks,
			},
		},
	},
	'banned-link-usage': {
		severity: 'error',
		evaluate: (context) => {
			const blockedLink = findBlockedProjectExternalLink(context)
			return blockedLink ? { valid: false, values: { url: blockedLink.url } } : { valid: true }
		},
		presentation: {
			message: messages.removeBannedLinksDescription,
			nag: {
				title: messages.removeBannedLinks,
				destination: 'links',
				linkTitle: messages.visitLinks,
			},
		},
	},
	'gpl-license-source-required': {
		severity: 'error',
		evaluate: (context) => {
			if (context.projectV3.project_types.includes('datapack')) return { valid: true }

			const hasSourceUrl = Boolean(context.project.source_url)
			const everyVersionHasAdditionalFiles = context.versions.every(
				(version) => version.files.length >= 2,
			)
			const requiresSource =
				licenseRequiresSource(context.projectV3.license.id) &&
				notSourceAsDistributed(context.projectV3.project_types) &&
				!hasSourceUrl &&
				!everyVersionHasAdditionalFiles

			return requiresSource
				? { valid: false, values: { projectType: context.project.project_type } }
				: { valid: true }
		},
		presentation: {
			message: messages.provideSourceDescription,
			nag: {
				title: messages.provideSource,
				destination: 'links',
				linkTitle: messages.visitLinks,
				formatValues: (values, formatMessage) => ({
					type: formatProjectTypeSentence(formatMessage, String(values.projectType)),
				}),
			},
		},
	},
} satisfies ValidationRuleSet<ProjectValidationContext>

export function getLinksNags(context: ProjectValidationContext): Nag[] {
	return toNags(evaluateRules(context, projectLinksValidationRules))
}
