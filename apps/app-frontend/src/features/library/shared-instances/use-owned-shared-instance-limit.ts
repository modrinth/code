import { injectAuth, injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { sharedInstanceLimitsQueryOptions } from './queries'

export function useOwnedSharedInstanceLimit() {
	const auth = injectAuth()
	const client = injectModrinthClient()
	const userId = computed(() => auth.user.value?.id ?? null)

	const limitsQuery = useQuery(
		computed(() => ({
			...sharedInstanceLimitsQueryOptions(client, userId.value ?? ''),
			enabled: !!userId.value,
		})),
	)

	const max = computed(() => limitsQuery.data.value?.max ?? null)
	const limitReached = computed(() => {
		const limits = limitsQuery.data.value
		return !!limits && limits.current >= limits.max
	})

	return { max, limitReached, refetch: limitsQuery.refetch }
}
