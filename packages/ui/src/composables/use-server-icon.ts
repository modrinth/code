import { type AbstractModrinthClient, ModrinthApiError } from '@modrinth/api-client'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

import { injectModrinthClient } from '#ui/providers'

import { useCachedIcon } from './use-cached-icon'

type UseServerIconOptions = {
	enabled?: MaybeRefOrGetter<boolean>
}

export function serverIconQueryOptions(serverId: string, client: AbstractModrinthClient) {
	return {
		queryKey: ['server-icon', serverId] as const,
		staleTime: 60_000,
		queryFn: async (): Promise<Blob | null> => {
			try {
				return await client.archon.icons_v1.get(serverId)
			} catch (error) {
				if (error instanceof ModrinthApiError && error.statusCode === 404) return null
				throw error
			}
		},
	}
}

export function useServerIcon(
	serverId: MaybeRefOrGetter<string>,
	options: UseServerIconOptions = {},
) {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const optionsRef = computed(() => ({
		...serverIconQueryOptions(toValue(serverId), client),
		enabled:
			typeof window !== 'undefined' && !!toValue(serverId) && (toValue(options.enabled) ?? true),
	}))
	const query = useQuery(optionsRef)
	const icon = useCachedIcon(() => query.data.value)

	return {
		...query,
		icon,
		queryKey: computed(() => optionsRef.value.queryKey),
		fetchIcon: (id = toValue(serverId)) => queryClient.fetchQuery(serverIconQueryOptions(id, client)),
	}
}
