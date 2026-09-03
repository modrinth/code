import { defineMessages } from '@modrinth/ui'

import type { NagDestinationId, NagLink } from '../../types/nags.ts'

const messages = defineMessages({
	description: {
		id: 'nags.edit-description.title',
		defaultMessage: 'Edit description',
	},
	disclosures: {
		id: 'nags.settings.disclosures.title',
		defaultMessage: 'Visit disclosure settings',
	},
	gallery: {
		id: 'nags.gallery.title',
		defaultMessage: 'Visit gallery page',
	},
	general: {
		id: 'nags.settings.title',
		defaultMessage: 'Visit general settings',
	},
	license: {
		id: 'nags.settings.license.title',
		defaultMessage: 'Visit license settings',
	},
	links: {
		id: 'nags.settings.links.title',
		defaultMessage: 'Visit links settings',
	},
	moderation: {
		id: 'nags.moderation.title',
		defaultMessage: 'Visit moderation thread',
	},
	permissions: {
		id: 'nags.settings.permissions.title',
		defaultMessage: 'Visit permissions settings',
	},
	server: {
		id: 'nags.server.title',
		defaultMessage: 'Visit server settings',
	},
	tags: {
		id: 'nags.settings.tags.title',
		defaultMessage: 'Visit tag settings',
	},
	versions: {
		id: 'nags.settings.versions.title',
		defaultMessage: 'Visit versions settings',
	},
})

export const nagDestinations = {
	description: {
		path: 'settings/description',
		title: messages.description,
		shouldShow: (context) => context.currentRoute !== 'type-project-settings-description',
	},
	disclosures: {
		path: 'settings/disclosures',
		title: messages.disclosures,
		shouldShow: (context) => context.currentRoute !== 'type-project-settings-disclosures',
	},
	gallery: {
		path: 'settings/gallery',
		title: messages.gallery,
		shouldShow: (context) => context.currentRoute !== 'type-project-settings-gallery',
	},
	general: {
		path: 'settings',
		title: messages.general,
		shouldShow: (context) =>
			!['type-project-settings', 'type-project-settings-general'].includes(context.currentRoute),
	},
	license: {
		path: 'settings/license',
		title: messages.license,
		shouldShow: (context) => context.currentRoute !== 'type-project-settings-license',
	},
	links: {
		path: 'settings/links',
		title: messages.links,
		shouldShow: (context) => context.currentRoute !== 'type-project-settings-links',
	},
	moderation: {
		path: 'moderation',
		title: messages.moderation,
		shouldShow: (context) => context.currentRoute !== 'type-project-moderation',
	},
	permissions: {
		path: 'settings/permissions',
		title: messages.permissions,
		shouldShow: (context) => context.currentRoute !== 'type-project-settings-permissions',
	},
	server: {
		path: 'settings/server',
		title: messages.server,
		shouldShow: (context) => context.currentRoute !== 'type-project-settings-server',
	},
	tags: {
		path: 'settings/tags',
		title: messages.tags,
		shouldShow: (context) => context.currentRoute !== 'type-project-settings-tags',
	},
	versions: {
		path: 'settings/versions',
		title: messages.versions,
		shouldShow: (context) => context.currentRoute !== 'type-project-settings-versions',
	},
} satisfies Record<NagDestinationId, NagLink>
