import type { LoadingStateProvider } from '@modrinth/ui'
import { createLoadingStateCore, provideLoadingState } from '@modrinth/ui'
import { watch } from 'vue'

/**
 * Initialize the cross-platform loading-state provider for the website.
 *
 * Responsibilities:
 * 1. Own the token-based ref-counter that drives `LoadingBar` and `ReadyTransition`.
 * 2. Bridge the legacy `useState('loading')` global so the many existing
 *    `startLoading()` / `stopLoading()` call sites continue to raise the bar.
 * 3. Register Nuxt `page:start` / `page:finish` hooks so route navigation
 *    auto-fires the bar (replaces the behavior previously inside
 *    `modrinth-loading-indicator.ts`).
 */
export function setupLoadingStateProvider(): LoadingStateProvider {
	const provider = createLoadingStateCore({ barEnabled: true })
	provideLoadingState(provider)

	const legacyState = useLoading()
	let legacyToken: symbol | null = null
	watch(
		legacyState,
		(value) => {
			if (value && !legacyToken) {
				legacyToken = provider.begin()
			} else if (!value && legacyToken) {
				provider.end(legacyToken)
				legacyToken = null
			}
		},
		{ immediate: true },
	)

	const nuxtApp = useNuxtApp()
	let pageToken: symbol | null = null
	nuxtApp.hook('page:start', () => {
		if (pageToken) provider.end(pageToken)
		pageToken = provider.begin()
	})
	nuxtApp.hook('page:finish', () => {
		if (pageToken) {
			provider.end(pageToken)
			pageToken = null
		}
	})

	return provider
}
