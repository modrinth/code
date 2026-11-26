import { computed, type Ref, type WritableComputedRef } from 'vue'

export type SettingsSource = 'app' | 'custom'

export function useSettingsSource<T extends Record<string, unknown>>(
	draft: Ref<T>,
	defaults: () => { [K in keyof T]: NonNullable<T[K]> },
): WritableComputedRef<SettingsSource> {
	let lastCustom: T | null = null

	const source = computed<SettingsSource>({
		get: () => (Object.values(draft.value).some((value) => value !== null) ? 'custom' : 'app'),
		set: (next) => {
			if (next === source.value) return
			if (next === 'app') {
				lastCustom = draft.value
				draft.value = Object.fromEntries(Object.keys(draft.value).map((key) => [key, null])) as T
			} else {
				draft.value = lastCustom ?? (JSON.parse(JSON.stringify(defaults())) as T)
			}
		},
	})

	return source
}
