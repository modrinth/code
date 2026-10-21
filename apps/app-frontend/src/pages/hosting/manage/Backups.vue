<script setup lang="ts">
import {
	injectModrinthClient,
	injectModrinthServerContext,
	serverBackupsQueueQueryOptions,
	ServersManageBackupsPage,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

const client = injectModrinthClient()
const { serverId, worldId, isServerRunning } = injectModrinthServerContext()
const queryClient = useQueryClient()

if (worldId.value) {
	try {
		await queryClient.ensureQueryData(
			serverBackupsQueueQueryOptions(client, serverId, worldId.value),
		)
	} catch {
		// Let mounted layouts' useQuery surface errors; do not fail route setup.
	}
}
</script>

<template>
	<ServersManageBackupsPage :is-server-running="isServerRunning" />
</template>
