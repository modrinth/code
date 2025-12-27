import type { Labrinth } from '@modrinth/api-client'
import { ClientIcon, MonitorSmartphoneIcon, ServerIcon, UserIcon } from '@modrinth/assets'
import { defineMessage, type MessageDescriptor } from '@vintl/vintl'
import type { Component } from 'vue'

export const ENVIRONMENTS_COPY: Record<
	Labrinth.Projects.v3.Environment,
	{ title: MessageDescriptor; description: MessageDescriptor; icon?: Component }
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
		icon: ClientIcon,
	},
	server_only: {
		title: defineMessage({
			id: 'project.environment.server-only.title',
			defaultMessage: 'Server-side only',
		}),
		description: defineMessage({
			id: 'project.environment.server-only.description',
			defaultMessage:
				'All functionality is done server-side and is compatible with vanilla clients.',
		}),
		icon: ServerIcon,
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
		icon: UserIcon,
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
		icon: ServerIcon,
	},
	client_and_server: {
		title: defineMessage({
			id: 'project.environment.client-and-server.title',
			defaultMessage: 'Client and server',
		}),
		description: defineMessage({
			id: 'project.environment.client-and-server.description',
			defaultMessage:
				'Has some functionality on both the client and server, even if only partially.',
		}),
		icon: MonitorSmartphoneIcon,
	},
	client_only_server_optional: {
		title: defineMessage({
			id: 'project.environment.client-only-server-optional.title',
			defaultMessage: 'Client and server',
		}),
		description: defineMessage({
			id: 'project.environment.client-only-server-optional.description',
			defaultMessage:
				'Has some functionality on both the client and server, even if only partially.',
		}),
		icon: MonitorSmartphoneIcon,
	},
	server_only_client_optional: {
		title: defineMessage({
			id: 'project.environment.server-only-client-optional.title',
			defaultMessage: 'Client and server',
		}),
		description: defineMessage({
			id: 'project.environment.server-only-client-optional.description',
			defaultMessage:
				'Has some functionality on both the client and server, even if only partially.',
		}),
		icon: MonitorSmartphoneIcon,
	},
	client_or_server: {
		title: defineMessage({
			id: 'project.environment.client-or-server.title',
			defaultMessage: 'Client and server',
		}),
		description: defineMessage({
			id: 'project.environment.client-or-server.description',
			defaultMessage:
				'Has some functionality on both the client and server, even if only partially.',
		}),
		icon: MonitorSmartphoneIcon,
	},
	client_or_server_prefers_both: {
		title: defineMessage({
			id: 'project.environment.client-or-server-prefers-both.title',
			defaultMessage: 'Client and server',
		}),
		description: defineMessage({
			id: 'project.environment.client-or-server-prefers-both.description',
			defaultMessage:
				'Has some functionality on both the client and server, even if only partially.',
		}),
		icon: MonitorSmartphoneIcon,
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
