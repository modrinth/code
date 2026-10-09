import type { AbstractModrinthClient } from '@modrinth/api-client'
import { type IconCacheContext, serverIconQueryOptions, serverListQueryOptions } from '@modrinth/ui'
import type { QueryClient } from '@tanstack/vue-query'

/**
 * Warms the hosting caches for a signed-in user: their server list, the icons of their
 * available servers, and billing data. Icons are skipped once `isCurrentSession` returns false.
 */
export async function prefetchHostingData(
	queryClient: QueryClient,
	client: AbstractModrinthClient,
	iconCache: IconCacheContext,
	isCurrentSession: () => boolean,
) {
	void queryClient.prefetchQuery({
		queryKey: ['billing', 'subscriptions'],
		queryFn: () => client.labrinth.billing_internal.getSubscriptions(),
		staleTime: 30_000,
	})
	void queryClient.prefetchQuery({
		queryKey: ['billing', 'payments'],
		queryFn: () => client.labrinth.billing_internal.getPayments(),
		staleTime: 30_000,
	})

	const response = await queryClient.fetchQuery(serverListQueryOptions(client)).catch(() => null)
	if (!response || !isCurrentSession()) return

	await Promise.allSettled(
		response.servers
			.filter((server) => server.status === 'available' && !server.is_medal)
			.map(async (server) => {
				const icon = await queryClient.fetchQuery(serverIconQueryOptions(server.server_id, client))
				if (icon) await iconCache.cacheIcon(icon)
			}),
	)
}
