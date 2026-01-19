import { stringify } from 'devalue'

function findNonPOJOs(
	obj: unknown,
	path: string,
	found: Array<{ path: string; type: string }> = [],
	seen = new WeakSet(),
): Array<{ path: string; type: string }> {
	if (obj === null || typeof obj !== 'object') return found

	// Prevent circular reference infinite loops
	if (seen.has(obj)) return found
	seen.add(obj)

	if (typeof obj === 'function') {
		found.push({ path, type: 'Function' })
		return found
	}

	const proto = Object.getPrototypeOf(obj)
	const constructorName = obj.constructor?.name ?? 'Unknown'

	// Check for non-POJOs (not Object, Array, or null prototype)
	if (proto !== Object.prototype && proto !== null && !Array.isArray(obj)) {
		found.push({ path, type: constructorName })
	}

	// Check for Vue internals that shouldn't be serialized
	if ('__v_isRef' in obj || '__v_isReactive' in obj || '_rawValue' in obj) {
		found.push({ path, type: `Vue:${constructorName}` })
	}

	// Check for IntlMessageFormat specifically
	if ('_ast' in obj || constructorName === 'IntlMessageFormat') {
		found.push({ path, type: 'IntlMessageFormat' })
	}

	try {
		for (const [key, value] of Object.entries(obj)) {
			findNonPOJOs(value, `${path}.${key}`, found, seen)
		}
	} catch {
		found.push({ path, type: `NonIterable:${constructorName}` })
	}

	return found
}

function checkPayload(payload: unknown, hookName: string): void {
	try {
		stringify(payload)
	} catch (e) {
		const nonPOJOs = findNonPOJOs(payload, 'payload')
		console.error(`[payload-debugger] [${hookName}] Devalue serialization failed:`, e)
		if (nonPOJOs.length > 0) {
			console.error(`[payload-debugger] [${hookName}] Non-POJO objects found:`)
			for (const { path, type } of nonPOJOs.slice(0, 20)) {
				console.error(`  - ${path}: ${type}`)
			}
			if (nonPOJOs.length > 20) {
				console.error(`  ... and ${nonPOJOs.length - 20} more`)
			}
		} else {
			console.error(
				`[payload-debugger] [${hookName}] No non-POJOs found by walker - issue may be circular refs or special values`,
			)
		}
	}
}

export default defineNuxtPlugin((nuxtApp) => {
	if (!import.meta.server) return

	nuxtApp.hooks.hook('app:rendered', () => {
		checkPayload(nuxtApp.payload, 'app:rendered')
	})

	if (nuxtApp.payload.data && typeof nuxtApp.payload.data === 'object') {
		const originalData = nuxtApp.payload.data
		const allowedConstructors = new Set(['Object', 'Array', 'String', 'Number', 'Boolean', 'Date'])

		nuxtApp.payload.data = new Proxy(originalData, {
			set(target, prop, value) {
				if (value !== null && typeof value === 'object') {
					const constructorName = value.constructor?.name
					if (constructorName && !allowedConstructors.has(constructorName)) {
						console.error(
							`[payload-debugger] [proxy] Non-POJO assigned to payload.data.${String(prop)}:`,
							constructorName,
						)
						console.error(new Error('Stack trace').stack)
					}
				}
				;(target as Record<string | symbol, unknown>)[prop] = value
				return true
			},
		})
	}
})
