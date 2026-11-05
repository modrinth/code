import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { injectModrinthClient } from '#ui/providers'

export function useGlobals() {
	const client = injectModrinthClient()

	const query = useQuery({
		queryKey: ['auth-globals'] as const,
		queryFn: () => client.labrinth.globals_internal.get(),
		staleTime: 1000 * 60 * 5,
	})

	const data = computed(() => ({
		...query.data.value,
		captcha_enabled: query.data.value?.captcha_enabled ?? true,
	}))

	return { ...query, data }
}
