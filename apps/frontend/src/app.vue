<template>
	<NuxtLayout>
		<NuxtRouteAnnouncer />
		<ModrinthLoadingIndicator />
		<NotificationPanel />
		<I18nDebugPanel />
		<NuxtPage />
	</NuxtLayout>
</template>
<script setup lang="ts">
import { I18nDebugPanel, NotificationPanel } from '@modrinth/ui'

import ModrinthLoadingIndicator from '~/components/ui/modrinth-loading-indicator.ts'
import { setupProviders } from '~/providers/setup.ts'

const auth = await useAuth()
setupProviders(auth)
</script>
