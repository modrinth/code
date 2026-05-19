<script setup lang="ts">
import { injectModrinthServerContext, ServersManageOverviewPage } from '@modrinth/ui'

const { server } = injectModrinthServerContext()
const flags = useFeatureFlags()

useHead({
	title: computed(() => `Overview - ${server.value?.name ?? 'Server'} - Modrinth`),
})
</script>

<template>
	<ServersManageOverviewPage :show-advanced-debug-info="flags.advancedDebugInfo" />
</template>
