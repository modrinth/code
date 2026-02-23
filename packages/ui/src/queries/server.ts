import type { AbstractModrinthClient } from '@modrinth/api-client'

const STALE_TIME = 1000 * 60 * 5 // 5 minutes

export const serverQueryOptions = {
	allocations: (serverId: string, client: AbstractModrinthClient) => ({
		queryKey: ['servers', 'allocations', serverId] as const,
		queryFn: () => client.archon.servers_v0.getAllocations(serverId),
		staleTime: STALE_TIME,
	}),
	startup: (serverId: string, client: AbstractModrinthClient) => ({
		queryKey: ['servers', 'startup', serverId] as const,
		queryFn: () => client.archon.servers_v0.getStartupConfig(serverId),
		staleTime: STALE_TIME,
	}),
}
