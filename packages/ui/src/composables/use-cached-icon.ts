import { computed, type MaybeRefOrGetter, shallowRef, toValue, watch } from 'vue'

import { injectIconCache } from '#ui/providers/icon-cache'

export function useCachedIcon(source: MaybeRefOrGetter<string | Blob | null | undefined>) {
	const cache = injectIconCache(null)
	const input = computed(() => toValue(source) ?? undefined)

	const needsResolving = (value: string | Blob | undefined) =>
		typeof value === 'string' ? !!cache && value.startsWith('https://') : !!value
	const resolved = shallowRef<{ source: string | Blob; url: string }>()

	watch(
		input,
		async (value, _, onCleanup) => {
			if (!value || !needsResolving(value) || typeof window === 'undefined') return
			const cached = cache?.peekIcon(value)
			if (cached) {
				resolved.value = { source: value, url: cached }
				return
			}
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

	const pending = computed(
		() => needsResolving(input.value) && resolved.value?.source !== input.value,
	)
	const src = computed(() => {
		const value = input.value
		if (typeof value === 'string' && !needsResolving(value)) return value
		return pending.value ? undefined : resolved.value?.url
	})

	return { src, pending }
}
