export { AbstractModrinthClient } from './core/abstract-client'
export { AbstractFeature, type FeatureConfig } from './core/abstract-feature'
export { ModrinthApiError, ModrinthServerError } from './core/errors'
export { type AuthConfig, AuthFeature } from './features/auth'
export {
	type CircuitBreakerConfig,
	CircuitBreakerFeature,
	type CircuitBreakerState,
	type CircuitBreakerStorage,
	InMemoryCircuitBreakerStorage,
} from './features/circuit-breaker'
export { type BackoffStrategy, type RetryConfig, RetryFeature } from './features/retry'
export { type VerboseLoggingConfig, VerboseLoggingFeature } from './features/verbose-logging'
export type { InferredClientModules } from './modules'
export * from './modules/types'
export { GenericModrinthClient } from './platform/generic'
export type { NuxtClientConfig } from './platform/nuxt'
export { NuxtCircuitBreakerStorage, NuxtModrinthClient } from './platform/nuxt'
export type { TauriClientConfig } from './platform/tauri'
export { TauriModrinthClient } from './platform/tauri'
export * from './types'
