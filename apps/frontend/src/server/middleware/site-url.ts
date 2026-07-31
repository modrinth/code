export default defineEventHandler(async (event) => {
	try {
		const mod = 'cloudflare:workers'
		const { env } = await import(/* @vite-ignore */ mod)
		const cfEnv = env as any
		const config = useRuntimeConfig(event)

		if (cfEnv.CF_PAGES_URL) config.public.siteUrl = cfEnv.CF_PAGES_URL
		if (cfEnv.BROWSER_BASE_URL) config.public.apiBaseUrl = cfEnv.BROWSER_BASE_URL
		if (cfEnv.BASE_URL) config.apiBaseUrl = cfEnv.BASE_URL
		if (cfEnv.PYRO_BASE_URL) {
			config.public.pyroBaseUrl = cfEnv.PYRO_BASE_URL
			config.pyroBaseUrl = cfEnv.PYRO_BASE_URL
		}
		if (cfEnv.STRIPE_PUBLISHABLE_KEY)
			config.public.stripePublishableKey = cfEnv.STRIPE_PUBLISHABLE_KEY
	} catch {
		/* empty */
	}
})
