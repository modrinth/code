export default defineNitroPlugin((nitroApp) => {
	nitroApp.hooks.hook('error', async (error, { event }) => {
		const statusCode = (error as { statusCode?: number }).statusCode ?? 500
		if (statusCode < 500) {
			// Nitro's defaultHandler logs every fatal error regardless of status. This hook runs
			// synchronously before the error handler, so unflagging here keeps expected 4xx out of logs.
			;(error as { fatal?: boolean }).fatal = false
			return
		}

		console.error(`[Context Error] at ${event?.path}:`, error)
	})
})
