function findNonPOJOs(
	obj: unknown,
	path: string,
	found: Array<{ path: string; type: string }> = [],
): Array<{ path: string; type: string }> {
	if (obj === null || typeof obj !== 'object') return found

	const proto = Object.getPrototypeOf(obj)
	if (proto !== Object.prototype && proto !== null && !Array.isArray(obj)) {
		found.push({ path, type: obj.constructor?.name ?? 'Unknown' })
	}

	for (const [key, value] of Object.entries(obj)) {
		findNonPOJOs(value, `${path}.${key}`, found)
	}

	return found
}

export default defineNuxtPlugin((nuxtApp) => {
	if (!import.meta.server) return

	nuxtApp.hooks.hook('app:rendered', () => {
		try {
			JSON.stringify(nuxtApp.payload)
		} catch (e) {
			console.error('[payload-debugger] Payload serialization would fail:', e)
			const nonPOJOs = findNonPOJOs(nuxtApp.payload, 'payload')
			if (nonPOJOs.length > 0) {
				console.error('[payload-debugger] Non-POJO objects found in payload:')
				for (const { path, type } of nonPOJOs) {
					console.error(`  - ${path}: ${type}`)
				}
			}
		}
	})
})
