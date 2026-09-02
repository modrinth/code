import { readEnv } from '~/helpers/env'

export default defineEventHandler(async (event) => {
	const config = useRuntimeConfig(event)

	const siteUrl = await readEnv('CF_PAGES_URL')
	const browserBaseUrl = await readEnv('BROWSER_BASE_URL')
	const baseUrl = await readEnv('BASE_URL')
	const pyroBaseUrl = await readEnv('PYRO_BASE_URL')
	const stripePublishableKey = await readEnv('STRIPE_PUBLISHABLE_KEY')

	if (siteUrl) config.public.siteUrl = siteUrl
	if (browserBaseUrl) config.public.apiBaseUrl = browserBaseUrl
	if (baseUrl) config.apiBaseUrl = baseUrl
	if (pyroBaseUrl) {
		config.public.pyroBaseUrl = pyroBaseUrl
		config.pyroBaseUrl = pyroBaseUrl
	}
	if (stripePublishableKey) config.public.stripePublishableKey = stripePublishableKey
})
