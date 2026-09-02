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

import { useFeatureFlags } from '~/composables/featureFlags.ts'
import { useVisitorUserAgent } from '~/composables/visitor-user-agent.ts'
import { withStagingArchonBaseUrl } from '~/helpers/archon.ts'
import { readEnv } from '~/helpers/env.ts'
import { getFrontendUserAgent, VISITOR_USER_AGENT_HEADER } from '~/helpers/user-agent.ts'

export function createModrinthClient(
	auth: Ref<{ token: string | undefined }>,
	config: {
		apiBaseUrl: string
		archonBaseUrl: string
		sharedInstancesBaseUrl: string
		commitHash: string
		rateLimitKey?: string
	},
): NuxtModrinthClient {
	const flags = useFeatureFlags()
	const visitorUserAgent = useVisitorUserAgent()
	const optionalFeatures = [
		import.meta.dev ? (new VerboseLoggingFeature() as AbstractFeature) : undefined,
	].filter(Boolean) as AbstractFeature[]

	const clientConfig: NuxtClientConfig = {
		labrinthBaseUrl: config.apiBaseUrl,
		archonBaseUrl: () =>
			withStagingArchonBaseUrl(config.archonBaseUrl, flags.value.archonApiStaging),
		sharedInstancesBaseUrl: config.sharedInstancesBaseUrl,
		userAgent: () => (import.meta.server ? getFrontendUserAgent(config.commitHash) : undefined),
		headers: visitorUserAgent ? { [VISITOR_USER_AGENT_HEADER]: visitorUserAgent } : undefined,
		archonSentryCapture: () => flags.value.archonSentryCapture,
		rateLimitKey: config.rateLimitKey || (() => readEnv('RATE_LIMIT_IGNORE_KEY')),
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
