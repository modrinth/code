<script setup lang="ts">
import {
	injectModrinthClient,
	injectModrinthServerContext,
	serverBackupsQueueQueryOptions,
	ServersManageBackupsPage,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

const client = injectModrinthClient()
const { server, serverId, worldId, isServerRunning } = injectModrinthServerContext()
const queryClient = useQueryClient()
const flags = useFeatureFlags()

if (worldId.value) {
	try {
		await queryClient.ensureQueryData(
			serverBackupsQueueQueryOptions(client, serverId, worldId.value),
		)
	} catch {
		// Let mounted layouts' useQuery surface errors; do not fail route setup.
	}
}

useHead({
	title: `Backups - ${server.value?.name ?? 'Server'} - Modrinth`,
})
</script>

<template>
	<ServersManageBackupsPage
		:is-server-running="isServerRunning"
		:show-copy-id-action="flags.developerMode"
		:show-debug-info="flags.advancedDebugInfo"
	/>
</template>
