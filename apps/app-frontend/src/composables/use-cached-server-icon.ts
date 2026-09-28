import { MinecraftServerIcon } from '@modrinth/assets'
import { useServerImage } from '@modrinth/ui'
import { computed, type MaybeRefOrGetter } from 'vue'

export function useCachedServerIcon(serverId: MaybeRefOrGetter<string>) {
	const { image } = useServerImage(serverId, { enabled: false })
	return computed(() => image.value ?? MinecraftServerIcon)
}
