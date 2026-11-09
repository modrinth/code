import { injectNotificationManager } from '@modrinth/ui'
import { onBeforeUnmount, onMounted, type Ref, ref } from 'vue'

import type { UnsavedChangesController } from '@/features/settings/app-settings-modal/context'

import { injectInstanceSettings } from './instance-settings-context'

function clone<T>(value: T): T {
	return JSON.parse(JSON.stringify(value))
}

export function useInstanceSettingsDraft() {
	const { handleError } = injectNotificationManager()
	const { registerUnsavedChangesController } = injectInstanceSettings()

	let controller: UnsavedChangesController | null = null

	onMounted(() => {
		registerUnsavedChangesController(controller)
	})

	onBeforeUnmount(() => {
		registerUnsavedChangesController(null)
	})

	function createDraft<T extends Record<string, unknown>>(
		initial: T,
		persist: (value: T) => Promise<void>,
	) {
		const original = ref(clone(initial)) as Ref<T>
		const draft = ref(clone(initial)) as Ref<T>
		const saving = ref(false)

		async function save() {
			if (saving.value) return
			saving.value = true
			const value = clone(draft.value)
			try {
				await persist(value)
				original.value = value
			} catch (error) {
				handleError(error as Error)
			} finally {
				saving.value = false
			}
		}

		controller = {
			hasChanges: () => JSON.stringify(original.value) !== JSON.stringify(draft.value),
			getOriginal: () => original.value,
			getModified: () => draft.value,
			isSaving: () => saving.value,
			reset: () => {
				draft.value = clone(original.value)
			},
			save,
		}

		return draft
	}

	return { createDraft }
}
