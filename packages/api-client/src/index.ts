export { AbstractModrinthClient } from './core/abstract-client'
export { AbstractFeature, type FeatureConfig } from './core/abstract-feature'
export { ModrinthApiError, ModrinthServerError } from './core/errors'
export { AuthFeature, type AuthConfig } from './features/auth'
export {
	CircuitBreakerFeature,
	InMemoryCircuitBreakerStorage,
	type CircuitBreakerConfig,
	type CircuitBreakerState,
	type CircuitBreakerStorage,
} from './features/circuit-breaker'
export { RetryFeature, type BackoffStrategy, type RetryConfig } from './features/retry'
export { VerboseLoggingFeature, type VerboseLoggingConfig } from './features/verbose-logging'
export type { InferredClientModules } from './modules'
export * from './modules/types'
export { GenericModrinthClient } from './platform/generic'
export { NuxtCircuitBreakerStorage, NuxtModrinthClient } from './platform/nuxt'
export type { NuxtClientConfig } from './platform/nuxt'
export { TauriModrinthClient } from './platform/tauri'
export type { TauriClientConfig } from './platform/tauri'
export * from './types'
