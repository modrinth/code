<script setup lang="ts">
import { ServersManagePlayPage } from '@modrinth/ui'
import { useRoute, useRouter } from 'vue-router'

import { useAppSettings } from '@/composables/use-app-settings.ts'

const route = useRoute()
const router = useRouter()
const appSettings = useAppSettings()

if (!appSettings.getFeatureFlag('ServerPlayFrontend')) {
	await router.replace(`/hosting/manage/${String(route.params.id)}`)
}
</script>

<template>
	<ServersManagePlayPage />
</template>
