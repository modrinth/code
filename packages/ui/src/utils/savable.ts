import { cloneDeep, isEqual } from 'es-toolkit'
import type { ComputedRef, Ref } from 'vue'
import { computed, ref, watch } from 'vue'

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
	const currentValues = ref(cloneDeep(data())) as Ref<T>
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

	watch(
		savedValues,
		(value, previousValue) => {
			if (isEqual(currentValues.value, previousValue)) {
				currentValues.value = cloneDeep(value)
			}
		},
		{ deep: true },
	)

	const reset = () => {
		currentValues.value = cloneDeep(data())
	}

	const saveInternal = async () => {
		if (!hasChanges.value) return
		saving.value = true
		try {
			await save(changes.value)
			currentValues.value = cloneDeep(data())
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
