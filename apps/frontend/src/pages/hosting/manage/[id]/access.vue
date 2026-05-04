<script setup lang="ts">
import { injectModrinthServerContext, ServersManageAccessPage } from '@modrinth/ui'

const { server } = injectModrinthServerContext()

useHead({
	title: computed(() => `Access - ${server.value?.name ?? 'Server'} - Modrinth`),
})
</script>

<template>
	<ServersManageAccessPage />
</template>
