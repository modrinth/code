import type { AuthConfig, NuxtClientConfig } from '@modrinth/api-client'
import {
	AuthFeature,
	CircuitBreakerFeature,
	NuxtCircuitBreakerStorage,
	NuxtModrinthClient,
} from '@modrinth/api-client'
import { createContext } from '@modrinth/ui'

export function createModrinthClient(
	auth: { token: string | undefined },
	config: { apiBaseUrl: string; rateLimitKey?: string },
): NuxtModrinthClient {
	const clientConfig: NuxtClientConfig = {
		labrinthBaseUrl: config.apiBaseUrl,
		rateLimitKey: config.rateLimitKey,
		features: [
			new AuthFeature({
				token: async () => auth.token!,
			} as AuthConfig),
			new CircuitBreakerFeature({
				storage: new NuxtCircuitBreakerStorage(),
				maxFailures: 3,
				resetTimeout: 30000,
			}),
		],
	}

	return new NuxtModrinthClient(clientConfig)
}

export const [injectModrinthClient, provideModrinthClient] = createContext<NuxtModrinthClient>(
	'root',
	'modrinthClient',
)
