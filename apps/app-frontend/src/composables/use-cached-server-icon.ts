import { MinecraftServerIcon } from '@modrinth/assets'
import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

export function useCachedServerIcon(
	serverId: MaybeRefOrGetter<string>,
	upstreamProjectId: MaybeRefOrGetter<string | null | undefined>,
) {
	const { data: detailIcon } = useQuery<string | null>({
		queryKey: computed(() => [
			'servers',
			'detail',
			toValue(serverId),
			'icon',
			toValue(upstreamProjectId) ?? null,
		]),
		enabled: false,
	})
	const { data: listingIcon } = useQuery<string | null>({
		queryKey: computed(() => ['server-icon', toValue(serverId)]),
		enabled: false,
	})

	return computed(() => {
		const icon = detailIcon.value === undefined ? listingIcon.value : detailIcon.value
		return icon ?? MinecraftServerIcon
	})
}
