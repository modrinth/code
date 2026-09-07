export default defineNitroPlugin((nitroApp) => {
	nitroApp.hooks.hook('error', async (error, { event }) => {
		const statusCode = (error as { statusCode?: number }).statusCode ?? 500
		if (statusCode < 500) return

		console.error(`[Context Error] at ${event?.path}:`, error)
	})
})
