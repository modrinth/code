import { computed, type MaybeRefOrGetter, shallowRef, toValue, watch } from 'vue'

import { injectIconCache } from '#ui/providers/icon-cache'

export function useCachedIcon(source: MaybeRefOrGetter<string | null | undefined>) {
	const cache = injectIconCache(null)
	const input = computed(() => toValue(source) ?? undefined)
	if (!cache) return { src: input, pending: computed(() => false) }

	const isCacheable = (value: string | undefined): value is string =>
		!!value?.startsWith('https://')
	const resolved = shallowRef<{ source: string; url: string }>()

	watch(
		input,
		async (value, _, onCleanup) => {
			if (!isCacheable(value)) return
			const cached = cache.peekIcon(value)
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

	const pending = computed(() => isCacheable(input.value) && resolved.value?.source !== input.value)
	const src = computed(() => {
		const value = input.value
		if (!isCacheable(value)) return value
		return pending.value ? undefined : resolved.value?.url
	})

	return { src, pending }
}
