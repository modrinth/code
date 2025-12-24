<script setup lang="ts">
import { injectModrinthServerContext, ServersManageFilesPage } from '@modrinth/ui'

const { server, fsOps, fsQueuedOps } = injectModrinthServerContext()
const flags = useFeatureFlags()

useHead({
	title: computed(() => `Files - ${server.value?.name ?? 'Server'} - Modrinth`),
})
</script>

<template>
	<ServersManageFilesPage
		:fs-ops="fsOps"
		:fs-queued-ops="fsQueuedOps"
		:show-debug-info="flags.advancedDebugInfo"
	/>
</template>
