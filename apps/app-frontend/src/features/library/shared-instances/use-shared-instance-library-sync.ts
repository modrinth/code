import { injectAuth } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, type Ref, watch } from 'vue'

import { useHiddenInstances } from '@/features/library/hidden-instances/use-hidden-instances'
import { cacheSharedInstanceLibrary } from '@/platform/app-lib/shared-instances/commands'

import {
	getUninstalledSharedInstanceId,
	SHARED_INSTANCE_ID_PREFIX,
	sharedInstanceLibraryKeys,
	toCachedSummary,
} from './queries'
import type { UninstalledSharedInstance } from './types'

export function useSharedInstanceLibrarySync({
	sharedInstances,
	uninstalledIds,
	settled,
}: {
	sharedInstances: Ref<UninstalledSharedInstance[]>
	uninstalledIds: Ref<string[]>
	settled: Ref<boolean>
}) {
	const auth = injectAuth()
	const queryClient = useQueryClient()
	const userId = computed(() => auth.user.value?.id ?? null)
	const { hiddenIds, hiddenIdsLoaded, setHidden } = useHiddenInstances()

	const settledSummaries = computed(() =>
		settled.value ? sharedInstances.value.map(toCachedSummary) : null,
	)

	watch(
		() => JSON.stringify(settledSummaries.value),
		() => {
			const currentUserId = userId.value
			const summaries = settledSummaries.value
			if (!summaries || !currentUserId) return

			queryClient.setQueryData(
				sharedInstanceLibraryKeys.cached(currentUserId),
				sharedInstances.value,
			)
			void cacheSharedInstanceLibrary(summaries).catch((error) =>
				console.warn('Failed to cache shared instance library:', error),
			)
		},
	)

	watch(
		() => settled.value && hiddenIdsLoaded.value,
		(ready) => {
			if (!ready) return
			const uninstalled = new Set(uninstalledIds.value.map(getUninstalledSharedInstanceId))
			for (const instanceId of hiddenIds.value) {
				if (instanceId.startsWith(SHARED_INSTANCE_ID_PREFIX) && !uninstalled.has(instanceId)) {
					void setHidden(instanceId, false)
				}
			}
		},
	)
}
