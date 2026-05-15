<script setup lang="ts">
import {
	injectIcarusClient,
	injectIcarusServerContext,
	ServersManageContentPage,
} from '@icarus/ui'
import { useQueryClient } from '@tanstack/vue-query'

const client = injectIcarusClient()
const { server, serverId, worldId } = injectIcarusServerContext()
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

useHead({
	title: `Content - ${server.value?.name ?? 'Server'} - Modrinth`,
})
</script>

<template>
	<ServersManageContentPage />
</template>
