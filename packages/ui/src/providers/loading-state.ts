import type { Ref } from 'vue'

import { createContext } from './create-context'

/**
 * Cross-platform loading-state contract injected by the host app.
 * Consumed by the shared `LoadingBar` and `ReadyTransition` components.
 */
export interface LoadingStateProvider {
	/** True iff at least one active load token is registered. */
	readonly pending: Readonly<Ref<boolean>>
	/** Host-level kill switch (e.g. disable the bar during a splash screen). */
	readonly barEnabled: Readonly<Ref<boolean>>
	/** Begin a tracked load. Returns a unique token; pair with `end(token)`. */
	begin(): symbol
	/** End a previously-begun load. Idempotent — unknown or repeat tokens are silently ignored. */
	end(token: symbol): void
	/** Fire a synthetic load that auto-releases after `durationMs` (default 500ms). For manual-refresh buttons. */
	beginManual(durationMs?: number): void
	/** Toggle the bar at the host level. */
	setEnabled(enabled: boolean): void
}

export const [injectLoadingState, provideLoadingState] = createContext<LoadingStateProvider>(
	'root',
	'loadingState',
)
