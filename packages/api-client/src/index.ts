export { AbstractModrinthClient } from './core/abstract-client'
export { AbstractFeature, type FeatureConfig } from './core/abstract-feature'
export { AbstractUploadClient } from './core/abstract-upload-client'
export {
	AbstractWebSocketClient,
	type WebSocketConnection,
	type WebSocketEventHandler,
	type WebSocketStatus
} from './core/abstract-websocket'
export { ModrinthApiError, ModrinthServerError } from './core/errors'
export { AuthFeature, type AuthConfig } from './features/auth'
export {
	CircuitBreakerFeature, InMemoryCircuitBreakerStorage, type CircuitBreakerConfig, type CircuitBreakerState,
	type CircuitBreakerStorage
} from './features/circuit-breaker'
export { NodeAuthFeature, type NodeAuth, type NodeAuthConfig } from './features/node-auth'
export { PANEL_VERSION, PanelVersionFeature } from './features/panel-version'
export { RetryFeature, type BackoffStrategy, type RetryConfig } from './features/retry'
export { VerboseLoggingFeature, type VerboseLoggingConfig } from './features/verbose-logging'
export type { InferredClientModules } from './modules'
export * from './modules/types'
export { GenericModrinthClient } from './platform/generic'
export { NuxtCircuitBreakerStorage, NuxtIcarusClient } from './platform/nuxt'
export type { NuxtClientConfig } from './platform/nuxt'
export { TauriIcarusClient } from './platform/tauri'
export type { TauriClientConfig } from './platform/tauri'
export { XHRUploadClient } from './platform/xhr-upload-client'
export { clearNodeAuthState, nodeAuthState, setNodeAuthState } from './state/node-auth'
export * from './types'
export { withJWTRetry } from './utils/jwt-retry'
export type { Override, RawDecimal } from './utils/types'

