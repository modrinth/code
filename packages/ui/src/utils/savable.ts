import { isEqual } from 'lodash-es'
import type { ComputedRef, Ref } from 'vue'
import { computed, ref } from 'vue'

export function useSavable<T extends Record<string, unknown>>(
	data: () => T,
	save: (changes: Partial<T>) => void | Promise<void>,
): {
	saved: ComputedRef<T>
	current: Ref<T>
	changes: ComputedRef<Partial<T>>
	hasChanges: ComputedRef<boolean>
	saving: Ref<boolean>
	reset: () => void
	save: () => Promise<void>
} {
	const savedValues = computed(data)
	const currentValues = ref({ ...data() }) as Ref<T>
	const saving = ref(false)

	const changes = computed<Partial<T>>(() => {
		const values: Partial<T> = {}
		const keys = Object.keys(currentValues.value) as (keyof T)[]
		for (const key of keys) {
			if (!isEqual(savedValues.value[key], currentValues.value[key])) {
				values[key] = currentValues.value[key]
			}
		}
		return values
	})

	const hasChanges = computed(() => Object.keys(changes.value).length > 0)

	const reset = () => {
		currentValues.value = data()
	}

	const saveInternal = async () => {
		if (!hasChanges.value) return
		saving.value = true
		try {
			await save(changes.value)
		} finally {
			saving.value = false
		}
	}

	return {
		saved: savedValues,
		current: currentValues,
		changes,
		hasChanges,
		saving,
		reset,
		save: saveInternal,
	}
}
