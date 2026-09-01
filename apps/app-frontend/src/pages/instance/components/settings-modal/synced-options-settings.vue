<script setup lang="ts">
import { EditIcon, RefreshCwIcon, RotateCounterClockwiseIcon, XIcon } from '@modrinth/assets'
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

import {
	get_synced_option_join_preview,
	get_synced_options_overview,
	set_synced_option,
	type SyncedOption,
	type SyncedOptionJoinResolution,
} from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import { appSettingsModalOpenSyncedOptionsKey } from '@/providers/app-settings-modal'

import { instanceKeys } from '../../query-options'
import HooksSettings from './hooks-settings.vue'
import { injectInstanceSettings } from './instance-settings-context'
import JavaSettings from './java-settings.vue'
import WindowSettings from './window-settings.vue'

const { instance, closeModal } = injectInstanceSettings()
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()
const openAppSettingsSyncedOptions = inject(appSettingsModalOpenSyncedOptionsKey, () => {})

const messages = defineMessages({
	sharedSettingsDescription: {
		id: 'instance.settings.tabs.synced-options.shared-settings.description',
		defaultMessage: 'Enable an override to keep a synced setting separate for this instance.',
	},
	openSyncedOptions: {
		id: 'instance.settings.tabs.synced-options.open-app-settings',
		defaultMessage: 'Manage synced settings',
	},
	multiplayerServers: {
		id: 'instance.settings.tabs.synced-options.multiplayer-servers',
		defaultMessage: 'Unsync multiplayer servers',
	},
	multiplayerServersDescription: {
		id: 'instance.settings.tabs.synced-options.multiplayer-servers.override-description',
		defaultMessage: "Keep this instance's multiplayer servers separate from synced servers.",
	},
	multiplayerServersDisabled: {
		id: 'instance.settings.tabs.synced-options.multiplayer-servers.disabled-in-app',
		defaultMessage: 'Multiplayer server syncing is turned off in app settings.',
	},
	commandHistory: {
		id: 'instance.settings.tabs.synced-options.command-history',
		defaultMessage: 'Unsync command history',
	},
	commandHistoryDescription: {
		id: 'instance.settings.tabs.synced-options.command-history.override-description',
		defaultMessage: "Keep this instance's command history separate from synced command history.",
	},
	commandHistoryDisabled: {
		id: 'instance.settings.tabs.synced-options.command-history.disabled-in-app',
		defaultMessage: 'Command history syncing is turned off in app settings.',
	},
	creativeHotbars: {
		id: 'instance.settings.tabs.synced-options.creative-hotbars',
		defaultMessage: 'Unsync saved creative hotbars',
	},
	creativeHotbarsDescription: {
		id: 'instance.settings.tabs.synced-options.creative-hotbars.override-description',
		defaultMessage: "Keep this instance's saved creative hotbars separate from synced hotbars.",
	},
	creativeHotbarsDisabled: {
		id: 'instance.settings.tabs.synced-options.creative-hotbars.disabled-in-app',
		defaultMessage: 'Saved creative hotbar syncing is turned off in app settings.',
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

type InstanceSyncedOption = Exclude<SyncedOption, 'screenshots'>

const globalDisabledMessages: Record<InstanceSyncedOption, keyof typeof messages> = {
	multiplayer_servers: 'multiplayerServersDisabled',
	command_history: 'commandHistoryDisabled',
	creative_hotbars: 'creativeHotbarsDisabled',
}

const rows: Array<{
	option: InstanceSyncedOption
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
const previewingOption = ref<InstanceSyncedOption | null>(null)
const previewExcluded = ref<Partial<Record<InstanceSyncedOption, boolean>>>({})

function excluded(option: InstanceSyncedOption): boolean {
	const preview = previewExcluded.value[option]
	if (preview !== undefined) return preview

	return (
		overviewQuery.data.value?.global_options[option] === true &&
		!instance.value.synced_options[option]
	)
}

function setPreviewExcluded(option: InstanceSyncedOption, value?: boolean) {
	if (value === undefined) {
		const { [option]: _, ...next } = previewExcluded.value
		previewExcluded.value = next
	} else {
		previewExcluded.value = { ...previewExcluded.value, [option]: value }
	}
}

function disabledReason(option: InstanceSyncedOption): string | undefined {
	if (overviewQuery.data.value?.global_options[option] === false) {
		return formatMessage(messages[globalDisabledMessages[option]])
	}
	return capabilities.value.get(option)?.disabled_reason ?? undefined
}

function showAppSyncedOptions(): void {
	closeModal?.()
	openAppSettingsSyncedOptions()
}

type SyncedOptionMutationVariables = {
	option: InstanceSyncedOption
	enabled: boolean
	resolution?: SyncedOptionJoinResolution
}

const mutationKey = ['instance-synced-options', 'set', instance.value.id] as const
const mutation = useMutation({
	mutationKey,
	mutationFn: ({ option, enabled, resolution }: SyncedOptionMutationVariables) =>
		set_synced_option(instance.value.id, option, enabled, resolution),
	onMutate: async ({ option, enabled }) => {
		const instanceId = instance.value.id
		const detailKey = instanceKeys.detail(instanceId)
		const listKey = instanceKeys.list()
		await Promise.all([
			queryClient.cancelQueries({ queryKey: detailKey }),
			queryClient.cancelQueries({ queryKey: listKey }),
		])

		const previousEnabled = instance.value.synced_options[option]
		const applyOption = (current: GameInstance): GameInstance => ({
			...current,
			synced_options: {
				...current.synced_options,
				[option]: enabled,
			},
		})

		queryClient.setQueryData<GameInstance>(detailKey, (current) =>
			applyOption(current ?? instance.value),
		)
		queryClient.setQueryData<GameInstance[]>(listKey, (instances) =>
			instances?.map((candidate) =>
				candidate.id === instanceId ? applyOption(candidate) : candidate,
			),
		)
		setPreviewExcluded(option)

		return { instanceId, previousEnabled }
	},
	onSuccess: () => {
		hotbarResolutionModal.value?.hide()
	},
	onError: (error, { option }, context) => {
		if (context) {
			const rollbackOption = (current: GameInstance): GameInstance => ({
				...current,
				synced_options: {
					...current.synced_options,
					[option]: context.previousEnabled,
				},
			})
			queryClient.setQueryData<GameInstance>(instanceKeys.detail(context.instanceId), (current) =>
				current ? rollbackOption(current) : current,
			)
			queryClient.setQueryData<GameInstance[]>(instanceKeys.list(), (instances) =>
				instances?.map((candidate) =>
					candidate.id === context.instanceId ? rollbackOption(candidate) : candidate,
				),
			)
		}
		setPreviewExcluded(option)
		handleError(error)
	},
	onSettled: async (_data, _error, variables) => {
		if (variables.option === 'multiplayer_servers') {
			await queryClient.invalidateQueries({
				queryKey: instanceKeys.worlds(instance.value.id),
			})
		}
		if (queryClient.isMutating({ mutationKey }) === 1) {
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: instanceKeys.detail(instance.value.id) }),
				queryClient.invalidateQueries({ queryKey: instanceKeys.list() }),
				queryClient.invalidateQueries({
					queryKey: ['instance-synced-options', instance.value.id],
				}),
			])
		}
	},
})

async function setExcluded(option: InstanceSyncedOption, nextExcluded: boolean) {
	const enabled = !nextExcluded
	if (!enabled || option !== 'creative_hotbars') {
		mutation.mutate({ option, enabled })
		return
	}

	setPreviewExcluded(option, nextExcluded)
	previewingOption.value = option
	try {
		const preview = await get_synced_option_join_preview(instance.value.id, option)
		if (preview.action === 'requires_resolution') {
			hotbarResolutionModal.value?.show()
		} else {
			mutation.mutate({ option, enabled })
		}
	} catch (error) {
		setPreviewExcluded(option)
		handleError(error)
	} finally {
		previewingOption.value = null
	}
}

function cancelHotbarResolution() {
	setPreviewExcluded('creative_hotbars')
	hotbarResolutionModal.value?.hide()
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
			@hide="setPreviewExcluded('creative_hotbars')"
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
						@click="cancelHotbarResolution"
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
				<div class="flex shrink-0 items-center">
					<span v-tooltip="disabledReason(row.option)" class="flex">
						<Toggle
							:id="`exclude-${row.option}`"
							:model-value="excluded(row.option)"
							:disabled="
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
