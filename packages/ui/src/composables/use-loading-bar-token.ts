import type { Ref } from 'vue'
import { onBeforeUnmount, watch } from 'vue'

import { injectLoadingState } from '#ui/providers/loading-state'

/**
 * Register a `LoadingBar` token for as long as `pending` is truthy.
 *
 * Use this when the component that owns the load is not the natural place
 * to mount a `<ReadyTransition>` (e.g. a page root with a complex v-if
 * cascade where wrapping the template is awkward). `<ReadyTransition>`
 * remains the preferred API when it fits.
 *
 * Safe to call without a provider mounted; becomes a no-op.
 */
export function useLoadingBarToken(pending: Ref<boolean>): void {
	const loadingState = injectLoadingState(null)
	if (!loadingState) return

	let token: symbol | null = null

	function release() {
		if (token) {
			loadingState.end(token)
			token = null
		}
	}

	watch(
		pending,
		(now) => {
			if (typeof window === 'undefined') return
			if (now && !token) {
				token = loadingState.begin()
			} else if (!now) {
				release()
			}
		},
		{ immediate: true },
	)

	onBeforeUnmount(release)
}
