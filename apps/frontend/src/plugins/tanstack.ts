// https://tanstack.com/query/v5/docs/framework/vue/examples/nuxt3
import type { DehydratedState, VueQueryPluginOptions } from '@tanstack/vue-query'
import { dehydrate, hydrate, QueryClient, VueQueryPlugin } from '@tanstack/vue-query'

import { defineNuxtPlugin, useState } from '#imports'

export default defineNuxtPlugin((nuxt) => {
	const vueQueryState = useState<DehydratedState | null>('vue-query')

	function createRetryPolicy(maxAttempts: number) {
		return (failureCount: number, error: unknown) => {
			const status = (error as any)?.statusCode ?? (error as any)?.status
			if (status === 404) return false
			return failureCount < maxAttempts
		}
	}

	const queryClient = new QueryClient({
		defaultOptions: { queries: { staleTime: 10000, retry: createRetryPolicy(3) } },
	})
	const options: VueQueryPluginOptions = { queryClient }

	nuxt.vueApp.use(VueQueryPlugin, options)

	// Expose queryClient for middleware and composables
	nuxt.provide('queryClient', queryClient)

	if (import.meta.server) {
		nuxt.hooks.hook('app:rendered', () => {
			vueQueryState.value = dehydrate(queryClient)
		})
	}

	if (import.meta.client) {
		hydrate(queryClient, vueQueryState.value)
	}
})
