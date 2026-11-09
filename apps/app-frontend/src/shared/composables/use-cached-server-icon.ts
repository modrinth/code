import { MinecraftServerIcon } from '@modrinth/assets'
import { useGeneratedIconSrc, useServerIcon } from '@modrinth/ui'
import { computed, type MaybeRefOrGetter } from 'vue'

export function useCachedServerIcon(serverId: MaybeRefOrGetter<string>) {
	const { config, image } = useServerIcon(serverId)
	const generatedSrc = useGeneratedIconSrc(config)
	return computed(() => generatedSrc.value ?? image.value ?? MinecraftServerIcon)
}
