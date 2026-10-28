import type { AbstractModrinthClient } from '@modrinth/api-client'
import { queryOptions } from '@tanstack/vue-query'

import { get_instance_worlds } from '@/helpers/worlds'

export const hostingInstanceKeys = {
	linkedServer: (instanceId: string, sharedInstanceId: string, userId: string | null | undefined) =>
		['instances', instanceId, 'hosting', sharedInstanceId, userId] as const,
	worlds: (instanceId: string) => ['instances', instanceId, 'hosting-worlds'] as const,
}

export function hostingLinkedServerQueryOptions(
	client: AbstractModrinthClient,
	instanceId: string,
	sharedInstanceId: string,
	userId: string | null | undefined,
) {
	return queryOptions({
		queryKey: hostingInstanceKeys.linkedServer(instanceId, sharedInstanceId, userId),
		queryFn: async () => {
			const shared = await client.sharedinstances.instances_v1.get(sharedInstanceId, {
				query_linked_server: true,
			})
			return {
				instanceId,
				sharedId: sharedInstanceId,
				userId,
				linkedServer: shared.linked_server,
			}
		},
		staleTime: 30_000,
		refetchInterval: 30_000,
		retry: false,
	})
}

export function hostingInstanceWorldsQueryOptions(instanceId: string) {
	return queryOptions({
		queryKey: hostingInstanceKeys.worlds(instanceId),
		queryFn: () => get_instance_worlds(instanceId),
	})
}
