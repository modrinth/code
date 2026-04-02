<script setup lang="ts">
import { injectModrinthServerContext, ServersManageOverviewPage } from '@modrinth/ui'

const { server } = injectModrinthServerContext()

useHead({
	title: computed(() => `Overview - ${server.value?.name ?? 'Server'} - Modrinth`),
})
</script>

<template>
	<ServersManageOverviewPage />
</template>
