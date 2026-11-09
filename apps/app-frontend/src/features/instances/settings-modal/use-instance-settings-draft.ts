import { injectNotificationManager } from '@modrinth/ui'
import { onBeforeUnmount, onMounted, type Ref, ref } from 'vue'

import { injectInstanceSettings } from './instance-settings-context'

function clone<T>(value: T): T {
	return JSON.parse(JSON.stringify(value))
}

export function useInstanceSettingsDraft<T extends Record<string, unknown>>(
	initial: T,
	persist: (value: T) => Promise<void>,
) {
	const { handleError } = injectNotificationManager()
	const { registerUnsavedChangesController } = injectInstanceSettings()

	const original = ref(clone(initial)) as Ref<T>
	const draft = ref(clone(initial)) as Ref<T>
	const saving = ref(false)

	function hasChanges() {
		return JSON.stringify(original.value) !== JSON.stringify(draft.value)
	}

	function reset() {
		draft.value = clone(original.value)
	}

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

	onMounted(() => {
		registerUnsavedChangesController({
			hasChanges,
			getOriginal: () => original.value,
			getModified: () => draft.value,
			isSaving: () => saving.value,
			reset,
			save,
		})
	})

	onBeforeUnmount(() => {
		registerUnsavedChangesController(null)
	})

	return { draft }
}
