<template>
	<NuxtLayout>
		<NuxtRouteAnnouncer />
		<ClientOnly>
			<LoadingBar />
		</ClientOnly>
		<NotificationPanel />
		<I18nDebugPanel />
		<NuxtPage />
	</NuxtLayout>
</template>
<script setup lang="ts">
import { I18nDebugPanel, LoadingBar, NotificationPanel } from '@modrinth/ui'

import { setupProviders } from '~/providers/setup.ts'

const auth = await useAuth()
setupProviders(auth)
</script>
