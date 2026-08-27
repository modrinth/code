<script setup lang="ts">
import {
	EditIcon,
	RefreshCwIcon,
	RotateCounterClockwiseIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	NewModal,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, inject, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import {
	get_synced_option_join_preview,
	get_synced_options_overview,
	set_synced_option,
	type SyncedOption,
	type SyncedOptionJoinResolution,
} from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import { appSettingsModalOpenSyncedOptionsKey } from '@/providers/app-settings-modal'

import { instanceKeys, screenshotKeys } from '../../query-options'
import HooksSettings from './hooks-settings.vue'
import { injectInstanceSettings } from './instance-settings-context'
import JavaSettings from './java-settings.vue'
import WindowSettings from './window-settings.vue'

const { instance, closeModal } = injectInstanceSettings()
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()
const route = useRoute()
const router = useRouter()
const openAppSettingsSyncedOptions = inject(appSettingsModalOpenSyncedOptionsKey, () => {})

const messages = defineMessages({
	sharedSettingsDescription: {
		id: 'instance.settings.tabs.synced-options.shared-settings.description',
		defaultMessage:
			'Game settings can be shared between instances. Choose what to share in app settings.',
	},
	openSyncedOptions: {
		id: 'instance.settings.tabs.synced-options.open-app-settings',
		defaultMessage: 'Open synced settings',
	},
	multiplayerServers: {
		id: 'instance.settings.tabs.synced-options.multiplayer-servers',
		defaultMessage: 'Multiplayer servers',
	},
	multiplayerServersDescription: {
		id: 'instance.settings.tabs.synced-options.multiplayer-servers.exclude-description',
		defaultMessage: 'Exclude this instance from multiplayer server syncing.',
	},
	multiplayerServersDisabled: {
		id: 'instance.settings.tabs.synced-options.multiplayer-servers.disabled-in-app',
		defaultMessage: 'Multiplayer server syncing is turned off in app settings.',
	},
	commandHistory: {
		id: 'instance.settings.tabs.synced-options.command-history',
		defaultMessage: 'Command history',
	},
	commandHistoryDescription: {
		id: 'instance.settings.tabs.synced-options.command-history.exclude-description',
		defaultMessage: 'Exclude this instance from command history syncing.',
	},
	commandHistoryDisabled: {
		id: 'instance.settings.tabs.synced-options.command-history.disabled-in-app',
		defaultMessage: 'Command history syncing is turned off in app settings.',
	},
	creativeHotbars: {
		id: 'instance.settings.tabs.synced-options.creative-hotbars',
		defaultMessage: 'Saved creative hotbars',
	},
	creativeHotbarsDescription: {
		id: 'instance.settings.tabs.synced-options.creative-hotbars.exclude-description',
		defaultMessage: 'Exclude this instance from saved creative hotbar syncing.',
	},
	creativeHotbarsDisabled: {
		id: 'instance.settings.tabs.synced-options.creative-hotbars.disabled-in-app',
		defaultMessage: 'Saved creative hotbar syncing is turned off in app settings.',
	},
	screenshots: {
		id: 'instance.settings.tabs.synced-options.screenshots',
		defaultMessage: 'Screenshots',
	},
	screenshotsDescription: {
		id: 'instance.settings.tabs.synced-options.screenshots.exclude-description',
		defaultMessage: 'Exclude this instance’s screenshots from the Screenshots page.',
	},
	screenshotsDisabled: {
		id: 'instance.settings.tabs.synced-options.screenshots.disabled-in-app',
		defaultMessage: 'Screenshots are turned off in app settings.',
	},
	hotbarConflictTitle: {
		id: 'instance.settings.tabs.synced-options.hotbars-conflict.title',
		defaultMessage: 'Choose creative hotbars',
	},
	hotbarConflictDescription: {
		id: 'instance.settings.tabs.synced-options.hotbars-conflict.description',
		defaultMessage:
			'{instance} and your synced version have different creative hotbars. Choose which one to use across your instances.',
	},
	hotbarBackupDescription: {
		id: 'instance.settings.tabs.synced-options.hotbars-conflict.backup-description',
		defaultMessage: 'The version being replaced will be backed up before anything changes.',
	},
	useSyncedHotbars: {
		id: 'instance.settings.tabs.synced-options.hotbars-conflict.use-synced',
		defaultMessage: 'Use synced',
	},
	useInstanceHotbars: {
		id: 'instance.settings.tabs.synced-options.hotbars-conflict.use-instance',
		defaultMessage: 'Overwrite others',
	},
})

const globalDisabledMessages: Record<SyncedOption, keyof typeof messages> = {
	multiplayer_servers: 'multiplayerServersDisabled',
	command_history: 'commandHistoryDisabled',
	creative_hotbars: 'creativeHotbarsDisabled',
	screenshots: 'screenshotsDisabled',
}

const rows: Array<{
	option: SyncedOption
	title: keyof typeof messages
	description?: keyof typeof messages
}> = [
	{
		option: 'multiplayer_servers',
		title: 'multiplayerServers',
		description: 'multiplayerServersDescription',
	},
	{
		option: 'command_history',
		title: 'commandHistory',
		description: 'commandHistoryDescription',
	},
	{
		option: 'creative_hotbars',
		title: 'creativeHotbars',
		description: 'creativeHotbarsDescription',
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
const hotbarResolutionModal = ref<InstanceType<typeof NewModal> | null>(null)
const previewingOption = ref<SyncedOption | null>(null)

function excluded(option: SyncedOption): boolean {
	return (
		overviewQuery.data.value?.global_options[option] === true &&
		!instance.value.synced_options[option]
	)
}

function disabledReason(option: SyncedOption): string | undefined {
	if (overviewQuery.data.value?.global_options[option] === false) {
		return formatMessage(messages[globalDisabledMessages[option]])
	}
	return capabilities.value.get(option)?.disabled_reason ?? undefined
}

function showAppSyncedOptions(): void {
	closeModal?.()
	openAppSettingsSyncedOptions()
}

const mutation = useMutation({
	mutationFn: ({
		option,
		enabled,
		resolution,
	}: {
		option: SyncedOption
		enabled: boolean
		resolution?: SyncedOptionJoinResolution
	}) => set_synced_option(instance.value.id, option, enabled, resolution),
	onSuccess: async (updatedInstance, variables) => {
		hotbarResolutionModal.value?.hide()
		queryClient.setQueryData(instanceKeys.detail(updatedInstance.id), updatedInstance)
		queryClient.setQueryData<GameInstance[]>(instanceKeys.list(), (instances) =>
			instances?.map((candidate) =>
				candidate.id === updatedInstance.id ? updatedInstance : candidate,
			),
		)
		await queryClient.invalidateQueries({
			queryKey: ['instance-synced-options', updatedInstance.id],
		})
		if (variables.option === 'multiplayer_servers') {
			await queryClient.invalidateQueries({
				queryKey: instanceKeys.worlds(updatedInstance.id),
			})
		}

		if (variables.option === 'screenshots') {
			await queryClient.invalidateQueries({ queryKey: screenshotKeys.all })
			if (updatedInstance.synced_options.screenshots && route.name === 'InstanceScreenshots') {
				await router.replace(`/instance/${encodeURIComponent(updatedInstance.id)}`)
			} else if (!updatedInstance.synced_options.screenshots && route.name === 'Screenshots') {
				await router.replace('/')
			}
		}
	},
	onError: handleError,
})

async function setExcluded(option: SyncedOption, nextExcluded: boolean) {
	const enabled = !nextExcluded
	if (!enabled || option !== 'creative_hotbars') {
		mutation.mutate({ option, enabled })
		return
	}

	previewingOption.value = option
	try {
		const preview = await get_synced_option_join_preview(instance.value.id, option)
		if (preview.action === 'requires_resolution') {
			hotbarResolutionModal.value?.show()
		} else {
			mutation.mutate({ option, enabled })
		}
	} catch (error) {
		handleError(error)
	} finally {
		previewingOption.value = null
	}
}

function resolveHotbars(resolution: SyncedOptionJoinResolution) {
	mutation.mutate({
		option: 'creative_hotbars',
		enabled: true,
		resolution,
	})
}
</script>

<template>
	<div class="flex flex-col gap-6">
		<NewModal
			ref="hotbarResolutionModal"
			:header="formatMessage(messages.hotbarConflictTitle)"
			fade="warning"
			max-width="560px"
		>
			<div class="flex flex-col gap-3 text-primary">
				<p class="m-0">
					{{
						formatMessage(messages.hotbarConflictDescription, {
							instance: instance.name,
						})
					}}
				</p>
				<p class="m-0 text-secondary">
					{{ formatMessage(messages.hotbarBackupDescription) }}
				</p>
			</div>
			<template #actions>
				<div class="flex flex-wrap justify-end gap-2">
					<Button
						type="outlined"
						:disabled="mutation.isPending.value"
						@click="hotbarResolutionModal?.hide()"
					>
						<XIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button
						type="colored"
						color="orange"
						:disabled="mutation.isPending.value"
						@click="resolveHotbars('use_instance')"
					>
						<EditIcon aria-hidden="true" />
						{{ formatMessage(messages.useInstanceHotbars) }}
					</Button>
					<Button
						type="colored"
						color="brand"
						:disabled="mutation.isPending.value"
						@click="resolveHotbars('use_synced')"
					>
						<RotateCounterClockwiseIcon aria-hidden="true" />
						{{ formatMessage(messages.useSyncedHotbars) }}
					</Button>
				</div>
			</template>
		</NewModal>

		<div class="flex items-center justify-between gap-4">
			<p class="m-0 text-secondary">
				{{ formatMessage(messages.sharedSettingsDescription) }}
			</p>
			<Button class="shrink-0" @click="showAppSyncedOptions">
				<RefreshCwIcon />
				{{ formatMessage(messages.openSyncedOptions) }}
			</Button>
		</div>

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
						v-if="
							(mutation.isPending.value && mutation.variables.value?.option === row.option) ||
							previewingOption === row.option
						"
						class="size-5 animate-spin"
					/>
					<span v-tooltip="disabledReason(row.option)" class="flex">
						<Toggle
							:id="`exclude-${row.option}`"
							:model-value="excluded(row.option)"
							:disabled="
								mutation.isPending.value ||
								previewingOption !== null ||
								overviewQuery.isPending.value ||
								!!disabledReason(row.option)
							"
							@update:model-value="(excluded) => setExcluded(row.option, excluded)"
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
