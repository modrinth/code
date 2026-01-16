import { sentryCloudflareNitroPlugin } from '@sentry/nuxt/module/plugins'

declare const __SENTRY_RELEASE__: string
declare const __SENTRY_ENVIRONMENT__: string

export default defineNitroPlugin(
	sentryCloudflareNitroPlugin({
		dsn: 'https://9cf8f56ab7055ab6b1042fad535f2a44@o485889.ingest.us.sentry.io/4510709722185728',
		tracesSampleRate: 0.0001, // match with wrangler.jsonc
		release: __SENTRY_RELEASE__,
		environment: __SENTRY_ENVIRONMENT__,
	}),
)
