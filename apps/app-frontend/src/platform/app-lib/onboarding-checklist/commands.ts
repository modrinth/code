import { invoke } from '@tauri-apps/api/core'

/** Marks the "Create first instance" onboarding step as done without creating an instance. */
export function skipCreateInstanceOnboarding(): Promise<void> {
	return invoke('plugin:onboarding-checklist|skip_create_instance_onboarding')
}
