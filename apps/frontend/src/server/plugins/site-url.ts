export default defineNitroPlugin(() => {
	// @ts-ignore
	const url = globalThis.CF_PAGES_URL
	if (url) {
		useRuntimeConfig().public.siteUrl = url
	}
})
