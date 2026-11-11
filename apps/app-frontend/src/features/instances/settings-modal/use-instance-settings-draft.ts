import { injectNotificationManager } from '@modrinth/ui'
import { onBeforeUnmount, onMounted, type Ref, ref, watch } from 'vue'

import { injectInstanceSettings } from './instance-settings-context'

function clone<T>(value: T): T {
	return JSON.parse(JSON.stringify(value))
}

function isEqual(a: unknown, b: unknown): boolean {
	return JSON.stringify(a) === JSON.stringify(b)
}

export function useInstanceSettingsDraft<T extends Record<string, unknown>>(
	initial: () => T,
	persist: (value: T) => Promise<void>,
): Ref<T> {
	const { handleError } = injectNotificationManager()
	const { registerUnsavedChangesController } = injectInstanceSettings()

	const original = ref(clone(initial())) as Ref<T>
	const draft = ref(clone(initial())) as Ref<T>
	const saving = ref(false)

	watch(initial, (next) => {
		if (saving.value || isEqual(next, original.value)) return
		if (!isEqual(draft.value, original.value)) return
		original.value = clone(next)
		draft.value = clone(next)
	})

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
			hasChanges: () => !isEqual(original.value, draft.value),
			getOriginal: () => original.value,
			getModified: () => draft.value,
			isSaving: () => saving.value,
			reset: () => {
				draft.value = clone(original.value)
			},
			save,
		})
	})

	onBeforeUnmount(() => {
		registerUnsavedChangesController(null)
	})

	return draft
}
