import type { App } from 'vue'
import type { Router } from 'vue-router'

export function setupErrorReporting(app: App, router: Router): void {
	if (!import.meta.env.PROD) return

	const previousHandler = app.config.errorHandler
	let pending: Promise<typeof import('@sentry/vue')> | undefined
	let queuedErrors = 0

	function removeListeners() {
		window.removeEventListener('pointerdown', activate)
		window.removeEventListener('keydown', activate)
		window.removeEventListener('error', onError)
		window.removeEventListener('unhandledrejection', onRejection)
	}

	function load() {
		pending ??= import('@sentry/vue').then((sentry) => {
			app.config.errorHandler = previousHandler
			sentry.init({
				app,
				dsn: 'https://9508775ee5034536bc70433f5f531dd4@o485889.ingest.us.sentry.io/4504579615227904',
				integrations: [sentry.browserTracingIntegration({ router })],
				tracesSampleRate: 0.1,
			})
			removeListeners()
			return sentry
		})
		return pending
	}

	function capture(error: unknown) {
		if (queuedErrors >= 20) return
		queuedErrors++
		void load()
			.then((sentry) => sentry.captureException(error))
			.catch(() => {})
			.finally(() => { queuedErrors-- })
	}

	function onError(event: ErrorEvent) {
		capture(event.error ?? event.message)
	}

	function onRejection(event: PromiseRejectionEvent) {
		capture(event.reason)
	}

	function activate() {
		void load().catch(() => {})
	}

	app.config.errorHandler = (error, instance, info) => {
		if (previousHandler) previousHandler(error, instance, info)
		else console.error(error)
		capture(error)
	}
	window.addEventListener('error', onError)
	window.addEventListener('unhandledrejection', onRejection)
	window.addEventListener('pointerdown', activate, { once: true, passive: true })
	window.addEventListener('keydown', activate, { once: true })
	app.onUnmount(removeListeners)
}
