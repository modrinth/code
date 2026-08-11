<template>
	<NuxtLayout>
		<NuxtRouteAnnouncer />
		<ClientOnly>
			<LoadingBar />
		</ClientOnly>
		<NotificationPanel />
		<AdsConsentNotification />
		<I18nDebugPanel />
		<NuxtPage />
	</NuxtLayout>
</template>
<script setup lang="ts">
import { I18nDebugPanel, LoadingBar, NotificationPanel } from '@modrinth/ui'

import AdsConsentNotification from '~/components/ui/AdsConsentNotification.vue'
import { setupProviders } from '~/providers/setup.ts'
import { setupUserCountryProvider } from '~/providers/setup/user-country.ts'

import { useAuth } from './composables/auth'

setupUserCountryProvider()

const auth = await useAuth()
setupProviders(auth)
</script>
