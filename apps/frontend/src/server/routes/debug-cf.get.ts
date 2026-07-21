export default defineEventHandler(async () => {
	try {
		const mod = 'cloudflare:workers'
		const { env } = await import(/* @vite-ignore */ mod)
		return {
			CF_PAGES_URL: (env as any).CF_PAGES_URL,
			PREVIEW: (env as any).PREVIEW,
		}
	} catch (e) {
		return { error: String(e) }
	}
})
