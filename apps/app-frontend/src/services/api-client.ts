import {
	AuthFeature,
	NodeAuthFeature,
	nodeAuthState,
	PanelVersionFeature,
	SelfHostedFeature,
	TauriModrinthClient,
	VerboseLoggingFeature,
} from '@modrinth/api-client'
import { getVersion } from '@tauri-apps/api/app'

import { config } from '@/config'
import { get as getModrinthCredentials } from '@/helpers/mr_auth'

const appVersion = getVersion()

export const apiClient: TauriModrinthClient = new TauriModrinthClient({
	userAgent: async () => `modrinth/theseus/${await appVersion}`,
	labrinthBaseUrl: config.labrinthBaseUrl,
	// With our own account backend, hosting is the account's paired Cores, found through it.
	selfHostedHosting: Boolean(config.accountApiUrl),
	archonBaseUrl: config.archonBaseUrl,
	sharedInstancesBaseUrl: config.sharedInstancesBaseUrl,
	features: [
		...(config.accountApiUrl
			? [
					new SelfHostedFeature({
						baseUrl: config.accountApiUrl,
						isCoreUrl: (url: string): boolean => apiClient.archon.cores_v1.isCoreUrl(url),
						token: async () => (await getModrinthCredentials())?.session,
					}),
				]
			: []),
		new NodeAuthFeature({
			getAuth: () => nodeAuthState.getAuth?.() ?? null,
			refreshAuth: async () => await nodeAuthState.refreshAuth?.(),
		}),
		new AuthFeature({
			token: async () => (await getModrinthCredentials())?.session,
		}),
		new PanelVersionFeature(),
		new VerboseLoggingFeature(),
	],
})

export { appVersion }
