import type { AbstractModrinthClient } from '@modrinth/api-client'
import type { QueryClient } from '@tanstack/vue-query'

import { serverKeys, serverListQueryOptions } from '#ui/composables/server-queries'
import { SERVER_PLAY_MIGRATION_TAG } from '#ui/layouts/wrapped/hosting/manage/providers/server-play-migration'

import type { ServerSharingUpdateTarget } from './ServerSharingUpdateModal.vue'

/**
 * Where the server sharing update should send the user, or `null` when it shouldn't be shown
 * because they own no servers, or one of them is already migrated to server play.
 */
export async function resolveServerSharingUpdateTarget(
	client: AbstractModrinthClient,
	queryClient: QueryClient,
	userId: string,
	currentServerId?: string,
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
	if (owned.length === 0 || !owned.every((server) => tagged.has(server.server_id))) return null

	const active = owned.filter((server) => server.status === 'available')
	const playServerId = owned.some((server) => server.server_id === currentServerId)
		? currentServerId
		: active.length === 1
			? active[0].server_id
			: undefined

	return playServerId
		? { path: `/hosting/manage/${playServerId}/play`, playPage: true }
		: { path: '/hosting/manage', playPage: false }
}
