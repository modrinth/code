import type { QueryClient } from '@tanstack/vue-query'
import { useQueryClient } from '@tanstack/vue-query'
import { getCurrentInstance } from 'vue'

export function useAppQueryClient(): QueryClient {
	// In components, use the standard composable
	if (getCurrentInstance()) {
		return useQueryClient()
	}

	// In middleware/server context, use the provided instance
	const nuxtApp = useNuxtApp()
	return nuxtApp.$queryClient as QueryClient
}
