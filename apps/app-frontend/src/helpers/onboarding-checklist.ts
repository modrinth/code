import { invoke } from '@tauri-apps/api/core'

export interface OnboardingChecklist {
	has_created_instance: boolean
	has_logged_into_minecraft: boolean
	has_logged_into_modrinth: boolean
	show_checklist: boolean
}

export async function getOnboardingChecklist(): Promise<OnboardingChecklist> {
	return await invoke('plugin:onboarding-checklist|get_onboarding_checklist')
}
