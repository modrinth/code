import { injectAuth, injectModrinthClient } from '@modrinth/ui'
import { useQueries, useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

import type { GameInstance } from '@/helpers/types'
import { getCachedSharedInstanceLibrary } from '@/platform/app-lib/shared-instances/commands'

import {
	fromCachedSummary,
	sharedInstanceDetailQueryOptions,
	sharedInstanceIdsQueryOptions,
	sharedInstanceLibraryKeys,
} from './queries'
import type { UninstalledSharedInstance } from './types'

export function useUninstalledSharedInstances(instances: MaybeRefOrGetter<GameInstance[]>) {
	const auth = injectAuth()
	const client = injectModrinthClient()
	const userId = computed(() => auth.user.value?.id ?? null)

	const cachedQuery = useQuery({
		networkMode: 'always' as const,
		queryKey: computed(() => sharedInstanceLibraryKeys.cached(userId.value)),
		queryFn: async () => (await getCachedSharedInstanceLibrary()).map(fromCachedSummary),
		enabled: computed(() => !!userId.value),
		staleTime: Infinity,
	})

	const idsQuery = useQuery(
		computed(() => ({
			...sharedInstanceIdsQueryOptions(client, userId.value ?? ''),
			enabled: !!userId.value,
		})),
	)

	const installedSharedInstanceIds = computed(
		() =>
			new Set(
				toValue(instances).flatMap((instance) =>
					instance.shared_instance?.id ? [instance.shared_instance.id] : [],
				),
			),
	)

	const uninstalledIds = computed(() =>
		[...new Set(idsQuery.data.value ?? [])].filter(
			(sharedInstanceId) => !installedSharedInstanceIds.value.has(sharedInstanceId),
		),
	)

	const detailQueries = useQueries({
		queries: computed(() =>
			uninstalledIds.value.map((sharedInstanceId) => ({
				...sharedInstanceDetailQueryOptions(client, userId.value!, sharedInstanceId),
				select: (instance: UninstalledSharedInstance | null) => ({ sharedInstanceId, instance }),
			})),
		),
	})

	const sharedInstances = computed<UninstalledSharedInstance[]>(() => {
		if (!userId.value) return []

		const cached = cachedQuery.data.value ?? []
		if (!idsQuery.isSuccess.value) {
			return cached.filter(
				(instance) => !installedSharedInstanceIds.value.has(instance.shared_instance_id),
			)
		}

		const loaded = new Map(
			detailQueries.value.flatMap((query) =>
				query.data ? [[query.data.sharedInstanceId, query.data.instance] as const] : [],
			),
		)
		const cachedById = new Map(cached.map((instance) => [instance.shared_instance_id, instance]))
		return uninstalledIds.value.flatMap((sharedInstanceId) => {
			const instance = loaded.has(sharedInstanceId)
				? loaded.get(sharedInstanceId)
				: cachedById.get(sharedInstanceId)
			return instance ? [instance] : []
		})
	})

	const settled = computed(
		() => idsQuery.isSuccess.value && !detailQueries.value.some((query) => query.isPending),
	)

	return { sharedInstances, uninstalledIds, settled }
}
