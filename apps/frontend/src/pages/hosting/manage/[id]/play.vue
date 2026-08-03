<script setup lang="ts">
import { injectModrinthServerContext, ServersManagePlayPage } from '@modrinth/ui'

const { server } = injectModrinthServerContext()
const flags = useFeatureFlags()
const route = useNativeRoute()

if (!flags.value.ServerPlayFrontend) {
	await navigateTo(`/hosting/manage/${String(route.params.id)}`, { replace: true })
}

useHead({
	title: computed(() => `Play - ${server.value?.name ?? 'Server'} - Modrinth`),
})
</script>

<template>
	<ServersManagePlayPage />
</template>
