import type { Labrinth } from '@modrinth/api-client'
import { ClientIcon, ServerIcon, UserIcon } from '@modrinth/assets'
import type { Component } from 'vue'

import { defineMessage, type MessageDescriptor } from '../../../../composables/i18n'

export const ENVIRONMENTS_COPY: Record<
	Labrinth.Projects.v3.Environment,
	{
		title: MessageDescriptor
		description: MessageDescriptor
	}
> = {
	client_only: {
		title: defineMessage({
			id: 'project.environment.client-only.title',
			defaultMessage: 'Client-side only',
		}),
		description: defineMessage({
			id: 'project.environment.client-only.description',
			defaultMessage:
				'All functionality is done client-side and is compatible with vanilla servers.',
		}),
	},
	server_only: {
		title: defineMessage({
			id: 'project.environment.server-only.title',
			defaultMessage: 'Server-side only, works in singleplayer too',
		}),
		description: defineMessage({
			id: 'project.environment.server-only.description',
			defaultMessage:
				'All functionality is done server-side and is compatible with vanilla clients.',
		}),
	},
	singleplayer_only: {
		title: defineMessage({
			id: 'project.environment.singleplayer-only.title',
			defaultMessage: 'Singleplayer only',
		}),
		description: defineMessage({
			id: 'project.environment.singleplayer-only.description',
			defaultMessage:
				'Only functions in Singleplayer or when not connected to a Multiplayer server.',
		}),
	},
	dedicated_server_only: {
		title: defineMessage({
			id: 'project.environment.dedicated-server-only.title',
			defaultMessage: 'Server-side only',
		}),
		description: defineMessage({
			id: 'project.environment.dedicated-server-only.description',
			defaultMessage:
				'All functionality is done server-side and is compatible with vanilla clients.',
		}),
	},
	client_and_server: {
		title: defineMessage({
			id: 'project.environment.client-and-server.title',
			defaultMessage: 'Client and server, required on both',
		}),
		description: defineMessage({
			id: 'project.environment.client-and-server.description',
			defaultMessage:
				'Has some functionality on both the client and server, even if only partially.',
		}),
	},
	client_only_server_optional: {
		title: defineMessage({
			id: 'project.environment.client-only-server-optional.title',
			defaultMessage: 'Client and server, optional on server',
		}),
		description: defineMessage({
			id: 'project.environment.client-only-server-optional.description',
			defaultMessage:
				'Has some functionality on both the client and server, even if only partially.',
		}),
	},
	server_only_client_optional: {
		title: defineMessage({
			id: 'project.environment.server-only-client-optional.title',
			defaultMessage: 'Client and server, optional on client',
		}),
		description: defineMessage({
			id: 'project.environment.server-only-client-optional.description',
			defaultMessage:
				'Has some functionality on both the client and server, even if only partially.',
		}),
	},
	client_or_server: {
		title: defineMessage({
			id: 'project.environment.client-or-server.title',
			defaultMessage: 'Client and server, optional on both',
		}),
		description: defineMessage({
			id: 'project.environment.client-or-server.description',
			defaultMessage:
				'Has some functionality on both the client and server, even if only partially.',
		}),
	},
	client_or_server_prefers_both: {
		title: defineMessage({
			id: 'project.environment.client-or-server-prefers-both.title',
			defaultMessage: 'Client and server, best when installed on both',
		}),
		description: defineMessage({
			id: 'project.environment.client-or-server-prefers-both.description',
			defaultMessage:
				'Has some functionality on both the client and server, even if only partially.',
		}),
	},
	unknown: {
		title: defineMessage({
			id: 'project.environment.unknown.title',
			defaultMessage: 'Unknown environment',
		}),
		description: defineMessage({
			id: 'project.environment.unknown.description',
			defaultMessage: 'The environment for this version could not be determined.',
		}),
	},
}

export const ENVIRONMENT_TAG_LABELS = {
	client: defineMessage({
		id: 'project.environment.tag.client',
		defaultMessage: 'Client',
	}),
	server: defineMessage({
		id: 'project.environment.tag.server',
		defaultMessage: 'Server',
	}),
	singleplayer: defineMessage({
		id: 'project.environment.tag.singleplayer',
		defaultMessage: 'Singleplayer',
	}),
	clientOptional: defineMessage({
		id: 'project.environment.tag.client-optional',
		defaultMessage: 'Client optional',
	}),
	serverOptional: defineMessage({
		id: 'project.environment.tag.server-optional',
		defaultMessage: 'Server optional',
	}),
} as const

export function getEnvironmentTags(
	environment?: Labrinth.Projects.v3.Environment,
): Array<{ icon: Component; label: MessageDescriptor }> {
	if (!environment || environment === 'unknown') {
		return []
	}

	switch (environment) {
		case 'client_only':
			return [{ icon: ClientIcon, label: ENVIRONMENT_TAG_LABELS.client }]

		case 'server_only':
			return [
				{ icon: ServerIcon, label: ENVIRONMENT_TAG_LABELS.server },
				{ icon: UserIcon, label: ENVIRONMENT_TAG_LABELS.singleplayer },
			]

		case 'singleplayer_only':
			return [{ icon: UserIcon, label: ENVIRONMENT_TAG_LABELS.singleplayer }]

		case 'dedicated_server_only':
			return [{ icon: ServerIcon, label: ENVIRONMENT_TAG_LABELS.server }]

		case 'client_and_server':
			return [
				{ icon: ClientIcon, label: ENVIRONMENT_TAG_LABELS.client },
				{ icon: ServerIcon, label: ENVIRONMENT_TAG_LABELS.server },
			]

		case 'client_only_server_optional':
			return [
				{ icon: ClientIcon, label: ENVIRONMENT_TAG_LABELS.client },
				{ icon: ServerIcon, label: ENVIRONMENT_TAG_LABELS.serverOptional },
			]

		case 'server_only_client_optional':
			return [
				{ icon: ServerIcon, label: ENVIRONMENT_TAG_LABELS.server },
				{ icon: ClientIcon, label: ENVIRONMENT_TAG_LABELS.clientOptional },
			]

		case 'client_or_server':
			return [
				{ icon: ClientIcon, label: ENVIRONMENT_TAG_LABELS.clientOptional },
				{ icon: ServerIcon, label: ENVIRONMENT_TAG_LABELS.serverOptional },
			]

		case 'client_or_server_prefers_both':
			return [
				{ icon: ClientIcon, label: ENVIRONMENT_TAG_LABELS.clientOptional },
				{ icon: ServerIcon, label: ENVIRONMENT_TAG_LABELS.serverOptional },
			]

		default:
			return []
	}
}
