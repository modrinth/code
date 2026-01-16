import { defineNuxtPlugin } from '#imports'

export default defineNuxtPlugin((nuxt) => {
	if (import.meta.server) {
		nuxt.hooks.hook('app:rendered', (ctx) => {
			if (ctx.ssrContext?.payload?.data) {
				const check = (obj: any, path = 'payload') => {
					if (!obj || typeof obj !== 'object') return
					if (
						obj.constructor &&
						obj.constructor.name !== 'Object' &&
						obj.constructor.name !== 'Array'
					) {
						console.error(`Non-POJO at ${path}:`, obj.constructor.name)
					}
					for (const [k, v] of Object.entries(obj)) {
						check(v, `${path}.${k}`)
					}
				}
				check(ctx.ssrContext.payload.data)
			}
		})
	}
})
