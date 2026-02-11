<script setup lang="ts">
import { injectModrinthServerContext, ServersManageContentPage } from '@modrinth/ui'

const { server } = injectModrinthServerContext()
const flags = useFeatureFlags()

useHead({
	title: `Content - ${server.value?.name ?? 'Server'} - Modrinth`,
})
</script>

<template>
	<ServersManageContentPage :use-v1-api="flags.useV1ContentTabAPI" />
</template>
