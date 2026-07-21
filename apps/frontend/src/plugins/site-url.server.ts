export default defineNuxtPlugin(async (nuxtApp) => {
	try {
		const mod = 'cloudflare:workers'
		const { env } = await import(/* @vite-ignore */ mod)
		const url = (env as any).CF_PAGES_URL
		if (url) {
			nuxtApp.config.public.siteUrl = url
		}
	} catch {
	}
})
