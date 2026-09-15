import {
	createLoadingStateCore,
	type LoadingStateProvider,
	provideLoadingState,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { getCurrentInstance, onScopeDispose } from 'vue'

import { debugStartup } from '@/helpers/startup-debug'

type AppLoadingStateProvider = Omit<LoadingStateProvider, 'begin'> & {
	begin(label?: string): symbol
}

/**
 * Source of truth for the desktop app's loading state.
 *
 * Owns the token-based ref-counter directly. Consumers
 * obtain the same reactive state via `injectLoadingState()` from `@modrinth/ui`.
 *
 * Returns the provider so the call site (App.vue) can also use it directly
 * without a second injection round-trip.
 */
export function setupLoadingStateProvider(
	getContext: () => Record<string, unknown> = () => ({}),
): AppLoadingStateProvider {
	const core = createLoadingStateCore({ barEnabled: false })
	if (!import.meta.env.DEV) {
		provideLoadingState(core)
		return core
	}
	const queryClient = useQueryClient()
	const active = new Map<symbol, { id: number; label: string; startedAt: number }>()
	let nextId = 0
	let timer: ReturnType<typeof setInterval> | undefined

	function stopReporting() {
		clearInterval(timer)
		timer = undefined
	}

	const provider: AppLoadingStateProvider = {
		...core,
		begin(label) {
			const token = core.begin()
			if (!import.meta.env.DEV || core.barEnabled.value) return token
			const instance = getCurrentInstance()
			const owner = instance?.type.name ?? instance?.type.__name ?? 'unknown component'
			const parent = instance?.parent?.type.name ?? instance?.parent?.type.__name
			const entry = {
				id: ++nextId,
				label: label ?? `${owner}${parent ? ` in ${parent}` : ''}`,
				startedAt: performance.now(),
			}
			active.set(token, entry)
			debugStartup('Splash load started', {
				id: entry.id,
				label: entry.label,
				activeCount: active.size,
				...(!label && !instance ? { caller: new Error().stack } : {}),
			})
			if (!timer) {
				timer = setInterval(() => {
					debugStartup('Splash still waiting', {
						...getContext(),
						loads: [...active.values()].map(({ id, label, startedAt }) => ({
							id,
							label,
							elapsedMs: Math.round(performance.now() - startedAt),
						})),
						observedQueries: queryClient
							.getQueryCache()
							.getAll()
							.filter(
								(query) =>
									query.getObserversCount() > 0 &&
									(query.state.status === 'pending' || query.state.fetchStatus !== 'idle'),
							)
							.map((query) => ({
								key: query.queryKey,
								status: query.state.status,
								fetchStatus: query.state.fetchStatus,
								failureCount: query.state.fetchFailureCount,
								hasData: query.state.data !== undefined,
							})),
					})
				}, 2000)
			}
			return token
		},
		end(token) {
			core.end(token)
			const entry = active.get(token)
			if (!entry) return
			active.delete(token)
			debugStartup('Splash load completed', {
				id: entry.id,
				label: entry.label,
				elapsedMs: Math.round(performance.now() - entry.startedAt),
				activeCount: active.size,
			})
			if (!active.size) stopReporting()
		},
		beginManual(durationMs = 500) {
			const token = provider.begin('Manual loading delay')
			setTimeout(() => provider.end(token), durationMs)
		},
		setEnabled(enabled) {
			core.setEnabled(enabled)
			if (enabled) {
				stopReporting()
				active.clear()
			}
		},
	}
	onScopeDispose(stopReporting)
	provideLoadingState(provider)
	return provider
}
