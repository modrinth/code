export default defineNitroPlugin((nitroApp) => {
	nitroApp.hooks.hook('request', (event) => {
		// @ts-ignore
		const url = globalThis.CF_PAGES_URL
		if (url) {
			useRuntimeConfig(event).public.siteUrl = url
		}
	})
})
