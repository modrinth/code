import {
	type AbstractFeature,
	type AuthConfig,
	AuthFeature,
	CircuitBreakerFeature,
	NodeAuthFeature,
	nodeAuthState,
	NuxtCircuitBreakerStorage,
	type NuxtClientConfig,
	NuxtModrinthClient,
	PanelVersionFeature,
	VerboseLoggingFeature,
} from '@modrinth/api-client'
import type { Ref } from 'vue'

async function getRateLimitKeyFromSecretsStore(): Promise<string | undefined> {
	try {
		const mod = 'cloudflare:workers'
		const { env } = await import(/* @vite-ignore */ mod)
		return await env.RATE_LIMIT_IGNORE_KEY?.get()
	} catch {
		// Not running in Cloudflare Workers environment
		return undefined
	}
}

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
		rateLimitKey: config.rateLimitKey || getRateLimitKeyFromSecretsStore,
		features: [
			// for modrinth hosting
			// is skipped for normal reqs
			new NodeAuthFeature({
				getAuth: () => nodeAuthState.getAuth?.() ?? null,
				refreshAuth: async () => {
					if (nodeAuthState.refreshAuth) {
						await nodeAuthState.refreshAuth()
					}
				},
			}),
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
