import { computed, type MaybeRefOrGetter, shallowRef, toValue, watch } from 'vue'

import { injectIconCache } from '#ui/providers/icon-cache'

export function useCachedIcon(source: MaybeRefOrGetter<string | null | undefined>) {
	const cache = injectIconCache(null)
	const input = computed(() => toValue(source) ?? undefined)
	const isCacheable = (value: string | undefined): value is string =>
		!!cache && !!value?.startsWith('https://')
	const resolved = shallowRef<{ source: string; url: string }>()

	watch(
		input,
		async (value, _, onCleanup) => {
			if (!isCacheable(value) || !cache) return
			const cached = cache.peekIcon?.(value)
			if (cached) {
				resolved.value = { source: value, url: cached }
				return
			}
			let active = true
			onCleanup(() => {
				active = false
			})
			try {
				const url = await cache.cacheIcon(value)
				if (active) resolved.value = { source: value, url }
			} catch (error) {
				console.warn('Could not cache icon', error)
				if (active) resolved.value = { source: value, url: value }
			}
		},
		{ immediate: true },
	)

	return computed(() => {
		const value = input.value
		if (!isCacheable(value)) return value
		return resolved.value?.source === value ? resolved.value.url : undefined
	})
}
