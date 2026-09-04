import { defineMessages } from '@modrinth/ui'

import type { NagDefinitions } from './types.ts'

const messages = defineMessages({
	addTitle: { id: 'nags.add-links.title', defaultMessage: 'Add external links' },
	addServerTitle: { id: 'nags.add-links-server.title', defaultMessage: 'Add external links' },
	add: {
		id: 'nags.add-links.description',
		defaultMessage:
			'Add any relevant links to external resources, such as source code, an issue tracker, or a permanent Discord invite.',
	},
	addServer: {
		id: 'nags.add-links-server.description',
		defaultMessage:
			'Add any relevant links to external resources, such as a website, store, or a permanent Discord invite.',
	},
	bannedTitle: { id: 'nags.banned-link-usage.title', defaultMessage: 'Remove prohibited links' },
	banned: {
		id: 'nags.banned-link-usage.description',
		defaultMessage: 'The link “{url}” is not allowed as an external link.',
	},
	gplTitle: { id: 'nags.gpl-license-source-required.title', defaultMessage: 'Provide source code' },
	gpl: {
		id: 'nags.gpl-license-source-required.description',
		defaultMessage:
			'Your {type} uses a license which requires source code to be available. Please provide a source code link, or add a source code additional file to each version, or consider using a different license.',
	},
	identicalTitle: { id: 'nags.identical-links.title', defaultMessage: 'Remove identical links' },
	identical: {
		id: 'nags.identical-links.description',
		defaultMessage:
			'Some of your external links appear to be identical. Each link should be listed only once and with the appropriate link type.',
	},
	discordTitle: { id: 'nags.misused-discord-link.title', defaultMessage: 'Move Discord invite' },
	discord: {
		id: 'nags.misused-discord-link-description',
		defaultMessage:
			'Discord invites can not be used for other link types. Please only put your Discord link in the Discord Invite link field.',
	},
	verifyTitle: { id: 'nags.verify-external-links.title', defaultMessage: 'Review external links' },
	verify: {
		id: 'nags.verify-external-links.description',
		defaultMessage:
			'Some of your external links may lead to domains that are inappropriate for that link type.',
	},
	visitLinks: { id: 'nags.visit-links-settings.title', defaultMessage: 'Visit links settings' },
})

export const linkNags = {
	'add-links': { title: messages.addTitle, description: messages.add, destination: 'links' },
	'add-links-server': {
		title: messages.addServerTitle,
		description: messages.addServer,
		destination: 'links',
	},
	'banned-link-usage': {
		title: messages.bannedTitle,
		description: messages.banned,
		destination: 'links',
		linkTitle: messages.visitLinks,
	},
	'gpl-license-source-required': {
		title: messages.gplTitle,
		description: messages.gpl,
		destination: 'links',
		linkTitle: messages.visitLinks,
	},
	'identical-links': {
		title: messages.identicalTitle,
		description: messages.identical,
		destination: 'links',
	},
	'misused-discord-link': {
		title: messages.discordTitle,
		description: messages.discord,
		destination: 'links',
		linkTitle: messages.visitLinks,
	},
	'verify-external-links': {
		title: messages.verifyTitle,
		description: messages.verify,
		destination: 'links',
		linkTitle: messages.visitLinks,
	},
} satisfies NagDefinitions
