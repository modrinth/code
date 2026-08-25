<script setup lang="ts">
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
		defaultMessage: 'Override synced options for this instance.',
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

function setScreenshotsSynced(enabled: boolean) {
	const updateInstance = (current: GameInstance | undefined) =>
		current
			? {
					...current,
					synced_options: { ...current.synced_options, screenshots: enabled },
				}
			: current

	queryClient.setQueryData<GameInstance>(instanceKeys.detail(instance.value.id), updateInstance)
	queryClient.setQueryData<GameInstance[]>(instanceKeys.list(), (instances) =>
		instances?.map((candidate) =>
			candidate.id === instance.value.id ? updateInstance(candidate)! : candidate,
		),
	)
}

let saveQueue = Promise.resolve()
let latestMutationId = 0

function saveScreenshotsSynced(enabled: boolean) {
	const instanceId = instance.value.id
	const save = saveQueue.then(() => set_synced_option(instanceId, 'screenshots', enabled))
	saveQueue = save.then(
		() => undefined,
		() => undefined,
	)
	return save
}

const mutation = useMutation({
	mutationFn: saveScreenshotsSynced,
	onMutate: (enabled) => {
		const mutationId = ++latestMutationId
		setScreenshotsSynced(enabled)
		return { mutationId }
	},
	onSuccess: async (updatedInstance, _enabled, context) => {
		if (context?.mutationId !== latestMutationId) return

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
	onError: async (error, _enabled, context) => {
		handleError(error)
		if (context?.mutationId !== latestMutationId) return

		await Promise.all([
			queryClient.invalidateQueries({ queryKey: instanceKeys.detail(instance.value.id) }),
			queryClient.invalidateQueries({ queryKey: instanceKeys.list() }),
			queryClient.invalidateQueries({ queryKey: screenshotKeys.all }),
		])
	},
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
			<div class="flex shrink-0 items-center">
				<Toggle
					id="sync-screenshots"
					:model-value="instance.synced_options.screenshots"
					@update:model-value="mutation.mutate"
				/>
			</div>
		</div>
	</div>
</template>
