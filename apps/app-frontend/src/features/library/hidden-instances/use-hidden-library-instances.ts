import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { instanceListQueryOptions } from '@/features/instances/queries'
import type { LibraryInstance } from '@/features/library/library-instance'
import { useUninstalledSharedInstances } from '@/features/library/shared-instances/use-uninstalled-shared-instances'

import { useHiddenInstances } from './use-hidden-instances'

export function useHiddenLibraryInstances() {
	const instancesQuery = useQuery(instanceListQueryOptions())
	const instances = computed(() => instancesQuery.data.value ?? [])
	const { sharedInstances } = useUninstalledSharedInstances(instances)
	const { hiddenIds } = useHiddenInstances()

	return computed(() => {
		const instancesById = new Map<string, LibraryInstance>(
			[...instances.value, ...sharedInstances.value].map((instance) => [instance.id, instance]),
		)
		return [...hiddenIds.value].flatMap((instanceId) => {
			const instance = instancesById.get(instanceId)
			return instance ? [instance] : []
		})
	})
}
