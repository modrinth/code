import {
	type AbstractFeature,
	type AuthConfig,
	AuthFeature,
	CircuitBreakerFeature,
	NuxtCircuitBreakerStorage,
	type NuxtClientConfig,
	NuxtModrinthClient,
	PanelVersionFeature,
	VerboseLoggingFeature,
} from '@modrinth/api-client'
import type { Ref } from 'vue'

export function createModrinthClient(
	auth: Ref<{ token: string | undefined }>,
	config: { apiBaseUrl: string; archonBaseUrl: string; rateLimitKey?: string },
): NuxtModrinthClient {
	const optionalFeatures = [
		import.meta.dev ? (new VerboseLoggingFeature() as AbstractFeature) : undefined,
	].filter(Boolean) as AbstractFeature[]

	const clientConfig: NuxtClientConfig = {
		labrinthBaseUrl: config.apiBaseUrl,
		archonBaseUrl: config.archonBaseUrl,
		rateLimitKey: config.rateLimitKey,
		features: [
			new AuthFeature({
				token: async () => auth.value.token,
			} as AuthConfig),
			new CircuitBreakerFeature({
				storage: new NuxtCircuitBreakerStorage(),
				maxFailures: 3,
				resetTimeout: 30000,
			}),
			new PanelVersionFeature(),
			...optionalFeatures,
		],
	}

	return new NuxtModrinthClient(clientConfig)
}
