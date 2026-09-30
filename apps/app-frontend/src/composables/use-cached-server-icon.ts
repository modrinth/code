import { MinecraftServerIcon } from '@modrinth/assets'
import { useServerIcon } from '@modrinth/ui'
import { computed, type MaybeRefOrGetter } from 'vue'

export function useCachedServerIcon(serverId: MaybeRefOrGetter<string>) {
	const { icon } = useServerIcon(serverId)
	return computed(() => icon.value ?? MinecraftServerIcon)
}
