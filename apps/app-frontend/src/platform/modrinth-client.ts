import {
	AuthFeature,
	NodeAuthFeature,
	nodeAuthState,
	PanelVersionFeature,
	TauriModrinthClient,
	VerboseLoggingFeature,
} from '@modrinth/api-client'
import { getVersion } from '@tauri-apps/api/app'

import { config } from '@/config'
import { get as getCreds } from '@/helpers/mr_auth'

export function createAppClient(): TauriModrinthClient {
	const appVersion = getVersion()

	return new TauriModrinthClient({
		userAgent: async () => `modrinth/theseus/${await appVersion} (support@modrinth.com)`,
		labrinthBaseUrl: config.labrinthBaseUrl,
		archonBaseUrl: config.archonBaseUrl,
		sharedInstancesBaseUrl: config.sharedInstancesBaseUrl,
		features: [
			new NodeAuthFeature({
				getAuth: () => nodeAuthState.getAuth?.() ?? null,
				refreshAuth: async () => {
					if (nodeAuthState.refreshAuth) {
						await nodeAuthState.refreshAuth()
					}
				},
			}),
			new AuthFeature({
				token: async () => (await getCreds())?.session,
			}),
			new PanelVersionFeature(),
			new VerboseLoggingFeature(),
		],
	})
}
