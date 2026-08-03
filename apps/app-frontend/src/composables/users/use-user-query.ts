import { useQuery } from '@tanstack/vue-query'
import { computed, type Ref } from 'vue'

import { get_user } from '@/helpers/cache.js'

export function useUserQuery(userId: Ref<string | null | undefined>) {
	return useQuery({
		queryKey: computed(() => ['user', userId.value]),
		queryFn: async ({ queryKey }) => {
			const id = queryKey[1]
			if (typeof id !== 'string') return null
			return await get_user(id, 'bypass').catch(() => null)
		},
		enabled: () => !!userId.value,
		staleTime: 30_000,
	})
}
