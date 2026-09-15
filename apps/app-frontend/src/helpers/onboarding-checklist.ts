import { invoke } from '@tauri-apps/api/core'

import type { OnboardingChecklist } from '@/generated/app-events/OnboardingChecklist'

export type { OnboardingChecklist }

export async function getOnboardingChecklist(): Promise<OnboardingChecklist> {
	return await invoke('plugin:onboarding-checklist|get_onboarding_checklist')
}

/** Complete the site-account onboarding step after Owyx email login. */
export async function markLoggedIntoOwyxSite(): Promise<void> {
	await invoke('plugin:onboarding-checklist|mark_logged_into_owyx_site')
}
