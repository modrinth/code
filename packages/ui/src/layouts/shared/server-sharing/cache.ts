import type { Archon } from '@modrinth/api-client'
import type { QueryClient } from '@tanstack/vue-query'

export async function clearServerSharedInstance(
	queryClient: QueryClient,
	serverId: string,
	worldId: string,
	sharedInstanceId: string,
) {
	const detailKey = ['servers', 'v1', 'detail', serverId] as const
	const instanceKey = ['shared-instances', sharedInstanceId] as const
	const diffKey = ['servers', 'share-diff', serverId, worldId] as const
	await Promise.all([
		queryClient.cancelQueries({ queryKey: detailKey }),
		queryClient.cancelQueries({ queryKey: instanceKey }),
		queryClient.cancelQueries({ queryKey: diffKey }),
	])
	queryClient.setQueryData<Archon.Servers.v1.ServerFull>(detailKey, (server) =>
		server
			? {
					...server,
					worlds: server.worlds.map((world) =>
						world.id === worldId && world.content?.shared_instance_id === sharedInstanceId
							? {
									...world,
									content: {
										...world.content,
										shared_instance_id: null,
										shared_instance_needs_update: false,
									},
								}
							: world,
					),
				}
			: server,
	)
	queryClient.removeQueries({ queryKey: instanceKey })
	queryClient.removeQueries({ queryKey: diffKey })
}
