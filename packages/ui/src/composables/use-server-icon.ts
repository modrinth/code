import { type Archon, ModrinthApiError } from '@modrinth/api-client'
import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

import { getIconSymbolAsset } from '#ui/components/base/icon-editor-modal/editor-catalog'
import { injectModrinthClient } from '#ui/providers'

import { blobToDataUrl } from './use-generated-icon-src'

export const serverIconKeys = {
	metadata: (serverId: string) => ['servers', 'detail', serverId, 'icon'] as const,
	image: (serverId: string, hash: string | undefined) =>
		['servers', 'detail', serverId, 'icon', 'image', hash] as const,
}

type UseServerIconOptions = {
	/** Icon metadata already loaded with the server. Skips fetching the metadata separately. */
	icon?: MaybeRefOrGetter<Archon.Icons.v1.ServerIcon | null | undefined>
	enabled?: MaybeRefOrGetter<boolean>
}

/**
 * Resolves a server's custom icon. Icons created with the icon editor are returned as their `config`
 * so they can be drawn locally at any size; other icons are fetched as an `image` data URL.
 */
export function useServerIcon(
	serverId: MaybeRefOrGetter<string>,
	options: UseServerIconOptions = {},
) {
	const client = injectModrinthClient()
	const enabled = computed(
		() =>
			typeof window !== 'undefined' && !!toValue(serverId) && (toValue(options.enabled) ?? true),
	)

	const metadataQuery = useQuery({
		queryKey: computed(() => serverIconKeys.metadata(toValue(serverId))),
		queryFn: async (): Promise<Archon.Icons.v1.ServerIcon | null> => {
			try {
				return await client.archon.icons_v1.getMetadata(toValue(serverId))
			} catch (error) {
				if (error instanceof ModrinthApiError && error.statusCode === 404) return null
				throw error
			}
		},
		enabled: computed(() => enabled.value && toValue(options.icon) === undefined),
		staleTime: 60_000,
	})

	const metadata = computed(() =>
		metadataQuery.data.value !== undefined ? metadataQuery.data.value : toValue(options.icon),
	)
	const config = computed(() => {
		const value = metadata.value?.config
		return value && getIconSymbolAsset(value.symbol) ? value : null
	})

	const imageQuery = useQuery({
		queryKey: computed(() => serverIconKeys.image(toValue(serverId), metadata.value?.hash)),
		queryFn: async () => blobToDataUrl(await client.archon.icons_v1.get(toValue(serverId), true)),
		enabled: computed(() => enabled.value && !!metadata.value && !config.value),
		staleTime: Infinity,
	})
	const image = computed(() =>
		metadata.value && !config.value ? (imageQuery.data.value ?? undefined) : undefined,
	)

	return {
		metadata,
		config,
		image,
		refetch: metadataQuery.refetch,
	}
}
