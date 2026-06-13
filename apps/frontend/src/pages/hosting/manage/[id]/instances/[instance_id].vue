<script setup lang="ts">
import { injectModrinthClient, ServersManageInstanceRootLayout } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

const route = useNativeRoute()
const serverId = route.params.id as string
const client = injectModrinthClient()
const queryClient = useQueryClient()

definePageMeta({
	middleware: 'server-instance-ready',
})

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
	<ServersManageInstanceRootLayout>
		<NuxtPage :route="route" />
	</ServersManageInstanceRootLayout>
</template>
