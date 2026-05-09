<script setup lang="ts">
import {
	injectModrinthClient,
	injectModrinthServerContext,
	ServersManageAccessPage,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

const client = injectModrinthClient()
const { serverId } = injectModrinthServerContext()
const queryClient = useQueryClient()

try {
	await Promise.all([
		queryClient.ensureQueryData({
			queryKey: ['servers', 'users', 'v1', serverId],
			queryFn: () => client.archon.server_users_v1.list(serverId),
			staleTime: 30_000,
		}),
		queryClient.ensureQueryData({
			queryKey: ['servers', 'v1', 'detail', serverId],
			queryFn: () => client.archon.servers_v1.get(serverId),
			staleTime: 30_000,
		}),
	])
} catch {
	// Let mounted layouts' useQuery surface errors; do not fail route setup.
}
</script>

<template>
	<ServersManageAccessPage />
</template>
