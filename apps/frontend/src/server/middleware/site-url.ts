export default defineEventHandler(async (event) => {
	try {
		const mod = 'cloudflare:workers'
		const { env } = await import(/* @vite-ignore */ mod)
		const url = (env as any).CF_PAGES_URL
		if (url) {
			useRuntimeConfig(event).public.siteUrl = url
		}
	} catch { /* empty */ }
})
