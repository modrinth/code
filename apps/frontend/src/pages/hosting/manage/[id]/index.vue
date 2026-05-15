<script setup lang="ts">
import { injectIcarusServerContext, ServersManageOverviewPage } from '@icarus/ui'

const { server } = injectIcarusServerContext()
const flags = useFeatureFlags()

useHead({
	title: computed(() => `Overview - ${server.value?.name ?? 'Server'} - Modrinth`),
})
</script>

<template>
	<ServersManageOverviewPage :show-advanced-debug-info="flags.advancedDebugInfo" />
</template>
