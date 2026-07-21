export default defineNuxtPlugin((nuxtApp) => {
	// @ts-ignore
	const url = globalThis.CF_PAGES_URL
	if (url) {
		nuxtApp.config.public.siteUrl = url
	}
})
