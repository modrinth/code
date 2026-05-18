<script setup lang="ts">
import { injectModrinthClient } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

const route = useNativeRoute()
const serverId = route.params.id as string
const client = injectModrinthClient()
const queryClient = useQueryClient()

if (serverId) {
	try {
		await queryClient.ensureQueryData({
			queryKey: ['servers', 'v1', 'detail', serverId],
			queryFn: () => client.archon.servers_v1.get(serverId),
			staleTime: 30_000,
		})
	} catch {
		// Let mounted layouts' useQuery surface errors; do not fail route setup.
	}
}
</script>

<template>
	<NuxtPage :route="route" />
</template>
