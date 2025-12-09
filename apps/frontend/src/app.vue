<template>
	<NuxtLayout>
		<ModrinthLoadingIndicator />
		<NotificationPanel />
		<NuxtPage />
	</NuxtLayout>
</template>
<script setup lang="ts">
import {
	NotificationPanel,
	provideModrinthClient,
	provideNotificationManager,
	providePageContext,
} from '@modrinth/ui'

import ModrinthLoadingIndicator from '~/components/ui/modrinth-loading-indicator.ts'
import { createModrinthClient } from '~/helpers/api.ts'
import { FrontendNotificationManager } from '~/providers/frontend-notifications.ts'

const auth = await useAuth()
const config = useRuntimeConfig()

provideNotificationManager(new FrontendNotificationManager())

const client = createModrinthClient(auth, {
	apiBaseUrl: config.public.apiBaseUrl.replace('/v2/', '/'),
	archonBaseUrl: config.public.pyroBaseUrl.replace('/v2/', '/'),
	rateLimitKey: config.rateLimitKey,
})
provideModrinthClient(client)
providePageContext({
	hierarchicalSidebarAvailable: ref(false),
	showAds: ref(false),
})
</script>
