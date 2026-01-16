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
	clientSide: defineMessage({
		id: 'project.about.compatibility.environments.client-side',
		defaultMessage: 'Client-side',
	}),
	serverSide: defineMessage({
		id: 'project.about.compatibility.environments.server-side',
		defaultMessage: 'Server-side',
	}),
	dedicatedServersOnly: defineMessage({
		id: 'project.about.compatibility.environments.dedicated-servers-only',
		defaultMessage: 'Dedicated servers only',
	}),
	singleplayerOnly: defineMessage({
		id: 'project.about.compatibility.environments.singleplayer-only',
		defaultMessage: 'Singleplayer only',
	}),
	singleplayer: defineMessage({
		id: 'project.about.compatibility.environments.singleplayer',
		defaultMessage: 'Singleplayer',
	}),
	clientAndServer: defineMessage({
		id: 'project.about.compatibility.environments.client-and-server',
		defaultMessage: 'Client and server',
	}),
	unknown: defineMessage({
		id: 'project.environment.tag.unknown',
		defaultMessage: 'Unknown',
	}),
	notApplicable: defineMessage({
		id: 'project.environment.tag.not-applicable',
		defaultMessage: 'N/A',
	}),
} as const

export function getEnvironmentTags(
	environment?: Labrinth.Projects.v3.Environment,
): Array<{ icon: Component | null; label: MessageDescriptor }> {
	switch (environment) {
		case 'client_only':
			return [{ icon: ClientIcon, label: ENVIRONMENT_TAG_LABELS.clientSide }]

		case 'server_only':
			return [
				{ icon: ServerIcon, label: ENVIRONMENT_TAG_LABELS.serverSide },
				{ icon: UserIcon, label: ENVIRONMENT_TAG_LABELS.singleplayer },
			]

		case 'singleplayer_only':
			return [{ icon: UserIcon, label: ENVIRONMENT_TAG_LABELS.singleplayerOnly }]

		case 'dedicated_server_only':
			return [{ icon: ServerIcon, label: ENVIRONMENT_TAG_LABELS.dedicatedServersOnly }]

		case 'client_and_server':
			return [{ icon: ClientIcon, label: ENVIRONMENT_TAG_LABELS.clientAndServer }]

		case 'client_only_server_optional':
			return [
				{ icon: ClientIcon, label: ENVIRONMENT_TAG_LABELS.clientSide },
				{ icon: ClientIcon, label: ENVIRONMENT_TAG_LABELS.clientAndServer },
			]

		case 'server_only_client_optional':
			return [
				{ icon: ServerIcon, label: ENVIRONMENT_TAG_LABELS.serverSide },
				{ icon: ClientIcon, label: ENVIRONMENT_TAG_LABELS.clientAndServer },
			]

		case 'client_or_server':
			return [
				{ icon: ClientIcon, label: ENVIRONMENT_TAG_LABELS.clientSide },
				{ icon: ServerIcon, label: ENVIRONMENT_TAG_LABELS.serverSide },
			]

		case 'client_or_server_prefers_both':
			return [
				{ icon: ClientIcon, label: ENVIRONMENT_TAG_LABELS.clientSide },
				{ icon: ServerIcon, label: ENVIRONMENT_TAG_LABELS.serverSide },
				{ icon: ClientIcon, label: ENVIRONMENT_TAG_LABELS.clientAndServer },
			]

		case 'unknown':
			return [{ label: ENVIRONMENT_TAG_LABELS.unknown, icon: null }]

		default:
			return [{ label: ENVIRONMENT_TAG_LABELS.notApplicable, icon: null }]
	}
}
