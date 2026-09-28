import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

import { injectModrinthClient } from '#ui/providers'
import { serverIconQueryOptions } from '#ui/queries/server-icon'

type UseServerImageOptions = {
	enabled?: MaybeRefOrGetter<boolean>
}

export function useServerImage(
	serverId: MaybeRefOrGetter<string>,
	options: UseServerImageOptions = {},
) {
	const client = injectModrinthClient()
	const optionsRef = computed(() => ({
		...serverIconQueryOptions(toValue(serverId), client),
		enabled: !!toValue(serverId) && (toValue(options.enabled) ?? true),
	}))
	const query = useQuery(optionsRef)

	return {
		...query,
		image: computed(() => query.data.value ?? undefined),
		queryKey: computed(() => optionsRef.value.queryKey),
	}
}
