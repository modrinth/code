import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

import { getIconSymbolAsset } from '#ui/components/base/icon-editor-modal/editor-catalog'
import { renderIcon } from '#ui/components/base/icon-editor-modal/render-icon'
import type { IconConfig } from '#ui/components/base/icon-editor-modal/types'

export function blobToDataUrl(blob: Blob): Promise<string> {
	return new Promise((resolve, reject) => {
		const reader = new FileReader()
		reader.onload = () => resolve(reader.result as string)
		reader.onerror = () => reject(reader.error)
		reader.readAsDataURL(blob)
	})
}

/** Renders an icon editor config to a PNG data URL, for places that need an image URL. */
export function useGeneratedIconSrc(config: MaybeRefOrGetter<IconConfig | null | undefined>) {
	const symbolAsset = computed(() => {
		const value = toValue(config)
		return value ? getIconSymbolAsset(value.symbol) : undefined
	})
	const query = useQuery({
		queryKey: computed(
			() => ['icons', 'generated', JSON.stringify(toValue(config) ?? null)] as const,
		),
		queryFn: async () => {
			const value = toValue(config)
			if (!value || !symbolAsset.value) return null
			return await blobToDataUrl(await renderIcon(value, symbolAsset.value))
		},
		enabled: computed(() => typeof window !== 'undefined' && !!symbolAsset.value),
		staleTime: Infinity,
	})

	return computed(() => (symbolAsset.value ? (query.data.value ?? undefined) : undefined))
}
