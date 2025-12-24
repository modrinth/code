import type { Labrinth } from '@modrinth/api-client'

export const ENVIRONMENTS_COPY: Record<
	Labrinth.Projects.v3.Environment,
	{ title: string; description: string }
> = {
	client_only: {
		title: 'Client-side only',
		description: 'All functionality is done client-side and is compatible with vanilla servers.',
	},
	server_only: {
		title: 'Server-side only',
		description: 'All functionality is done server-side and is compatible with vanilla clients.',
	},
	singleplayer_only: {
		title: 'Singleplayer only',
		description: 'Only functions in Singleplayer or when not connected to a Multiplayer server.',
	},
	dedicated_server_only: {
		title: 'Server-side only',
		description: 'All functionality is done server-side and is compatible with vanilla clients.',
	},
	client_and_server: {
		title: 'Client and server',
		description: 'Has some functionality on both the client and server, even if only partially.',
	},
	client_only_server_optional: {
		title: 'Client and server',
		description: 'Has some functionality on both the client and server, even if only partially.',
	},
	server_only_client_optional: {
		title: 'Client and server',
		description: 'Has some functionality on both the client and server, even if only partially.',
	},
	client_or_server: {
		title: 'Client and server',
		description: 'Has some functionality on both the client and server, even if only partially.',
	},
	client_or_server_prefers_both: {
		title: 'Client and server',
		description: 'Has some functionality on both the client and server, even if only partially.',
	},
	unknown: {
		title: 'Unknown environment',
		description: 'The environment for this version could not be determined.',
	},
}
