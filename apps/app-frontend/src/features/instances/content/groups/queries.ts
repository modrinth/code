import { queryOptions } from '@tanstack/vue-query'

import { instanceKeys } from '@/features/instances/queries'
import { listContentGroups } from '@/platform/app-lib/content-groups/commands'

export const contentGroupKeys = {
	all: () => [...instanceKeys.all, 'content-groups'] as const,
	list: (instanceId: string) => [...contentGroupKeys.all(), instanceId] as const,
}

export const contentGroupMutationKey = [...contentGroupKeys.all(), 'mutation'] as const

export function contentGroupListQueryOptions(instanceId: string) {
	return queryOptions({
		queryKey: contentGroupKeys.list(instanceId),
		networkMode: 'always',
		queryFn: () => listContentGroups(instanceId),
		staleTime: 30_000,
	})
}
