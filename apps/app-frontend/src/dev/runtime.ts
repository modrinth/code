import { invoke } from '@tauri-apps/api/core'
import { applyDevAppConfig } from '@/config'

export interface DevAppConfig {
	credentialNamespace: string
	authMode: 'dev' | 'real'
	username: string | null
	branch: string
	title: string
	dataDir: string
	backendUrl?: string
}

let config: DevAppConfig | null = null

export async function initializeDevRuntime(): Promise<void> {
	config = await invoke<DevAppConfig | null>('get_amberite_dev_config')
	if (!config) return
	applyDevAppConfig(config)
}

export function getDevAppConfig(): DevAppConfig | null {
	return config
}
