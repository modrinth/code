<script setup lang="ts">
import { injectModrinthServerContext, ServersManageContentPage } from '@modrinth/ui'

const { server } = injectModrinthServerContext()

useHead({
	title: `Content - ${server.value?.name ?? 'Server'} - Modrinth`,
})
</script>

<template>
	<ServersManageContentPage />
</template>
