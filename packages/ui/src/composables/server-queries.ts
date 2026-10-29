import { type AbstractModrinthClient, ModrinthApiError } from '@modrinth/api-client'
import dayjs from 'dayjs'

/** Query keys for Archon server data. Shorter forms are prefixes of longer ones for invalidation. */
export const serverKeys = {
	list: () => ['servers'] as const,
	detail: (serverId: string) => ['servers', 'detail', serverId] as const,
	detailV1: (serverId: string) => ['servers', 'v1', 'detail', serverId] as const,
	content: (serverId: string) => ['content', 'list', 'v1', serverId] as const,
	icon: (serverId: string) => ['server-icon', serverId] as const,
	startup: (serverId: string) => ['servers', 'startup', 'v1', serverId] as const,
	shareDiff: (serverId: string, ...scope: [worldId?: string, userId?: string]) =>
		['servers', 'share-diff', serverId, ...scope] as const,
	shareFiles: (serverId: string, worldId: string, userId: string | undefined) =>
		['servers', 'share-files', serverId, worldId, userId] as const,
}

/** Query keys for shared instance data. Shorter forms are prefixes of longer ones for invalidation. */
export const sharedInstanceKeys = {
	instance: (instanceId: string, ...scope: [userId?: string]) =>
		['shared-instances', instanceId, 'instance', ...scope] as const,
	players: (instanceId: string, ...scope: [userId?: string]) =>
		['shared-instances', instanceId, 'players', ...scope] as const,
	invites: (instanceId: string, ...scope: [userId?: string]) =>
		['shared-instances', instanceId, 'invites', ...scope] as const,
	friends: (userId: string | undefined) => ['shared-instances', 'friends', userId] as const,
}

export function serverListQueryOptions(client: AbstractModrinthClient) {
	return {
		queryKey: serverKeys.list(),
		staleTime: 30_000,
		queryFn: async () => {
			const response = await client.archon.servers_v0.list({ limit: 100 })
			if (response.servers.some((server) => server.is_medal)) {
				const subscriptions = await client.labrinth.billing_internal.getSubscriptions()
				for (const server of response.servers) {
					if (!server.is_medal) continue
					const subscription = subscriptions.find((sub) => sub.metadata?.id === server.server_id)
					if (subscription) {
						server.medal_expires = dayjs(subscription.created).add(5, 'days').toISOString()
					}
				}
			}
			return response
		},
	}
}

export function serverIconQueryOptions(serverId: string, client: AbstractModrinthClient) {
	return {
		queryKey: serverKeys.icon(serverId),
		staleTime: 60_000,
		queryFn: async (): Promise<Blob | null> => {
			try {
				return await client.archon.icons_v1.get(serverId, true)
			} catch (error) {
				if (error instanceof ModrinthApiError && error.statusCode === 404) return null
				throw error
			}
		},
	}
}
