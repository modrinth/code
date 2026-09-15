import { useDebugLogger } from '@modrinth/ui'

const debug = useDebugLogger('Startup')

export function debugStartup(event: string, details: Record<string, unknown> = {}): void {
	if (!import.meta.env.DEV) return
	debug(
		event,
		JSON.stringify({
			at: new Date().toISOString(),
			sinceNavigationMs: Math.round(performance.now()),
			...details,
		}),
	)
}

export async function traceStartupStep<T>(label: string, run: () => Promise<T>): Promise<T> {
	if (!import.meta.env.DEV) return run()
	const startedAt = performance.now()
	debugStartup('Step started', { label })
	const timer = setTimeout(() => {
		debugStartup('Step still pending', {
			label,
			elapsedMs: Math.round(performance.now() - startedAt),
		})
	}, 2000)
	try {
		const result = await run()
		debugStartup('Step completed', { label, elapsedMs: Math.round(performance.now() - startedAt) })
		return result
	} catch (error) {
		debugStartup('Step failed', {
			label,
			elapsedMs: Math.round(performance.now() - startedAt),
			errorType: error instanceof Error ? error.name : typeof error,
		})
		throw error
	} finally {
		clearTimeout(timer)
	}
}
