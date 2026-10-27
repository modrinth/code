import { getHiddenInstanceIds } from '@/platform/app-lib/hidden-instances/commands'

export const hiddenInstanceKeys = {
	all: ['hidden-instances'] as const,
	ids: (userId: string | null) => [...hiddenInstanceKeys.all, userId] as const,
}

export const hiddenInstanceIdsQueryOptions = (userId: string | null) => ({
	queryKey: hiddenInstanceKeys.ids(userId),
	queryFn: getHiddenInstanceIds,
	staleTime: Infinity,
})
