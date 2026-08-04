<script setup lang="ts">
import {
	injectModrinthClient,
	injectModrinthServerContext,
	ServersManageAccessPage,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

const client = injectModrinthClient()
const { server, serverId } = injectModrinthServerContext()
const queryClient = useQueryClient()

try {
	await queryClient.ensureQueryData({
		queryKey: ['servers', 'users', 'v1', serverId],
		queryFn: () => client.archon.server_users_v1.list(serverId),
		staleTime: 30_000,
	})
} catch {
	// Let mounted layouts' useQuery surface errors; do not fail route setup.
}

useHead({
	title: computed(() => `Access - ${server.value?.name ?? 'Server'} - Modrinth`),
})
</script>

<template>
	<ServersManageAccessPage />
</template>
