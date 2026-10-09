<script setup lang="ts">
import {
	injectModrinthClient,
	injectModrinthServerContext,
	ServersManageContentPage,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

const client = injectModrinthClient()
const { serverId, worldId } = injectModrinthServerContext()
const queryClient = useQueryClient()

if (worldId.value) {
	try {
		await queryClient.ensureQueryData({
			queryKey: ['content', 'list', 'v1', serverId],
			queryFn: () =>
				client.archon.content_v1.getAddons(serverId, worldId.value!, { from_modpack: false }),
			staleTime: 30_000,
		})
	} catch {
		// Let mounted layouts' useQuery surface errors; do not fail route setup.
	}
}
</script>

<template>
	<ServersManageContentPage />
</template>
