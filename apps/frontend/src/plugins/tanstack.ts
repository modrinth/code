// https://tanstack.com/query/v5/docs/framework/vue/examples/nuxt3
import type { DehydratedState, VueQueryPluginOptions } from '@tanstack/vue-query'
import { dehydrate, hydrate, QueryClient, VueQueryPlugin } from '@tanstack/vue-query'

import { defineNuxtPlugin, useState } from '#imports'

export default defineNuxtPlugin((nuxt) => {
	const vueQueryState = useState<DehydratedState | null>('vue-query')

	const queryClient = new QueryClient({
		defaultOptions: { queries: { staleTime: 10000 } },
	})
	const options: VueQueryPluginOptions = { queryClient }

	nuxt.vueApp.use(VueQueryPlugin, options)

	// Expose queryClient for middleware and composables
	nuxt.provide('queryClient', queryClient)

	if (import.meta.server) {
		nuxt.hooks.hook('app:rendered', () => {
			const dehydrated = dehydrate(queryClient)
			console.log(
				'[tanstack SSR] dehydrating:',
				dehydrated.queries.length,
				'queries',
				dehydrated.queries.map((q) => q.queryHash),
			)
			vueQueryState.value = dehydrated
		})
	}

	if (import.meta.client) {
		console.log(
			'[tanstack CLIENT] hydrating with state:',
			vueQueryState.value
				? `${vueQueryState.value.queries?.length ?? 0} queries`
				: 'NULL',
		)
		hydrate(queryClient, vueQueryState.value)
		console.log(
			'[tanstack CLIENT] cache after hydrate:',
			queryClient.getQueryCache().getAll().length,
			'queries',
			queryClient.getQueryCache().getAll().map((q) => q.queryHash),
		)
	}
})
