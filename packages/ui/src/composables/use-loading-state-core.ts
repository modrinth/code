import { computed, ref, shallowRef } from 'vue'

import type { LoadingStateProvider } from '#ui/providers/loading-state'

export interface LoadingStateCoreOptions {
	/** Initial value of the host kill-switch. Default: true. */
	barEnabled?: boolean
}

/**
 * Build a token-based `LoadingStateProvider` implementation.
 *
 * Multiple `ReadyTransition` instances (or any caller) can hold tokens at the
 * same time; the bar stays visible while at least one is live. `end(token)`
 * is idempotent so a stale token release after unmount is harmless.
 *
 * SSR safe: timers and DOM access are deferred to component code; this core
 * is pure reactive state.
 */
export function createLoadingStateCore(opts: LoadingStateCoreOptions = {}): LoadingStateProvider {
	const tokens = shallowRef<Set<symbol>>(new Set())
	const barEnabled = ref(opts.barEnabled ?? true)
	const pending = computed(() => tokens.value.size > 0)

	function begin(): symbol {
		const token = Symbol('loading-state-token')
		const next = new Set(tokens.value)
		next.add(token)
		tokens.value = next
		return token
	}

	function end(token: symbol): void {
		if (!tokens.value.has(token)) return
		const next = new Set(tokens.value)
		next.delete(token)
		tokens.value = next
	}

	function beginManual(durationMs = 500): void {
		const token = begin()
		if (typeof window === 'undefined') {
			end(token)
			return
		}
		window.setTimeout(() => end(token), durationMs)
	}

	function setEnabled(enabled: boolean): void {
		barEnabled.value = enabled
	}

	return {
		pending,
		barEnabled,
		begin,
		end,
		beginManual,
		setEnabled,
	}
}
