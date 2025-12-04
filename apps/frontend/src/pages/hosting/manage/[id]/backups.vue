<script setup lang="ts">
import { injectModrinthServerContext, ServersManageBackupsPage } from '@modrinth/ui'

const { server, isServerRunning } = injectModrinthServerContext()
const flags = useFeatureFlags()

useHead({
	title: `Backups - ${server.value.name ?? 'Server'} - Modrinth`,
})
</script>

<template>
	<ServersManageBackupsPage
		:is-server-running="isServerRunning"
		:show-debug-info="flags.advancedDebugInfo"
	/>
</template>
