<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { defineMessages, injectNotificationManager, Toggle, useVIntl } from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { useRoute, useRouter } from 'vue-router'

import { set_synced_option } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

import { instanceKeys, screenshotKeys } from '../../query-options'
import { injectInstanceSettings } from './instance-settings-context'

const { instance } = injectInstanceSettings()
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()
const route = useRoute()
const router = useRouter()

const messages = defineMessages({
	description: {
		id: 'instance.settings.tabs.synced-options.description',
		defaultMessage: 'Choose which content from this instance appears across the app.',
	},
	screenshots: {
		id: 'instance.settings.tabs.synced-options.screenshots',
		defaultMessage: 'Screenshots',
	},
	screenshotsDescription: {
		id: 'instance.settings.tabs.synced-options.screenshots.description',
		defaultMessage:
			'Show screenshots from this instance in the global Screenshots page. Turn this off to use an instance-only Screenshots tab.',
	},
})

const mutation = useMutation({
	mutationFn: (enabled: boolean) => set_synced_option(instance.value.id, 'screenshots', enabled),
	onSuccess: async (updatedInstance) => {
		queryClient.setQueryData(instanceKeys.detail(updatedInstance.id), updatedInstance)
		queryClient.setQueryData<GameInstance[]>(instanceKeys.list(), (instances) =>
			instances?.map((candidate) =>
				candidate.id === updatedInstance.id ? updatedInstance : candidate,
			),
		)
		await queryClient.invalidateQueries({ queryKey: screenshotKeys.all })

		if (updatedInstance.synced_options.screenshots && route.name === 'InstanceScreenshots') {
			await router.replace(`/instance/${encodeURIComponent(updatedInstance.id)}`)
		}
	},
	onError: handleError,
})
</script>

<template>
	<div class="flex flex-col gap-6">
		<p class="m-0 text-secondary">{{ formatMessage(messages.description) }}</p>
		<div class="flex items-center justify-between gap-6">
			<div class="flex min-w-0 flex-col gap-1">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.screenshots) }}
				</h2>
				<p class="m-0 text-secondary">
					{{ formatMessage(messages.screenshotsDescription) }}
				</p>
			</div>
			<div class="flex shrink-0 items-center gap-2">
				<SpinnerIcon v-if="mutation.isPending.value" class="size-5 animate-spin" />
				<Toggle
					id="sync-screenshots"
					:model-value="instance.synced_options.screenshots"
					:disabled="mutation.isPending.value"
					@update:model-value="mutation.mutate"
				/>
			</div>
		</div>
	</div>
</template>
