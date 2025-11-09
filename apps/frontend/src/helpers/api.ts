import {
	AuthFeature,
	CircuitBreakerFeature,
	NuxtCircuitBreakerStorage,
	NuxtModrinthClient,
	VerboseLoggingFeature,
	type AbstractFeature,
	type AuthConfig,
	type NuxtClientConfig,
} from '@modrinth/api-client'

export function createModrinthClient(
	auth: { token: string | undefined },
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
				token: async () => auth.token,
			} as AuthConfig),
			new CircuitBreakerFeature({
				storage: new NuxtCircuitBreakerStorage(),
				maxFailures: 3,
				resetTimeout: 30000,
			}),
			...optionalFeatures,
		],
	}

	return new NuxtModrinthClient(clientConfig)
}
