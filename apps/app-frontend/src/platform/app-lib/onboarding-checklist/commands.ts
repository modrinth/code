import { invoke } from '@tauri-apps/api/core'

export function skipCreateInstanceOnboarding(): Promise<void> {
	return invoke('plugin:onboarding-checklist|skip_create_instance_onboarding')
}
