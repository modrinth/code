import type { LoadingStateProvider } from '@icarus/ui'
import { createLoadingStateCore, provideLoadingState } from '@icarus/ui'

/**
 * Source of truth for the desktop app's loading state.
 *
 * Owns the token-based ref-counter directly (no Pinia store). Consumers
 * obtain the same reactive state via `injectLoadingState()` from `@icarus/ui`.
 *
 * Returns the provider so the call site (App.vue) can also use it directly
 * without a second injection round-trip.
 */
export function setupLoadingStateProvider(): LoadingStateProvider {
	const provider = createLoadingStateCore({ barEnabled: false })
	provideLoadingState(provider)
	return provider
}
