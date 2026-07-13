<script setup lang="ts">
import { ServersManagePlayPage } from '@modrinth/ui'
import { useRoute, useRouter } from 'vue-router'

import { useTheming } from '@/store/theme'

const route = useRoute()
const router = useRouter()
const themeStore = useTheming()

if (!themeStore.getFeatureFlag('ServerPlayFrontend')) {
	await router.replace(`/hosting/manage/${String(route.params.id)}`)
}
</script>

<template>
	<ServersManagePlayPage />
</template>
