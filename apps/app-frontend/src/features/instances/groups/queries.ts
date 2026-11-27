import { queryOptions } from '@tanstack/vue-query'

import { instanceKeys } from '@/features/instances/queries'
import { listGroups } from '@/platform/app-lib/instance-groups/commands'

export const instanceGroupKeys = {
	list: () => [...instanceKeys.all, 'groups'] as const,
}

export function instanceGroupListQueryOptions() {
	return queryOptions({
		queryKey: instanceGroupKeys.list(),
		networkMode: 'always',
		queryFn: listGroups,
		staleTime: 30_000,
	})
}
