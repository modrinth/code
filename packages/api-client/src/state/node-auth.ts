import type { NodeAuth } from '../features/node-auth'

/**
 * Global node auth state.
 * Set by server management pages, read by NodeAuthFeature.
 */
export const nodeAuthState = {
	getAuth: null as (() => NodeAuth | null) | null,
	refreshAuth: null as (() => Promise<void>) | null,
}

/**
 * Configure the node auth state. Call this when entering server management.
 *
 * @param getAuth - Function that returns current auth or null
 * @param refreshAuth - Function to refresh the auth token
 *
 * @example
 * ```typescript
 * // In server management page setup
 * setNodeAuthState(
 *   () => fsAuth.value,
 *   refreshFsAuth,
 * )
 * ```
 */
export function setNodeAuthState(getAuth: () => NodeAuth | null, refreshAuth: () => Promise<void>) {
	nodeAuthState.getAuth = getAuth
	nodeAuthState.refreshAuth = refreshAuth
}

/**
 * Clear the node auth state. Call this when leaving server management.
 *
 * @example
 * ```typescript
 * onUnmounted(() => {
 *   clearNodeAuthState()
 * })
 * ```
 */
export function clearNodeAuthState() {
	nodeAuthState.getAuth = null
	nodeAuthState.refreshAuth = null
}
