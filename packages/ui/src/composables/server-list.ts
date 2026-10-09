import type { AbstractModrinthClient } from '@modrinth/api-client'
import dayjs from 'dayjs'

export function serverListQueryOptions(client: AbstractModrinthClient) {
	return {
		queryKey: ['servers'] as const,
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
