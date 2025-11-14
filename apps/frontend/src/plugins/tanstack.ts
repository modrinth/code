// https://tanstack.com/query/v5/docs/framework/vue/examples/nuxt3
import type { DehydratedState, VueQueryPluginOptions } from '@tanstack/vue-query'
import { dehydrate, hydrate, QueryClient, VueQueryPlugin } from '@tanstack/vue-query'

import { defineNuxtPlugin, useState } from '#imports'

export default defineNuxtPlugin((nuxt) => {
	const vueQueryState = useState<DehydratedState | null>('vue-query')

	const queryClient = new QueryClient({
		defaultOptions: { queries: { staleTime: 5000 } },
	})
	const options: VueQueryPluginOptions = { queryClient }

	nuxt.vueApp.use(VueQueryPlugin, options)

	if (import.meta.server) {
		nuxt.hooks.hook('app:rendered', () => {
			vueQueryState.value = dehydrate(queryClient)
		})
	}

	if (import.meta.client) {
		nuxt.hooks.hook('app:created', () => {
			hydrate(queryClient, vueQueryState.value)
		})
	}
})
