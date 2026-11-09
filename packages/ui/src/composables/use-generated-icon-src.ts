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

export function generatedIconSrcQueryOptions(config: IconConfig | null) {
	return {
		queryKey: ['icons', 'generated', JSON.stringify(config)] as const,
		queryFn: async () => {
			const symbolAsset = config ? getIconSymbolAsset(config.symbol) : undefined
			if (!config || !symbolAsset) return null
			return await blobToDataUrl(await renderIcon(config, symbolAsset))
		},
		staleTime: Infinity,
	}
}

/** Renders an icon editor config to a PNG data URL, for places that need an image URL. */
export function useGeneratedIconSrc(config: MaybeRefOrGetter<IconConfig | null | undefined>) {
	const value = computed(() => {
		const current = toValue(config)
		return current && getIconSymbolAsset(current.symbol) ? current : null
	})
	const query = useQuery(
		computed(() => ({
			...generatedIconSrcQueryOptions(value.value),
			enabled: typeof window !== 'undefined' && !!value.value,
		})),
	)

	return computed(() => (value.value ? (query.data.value ?? undefined) : undefined))
}
