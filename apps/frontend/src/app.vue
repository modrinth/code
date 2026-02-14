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
import {
	I18nDebugPanel,
	NotificationPanel,
	provideModalBehavior,
	provideModrinthClient,
	provideNotificationManager,
	providePageContext,
	provideTags,
} from '@modrinth/ui'

import ModrinthLoadingIndicator from '~/components/ui/modrinth-loading-indicator.ts'
import { createModrinthClient } from '~/helpers/api.ts'
import { FrontendNotificationManager } from '~/providers/frontend-notifications.ts'

const auth = await useAuth()
const config = useRuntimeConfig()

provideNotificationManager(new FrontendNotificationManager())

const cosmetics = useCosmetics()

const client = createModrinthClient(auth, {
	apiBaseUrl: config.public.apiBaseUrl.replace('/v2/', '/'),
	archonBaseUrl: config.public.pyroBaseUrl.replace('/v2/', '/'),
	rateLimitKey: config.rateLimitKey,
})
provideModrinthClient(client)

const generatedState = useGeneratedState()
provideTags({
	gameVersions: computed(() => generatedState.value.gameVersions),
	loaders: computed(() => generatedState.value.loaders),
})

providePageContext({
	hierarchicalSidebarAvailable: ref(false),
	showAds: ref(false),
})
provideModalBehavior({
	noblur: computed(() => !(cosmetics.value?.advancedRendering ?? true)),
})
</script>
