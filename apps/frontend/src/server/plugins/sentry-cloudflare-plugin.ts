import { sentryCloudflareNitroPlugin } from '@sentry/nuxt/module/plugins'

export default defineNitroPlugin(
	sentryCloudflareNitroPlugin({
		dsn: 'https://9cf8f56ab7055ab6b1042fad535f2a44@o485889.ingest.us.sentry.io/4510709722185728',
		tracesSampleRate: 1.0,
	}),
)
