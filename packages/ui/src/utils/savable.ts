import type { ComputedRef, Ref } from 'vue'
import { computed, ref } from 'vue'

export function useSavable<T extends Record<string, unknown>>(
	data: () => T,
	save: (changes: Partial<T>) => void,
): {
	saved: ComputedRef<T>
	current: Ref<T>
	reset: () => void
	save: () => void
} {
	const savedValues = computed(data)
	const currentValues = ref({ ...data() }) as Ref<T>
	const changes = computed<Partial<T>>(() => {
		const values: Partial<T> = {}
		const keys = Object.keys(currentValues.value) as (keyof T)[]
		for (const key of keys) {
			if (savedValues.value[key] !== currentValues.value[key]) {
				values[key] = currentValues.value[key]
			}
		}
		return values
	})

	const reset = () => {
		currentValues.value = data()
	}

	const saveInternal = () => (changes.value ? save(changes.value) : {})

	return {
		saved: savedValues,
		current: currentValues,
		reset,
		save: saveInternal,
	}
}
