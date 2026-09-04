<script setup lang="ts">
import { injectModrinthServerContext, ServersManagePlayPage } from '@modrinth/ui'

const { server } = injectModrinthServerContext()

useHead({
	title: computed(() => `Play - ${server.value?.name ?? 'Server'} - Modrinth`),
})
</script>

<template>
	<ServersManagePlayPage />
</template>
