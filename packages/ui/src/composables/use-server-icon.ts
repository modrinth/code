import { type AbstractModrinthClient, type Archon, ModrinthApiError } from '@modrinth/api-client'
import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

import { getIconSymbolAsset } from '#ui/components/base/icon-editor-modal/editor-catalog'
import type { IconConfig } from '#ui/components/base/icon-editor-modal/types'
import { injectModrinthClient } from '#ui/providers'

import { useCachedIcon } from './use-cached-icon'

export const serverIconKeys = {
	metadata: (serverId: string) => ['server-icons', serverId, 'metadata'] as const,
	image: (serverId: string, hash: string | undefined) =>
		['server-icons', serverId, 'image', hash] as const,
}

export function serverIconMetadataQueryOptions(serverId: string, client: AbstractModrinthClient) {
	return {
		queryKey: serverIconKeys.metadata(serverId),
		queryFn: async (): Promise<Archon.Icons.v1.ServerIcon | null> => {
			try {
				return await client.archon.icons_v1.getMetadata(serverId)
			} catch (error) {
				if (error instanceof ModrinthApiError && error.statusCode === 404) return null
				throw error
			}
		},
		staleTime: 60_000,
	}
}

/** Keyed by the icon hash, so a new icon is a new query and an unchanged one never refetches. */
export function serverIconImageQueryOptions(
	serverId: string,
	hash: string | undefined,
	client: AbstractModrinthClient,
) {
	return {
		queryKey: serverIconKeys.image(serverId, hash),
		queryFn: () => client.archon.icons_v1.get(serverId, true),
		staleTime: Infinity,
	}
}

/** Returns the config of an icon created with the icon editor, if its symbol is known. */
export function serverIconEditorConfig(
	icon: Archon.Icons.v1.ServerIcon | null | undefined,
): IconConfig | null {
	const value = icon?.config
	return value && getIconSymbolAsset(value.symbol) ? value : null
}

type UseServerIconOptions = {
	/** Icon metadata already loaded with the server. Skips fetching the metadata separately. */
	icon?: MaybeRefOrGetter<Archon.Icons.v1.ServerIcon | null | undefined>
	enabled?: MaybeRefOrGetter<boolean>
}

/**
 * Resolves a server's custom icon. Icons created with the icon editor are returned as their `config`
 * so they can be drawn locally at any size; other icons are fetched as an `image` URL.
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

	const metadataQuery = useQuery(
		computed(() => ({
			...serverIconMetadataQueryOptions(toValue(serverId), client),
			enabled: enabled.value && toValue(options.icon) === undefined,
		})),
	)

	const metadata = computed(() =>
		metadataQuery.data.value !== undefined ? metadataQuery.data.value : toValue(options.icon),
	)
	const config = computed(() => serverIconEditorConfig(metadata.value))

	const imageQuery = useQuery(
		computed(() => ({
			...serverIconImageQueryOptions(toValue(serverId), metadata.value?.hash, client),
			enabled: enabled.value && !!metadata.value && !config.value,
		})),
	)
	const { src: image } = useCachedIcon(() =>
		metadata.value && !config.value ? imageQuery.data.value : undefined,
	)

	return {
		metadata,
		config,
		image,
		refetch: metadataQuery.refetch,
	}
}
