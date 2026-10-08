import * as Sentry from '@sentry/nuxt'

declare const __SENTRY_RELEASE__: string
declare const __SENTRY_ENVIRONMENT__: string

Sentry.init({
	dsn: 'https://3edc21136484bd0bd366334d93d588da@o485889.ingest.us.sentry.io/4512221375561728',
	tracesSampleRate: 0.0001,
	release: __SENTRY_RELEASE__,
	environment: __SENTRY_ENVIRONMENT__,
})
