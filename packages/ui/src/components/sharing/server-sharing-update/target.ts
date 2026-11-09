import type { AbstractModrinthClient } from '@modrinth/api-client'
import type { QueryClient } from '@tanstack/vue-query'

import { serverKeys, serverListQueryOptions } from '#ui/composables/server-queries'
import { SERVER_PLAY_MIGRATION_TAG } from '#ui/layouts/wrapped/hosting/manage/providers/server-play-migration'

import type { ServerSharingUpdateTarget } from './ServerSharingUpdateModal.vue'

const TARGET_CACHE_MS = 3 * 60 * 60 * 1000

/**
 * Where the server sharing update should send the user, or `null` when it shouldn't be shown.
 * Current customers see it only while every server they own is still tagged for migration;
 * users who own no servers see it with a purchase CTA if they previously paid for one.
 * Cached for a few hours, or until the server list is invalidated.
 */
export function resolveServerSharingUpdateTarget(
	client: AbstractModrinthClient,
	queryClient: QueryClient,
	userId: string,
	currentServerId?: string,
): Promise<ServerSharingUpdateTarget | null> {
	return queryClient.fetchQuery({
		queryKey: serverKeys.sharingUpdateTarget(userId, currentServerId),
		queryFn: () => computeTarget(client, queryClient, userId, currentServerId),
		staleTime: TARGET_CACHE_MS,
		gcTime: TARGET_CACHE_MS,
		retry: false,
	})
}

async function computeTarget(
	client: AbstractModrinthClient,
	queryClient: QueryClient,
	userId: string,
	currentServerId: string | undefined,
): Promise<ServerSharingUpdateTarget | null> {
	const [list, fullList] = await Promise.all([
		queryClient.fetchQuery({ ...serverListQueryOptions(client), retry: false }),
		queryClient.fetchQuery({
			queryKey: serverKeys.listV1(),
			queryFn: () => client.archon.servers_v1.list(),
			staleTime: 30_000,
			retry: false,
		}),
	])

	const tagged = new Set(
		fullList
			.filter((server) => server.tags.includes(SERVER_PLAY_MIGRATION_TAG))
			.map((server) => server.id),
	)
	const owned = list.servers.filter((server) => server.owner_id === userId)
	if (owned.length === 0) {
		const subscriptions = await queryClient.fetchQuery({
			queryKey: ['billing', 'subscriptions'],
			queryFn: () => client.labrinth.billing_internal.getSubscriptions(),
			staleTime: 30_000,
			retry: false,
		})
		return subscriptions.some((subscription) => subscription.metadata?.type === 'pyro')
			? { kind: 'purchase' }
			: null
	}
	if (!owned.every((server) => tagged.has(server.server_id))) return null

	const active = owned.filter((server) => server.status === 'available')
	const playServerId = owned.some((server) => server.server_id === currentServerId)
		? currentServerId
		: active.length === 1
			? active[0].server_id
			: undefined

	return playServerId
		? { kind: 'invite', path: `/hosting/manage/${playServerId}/play`, playPage: true }
		: { kind: 'invite', path: '/hosting/manage', playPage: false }
}
