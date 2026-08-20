import { invoke } from '@tauri-apps/api/core'

import type { OnboardingChecklist } from '@/generated/app-events/OnboardingChecklist'

export type { OnboardingChecklist }

export async function getOnboardingChecklist(): Promise<OnboardingChecklist> {
	return await invoke('plugin:onboarding-checklist|get_onboarding_checklist')
}
