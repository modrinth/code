<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { defineMessages, injectNotificationManager, Toggle, useVIntl } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import {
	get_synced_options_overview,
	set_synced_option,
	type SyncedOption,
} from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

import { instanceKeys, screenshotKeys } from '../../query-options'
import HooksSettings from './hooks-settings.vue'
import { injectInstanceSettings } from './instance-settings-context'
import JavaSettings from './java-settings.vue'
import WindowSettings from './window-settings.vue'

const { instance } = injectInstanceSettings()
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()
const route = useRoute()
const router = useRouter()

const messages = defineMessages({
	description: {
		id: 'instance.settings.tabs.synced-options.description',
		defaultMessage:
			'Sync this instance’s options and config with other instances so you don’t have to set them up every time.',
	},
	multiplayerServers: {
		id: 'instance.settings.tabs.synced-options.multiplayer-servers',
		defaultMessage: 'Multiplayer servers',
	},
	commandHistory: {
		id: 'instance.settings.tabs.synced-options.command-history',
		defaultMessage: 'Command history',
	},
	creativeHotbars: {
		id: 'instance.settings.tabs.synced-options.creative-hotbars',
		defaultMessage: 'Saved creative hotbars',
	},
	screenshots: {
		id: 'instance.settings.tabs.synced-options.screenshots',
		defaultMessage: 'Screenshots',
	},
	screenshotsDescription: {
		id: 'instance.settings.tabs.synced-options.screenshots.description',
		defaultMessage: 'View this instance’s screenshots alongside all your others.',
	},
})

const rows: Array<{
	option: SyncedOption
	title: keyof typeof messages
	description?: keyof typeof messages
}> = [
	{
		option: 'multiplayer_servers',
		title: 'multiplayerServers',
	},
	{
		option: 'command_history',
		title: 'commandHistory',
	},
	{
		option: 'creative_hotbars',
		title: 'creativeHotbars',
	},
	{
		option: 'screenshots',
		title: 'screenshots',
		description: 'screenshotsDescription',
	},
]

const overviewQuery = useQuery(
	computed(() => ({
		queryKey: ['instance-synced-options', instance.value.id],
		queryFn: () => get_synced_options_overview(instance.value.id),
	})),
)

const capabilities = computed(
	() =>
		new Map(
			overviewQuery.data.value?.capabilities.map((capability) => [capability.option, capability]) ??
				[],
		),
)

function enabled(option: SyncedOption): boolean {
	return instance.value.synced_options[option]
}

function disabledReason(option: SyncedOption): string | undefined {
	return capabilities.value.get(option)?.disabled_reason ?? undefined
}

const mutation = useMutation({
	mutationFn: ({ option, enabled }: { option: SyncedOption; enabled: boolean }) =>
		set_synced_option(instance.value.id, option, enabled),
	onSuccess: async (updatedInstance, variables) => {
		queryClient.setQueryData(instanceKeys.detail(updatedInstance.id), updatedInstance)
		queryClient.setQueryData<GameInstance[]>(instanceKeys.list(), (instances) =>
			instances?.map((candidate) =>
				candidate.id === updatedInstance.id ? updatedInstance : candidate,
			),
		)
		await queryClient.invalidateQueries({
			queryKey: ['instance-synced-options', updatedInstance.id],
		})

		if (variables.option === 'screenshots') {
			await queryClient.invalidateQueries({ queryKey: screenshotKeys.all })
			if (variables.enabled && route.name === 'InstanceScreenshots') {
				await router.replace(`/instance/${encodeURIComponent(updatedInstance.id)}`)
			}
		}
	},
	onError: handleError,
})
</script>

<template>
	<div class="flex flex-col gap-6">
		<p class="m-0 text-secondary">{{ formatMessage(messages.description) }}</p>
		<div class="flex flex-col gap-4">
			<div v-for="row in rows" :key="row.option" class="flex items-center justify-between gap-6">
				<div class="flex min-w-0 flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages[row.title]) }}
					</h2>
					<p v-if="row.description" class="m-0 text-secondary">
						{{ formatMessage(messages[row.description]) }}
					</p>
				</div>
				<div class="flex shrink-0 items-center gap-2">
					<SpinnerIcon
						v-if="mutation.isPending.value && mutation.variables.value?.option === row.option"
						class="size-5 animate-spin"
					/>
					<span v-tooltip="disabledReason(row.option)" class="flex">
						<Toggle
							:id="`sync-${row.option}`"
							:model-value="enabled(row.option)"
							:disabled="
								mutation.isPending.value ||
								overviewQuery.isPending.value ||
								!!disabledReason(row.option)
							"
							@update:model-value="(next) => mutation.mutate({ option: row.option, enabled: next })"
						/>
					</span>
				</div>
			</div>
		</div>

		<hr class="m-0 h-px border-none bg-button-border" />

		<WindowSettings />

		<hr class="m-0 h-px border-none bg-button-border" />

		<JavaSettings />

		<hr class="m-0 h-px border-none bg-button-border" />

		<HooksSettings />
	</div>
</template>
