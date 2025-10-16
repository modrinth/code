<template>
	<NuxtLayout>
		<ModrinthLoadingIndicator />
		<NotificationPanel />
		<NuxtPage />
	</NuxtLayout>
</template>
<script setup lang="ts">
import { NotificationPanel, provideNotificationManager } from '@modrinth/ui'
import { provideApi } from '@modrinth/ui/src/providers/api.ts'
import { RestModrinthApi } from '@modrinth/utils'

import ModrinthLoadingIndicator from '~/components/ui/modrinth-loading-indicator.ts'

import { FrontendNotificationManager } from './providers/frontend-notifications.ts'

provideNotificationManager(new FrontendNotificationManager())

provideApi(
	new RestModrinthApi((url: string, options?: object) => {
		const match = url.match(/^\/v(\d+)\/(.+)$/)

		if (match) {
			const apiVersion = Number(match[1])
			const path = match[2]

			return useBaseFetch(path, {
				...options,
				apiVersion,
			}) as Promise<Response>
		} else {
			throw new Error('Invalid format')
		}
	}),
)
</script>
