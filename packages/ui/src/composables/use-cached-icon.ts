import { computed, type MaybeRefOrGetter, shallowRef, toValue, watch } from 'vue'

import { injectIconCache } from '#ui/providers/icon-cache'

export function useCachedIcon(source: MaybeRefOrGetter<string | Blob | null | undefined>) {
	const cache = injectIconCache(null)
	const input = computed(() => toValue(source) ?? undefined)
	const resolved = shallowRef<{ source: string | Blob; url: string }>()

	watch(
		input,
		async (value, _, onCleanup) => {
			resolved.value = undefined
			if (!value || typeof window === 'undefined') return
			if (typeof value === 'string' && (!cache || !value.startsWith('https://'))) return
			let active = true
			let objectUrl: string | undefined
			onCleanup(() => {
				active = false
				if (objectUrl) URL.revokeObjectURL(objectUrl)
			})
			const fallback = () => {
				if (typeof value === 'string') return value
				objectUrl = URL.createObjectURL(value)
				return objectUrl
			}
			try {
				const url = cache ? await cache.cacheIcon(value) : fallback()
				if (active) resolved.value = { source: value, url }
			} catch (error) {
				console.warn('Could not cache icon', error)
				if (active) resolved.value = { source: value, url: fallback() }
			}
		},
		{ immediate: true },
	)

	return computed(() => {
		const value = input.value
		if (!value) return undefined
		if (typeof value === 'string' && (!cache || !value.startsWith('https://'))) return value
		return resolved.value?.source === value ? resolved.value.url : undefined
	})
}
