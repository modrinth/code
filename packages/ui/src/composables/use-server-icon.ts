import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

import { injectModrinthClient } from '#ui/providers'

import { serverIconQueryOptions } from './server-queries'
import { useCachedIcon } from './use-cached-icon'

type UseServerIconOptions = {
	enabled?: MaybeRefOrGetter<boolean>
}

export function useServerIcon(
	serverId: MaybeRefOrGetter<string>,
	options: UseServerIconOptions = {},
) {
	const client = injectModrinthClient()
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
	}
}
