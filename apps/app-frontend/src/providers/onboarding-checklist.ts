import { createContext } from '@modrinth/ui'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { computed, type ComputedRef, onUnmounted, ref } from 'vue'

import {
	getOnboardingChecklist,
	type OnboardingChecklist,
} from '@/helpers/onboarding-checklist'

export interface OnboardingChecklistContext {
	hasCreatedInstance: ComputedRef<boolean>
	hasLoggedIntoMinecraft: ComputedRef<boolean>
	hasLoggedIntoModrinth: ComputedRef<boolean>
	isReady: ComputedRef<boolean>
	showChecklist: ComputedRef<boolean>
}

export interface OnboardingChecklistProvider extends OnboardingChecklistContext {
	initialize: () => Promise<void>
}

export const [injectOnboardingChecklist, provideOnboardingChecklist] =
	createContext<OnboardingChecklistContext>('root', 'onboardingChecklist')

export function setupOnboardingChecklistProvider(): OnboardingChecklistProvider {
	const checklist = ref<OnboardingChecklist>()
	let unlisten: UnlistenFn | undefined

	const context: OnboardingChecklistContext = {
		hasCreatedInstance: computed(() => checklist.value?.has_created_instance ?? false),
		hasLoggedIntoMinecraft: computed(
			() => checklist.value?.has_logged_into_minecraft ?? false,
		),
		hasLoggedIntoModrinth: computed(
			() => checklist.value?.has_logged_into_modrinth ?? false,
		),
		isReady: computed(() => checklist.value !== undefined),
		showChecklist: computed(() => checklist.value?.show_checklist ?? false),
	}

	provideOnboardingChecklist(context)

	const applyChecklist = (nextChecklist: OnboardingChecklist) => {
		checklist.value = {
			has_created_instance:
				(checklist.value?.has_created_instance ?? false) ||
				nextChecklist.has_created_instance,
			has_logged_into_minecraft:
				(checklist.value?.has_logged_into_minecraft ?? false) ||
				nextChecklist.has_logged_into_minecraft,
			has_logged_into_modrinth:
				(checklist.value?.has_logged_into_modrinth ?? false) ||
				nextChecklist.has_logged_into_modrinth,
			show_checklist:
				(checklist.value?.show_checklist ?? true) && nextChecklist.show_checklist,
		}
	}

	const initialize = async () => {
		unlisten = await listen<OnboardingChecklist>('onboarding_checklist', (event) => {
			applyChecklist(event.payload)
		})
		applyChecklist(await getOnboardingChecklist())
	}

	onUnmounted(() => unlisten?.())

	return { ...context, initialize }
}
