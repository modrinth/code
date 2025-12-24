<script setup lang="ts">
import { injectModrinthServerContext, ServersManageFilesPage } from '@modrinth/ui'

const { server } = injectModrinthServerContext()
const flags = useFeatureFlags()

useHead({
	title: computed(() => `Files - ${server.value?.name ?? 'Server'} - Modrinth`),
})
</script>

<template>
	<ServersManageFilesPage :show-debug-info="flags.advancedDebugInfo" />
</template>
