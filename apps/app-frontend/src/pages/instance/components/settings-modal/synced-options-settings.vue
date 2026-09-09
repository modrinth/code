<script setup lang="ts">
import { EditIcon, RefreshCwIcon, RotateCounterClockwiseIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	IconButton,
	injectNotificationManager,
	NewModal,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, inject, ref } from 'vue'

import GameSettingsModal from '@/components/ui/settings/instances/game-settings-modal/index.vue'
import CommandHistoryModal from '@/components/ui/settings/instances/instances-synced-settings/command-history-modal.vue'
import SyncedServersModal from '@/components/ui/settings/instances/instances-synced-settings/servers-modal.vue'
import SyncedPacksModal from '@/components/ui/settings/instances/SyncedPacksModal.vue'
import {
	get_synced_option_join_preview,
	get_synced_options_overview,
	isSyncedOptionAvailable,
	set_synced_option,
	type SyncedOption,
	type SyncedOptionJoinResolution,
} from '@/helpers/instance'
import {
	gameOptionsSyncSourcesQueryOptions,
	initializedSyncedOptionsQueryOptions,
	syncedOptionsKeys,
	syncedServersQueryOptions,
} from '@/helpers/synced-options'
import type { GameInstance } from '@/helpers/types'
import { appSettingsModalOpenSyncedOptionsKey } from '@/providers/app-settings-modal'

import { instanceKeys, instanceListQueryOptions } from '../../query-options'
import HooksSettings from './hooks-settings.vue'
import { injectInstanceSettings } from './instance-settings-context'
import JavaSettings from './java-settings.vue'
import WindowSettings from './window-settings.vue'

const { instance, closeModal } = injectInstanceSettings()
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()
const openAppSettingsSyncedOptions = inject(appSettingsModalOpenSyncedOptionsKey, () => {})

const syncedPacksModal = ref<InstanceType<typeof SyncedPacksModal>>()
const commandHistoryModal = ref<InstanceType<typeof CommandHistoryModal>>()
const syncedServersModal = ref<InstanceType<typeof SyncedServersModal>>()

const messages = defineMessages({
	resourcePacks: {
		id: 'instance.settings.synced-options.resource-packs',
		defaultMessage: 'Unsync resource packs',
	},
	resourcePacksDescription: {
		id: 'instance.settings.synced-options.resource-packs.description',
		defaultMessage: "Keep this instance's resource packs separate from synced packs.",
	},
	resourcePacksDisabled: {
		id: 'instance.settings.synced-options.resource-packs.disabled',
		defaultMessage: 'Resource pack syncing is turned off in app settings.',
	},
	dataPacks: {
		id: 'instance.settings.synced-options.data-packs',
		defaultMessage: 'Unsync data packs',
	},
	dataPacksDescription: {
		id: 'instance.settings.synced-options.data-packs.description',
		defaultMessage: "Keep this instance's data packs separate from synced packs.",
	},
	dataPacksDisabled: {
		id: 'instance.settings.synced-options.data-packs.disabled',
		defaultMessage: 'Data pack syncing is turned off in app settings.',
	},
	editPacks: {
		id: 'instance.settings.synced-options.edit-packs',
		defaultMessage: 'Edit synced packs',
	},
	packsOverride: {
		id: 'instance.settings.synced-options.packs-override',
		defaultMessage:
			'Turn off this override to edit synced packs. Independent packs can be managed in the content tab.',
	},
	syncedDataOverride: {
		id: 'instance.settings.synced-options.data-override',
		defaultMessage: 'Turn off this override to edit synced data.',
	},
	noSyncedDataToEdit: {
		id: 'app.settings.synced-options.edit.no-synced-data',
		defaultMessage: 'Choose a sync source before editing this setting.',
	},
	noServersSyncedYet: {
		id: 'app.settings.synced-options.multiplayer-servers.none-synced-yet',
		defaultMessage: "You haven't synced any servers yet",
	},
	sharedSettingsDescription: {
		id: 'instance.settings.tabs.synced-options.shared-settings.description',
		defaultMessage: 'Enable an override to keep a synced setting separate for this instance.',
	},
	openSyncedOptions: {
		id: 'instance.settings.tabs.synced-options.open-app-settings',
		defaultMessage: 'Manage synced settings',
	},
	gameSettings: {
		id: 'instance.settings.tabs.synced-options.game-settings',
		defaultMessage: 'Unsync game settings',
	},
	gameSettingsDescription: {
		id: 'instance.settings.tabs.synced-options.game-settings.description',
		defaultMessage: "Keep this instance's options.txt separate from the synced copy.",
	},
	editGameSettings: {
		id: 'app.settings.synced-options.game-settings.button',
		defaultMessage: 'Edit game settings',
	},
	noGameOptionsToEdit: {
		id: 'app.settings.synced-options.game-settings.no-options-to-edit',
		defaultMessage: "You haven't got any options yet to edit",
	},
	gameSettingsDisabled: {
		id: 'instance.settings.tabs.synced-options.game-settings.disabled-in-app',
		defaultMessage: 'Game settings syncing is turned off in app settings.',
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
	resource_packs: 'resourcePacksDisabled',
	data_packs: 'dataPacksDisabled',
	game_options: 'gameSettingsDisabled',
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
		option: 'game_options',
		title: 'gameSettings',
		description: 'gameSettingsDescription',
	},
	{
		option: 'multiplayer_servers',
		title: 'multiplayerServers',
		description: 'multiplayerServersDescription',
	},
	{
		option: 'resource_packs',
		title: 'resourcePacks',
		description: 'resourcePacksDescription',
	},
	{
		option: 'data_packs',
		title: 'dataPacks',
		description: 'dataPacksDescription',
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

const availableRows = rows.filter((row) => isSyncedOptionAvailable(row.option))

const overviewQuery = useQuery(
	computed(() => ({
		queryKey: ['instance-synced-options', instance.value.id],
		queryFn: () => get_synced_options_overview(instance.value.id),
	})),
)
const initializedOptionsQuery = useQuery(initializedSyncedOptionsQueryOptions())
const syncedServersQuery = useQuery(syncedServersQueryOptions())
const instancesQuery = useQuery(instanceListQueryOptions())
const gameOptionSourcesQuery = useQuery(gameOptionsSyncSourcesQueryOptions())

const capabilities = computed(
	() =>
		new Map(
			overviewQuery.data.value?.capabilities.map((capability) => [capability.option, capability]) ??
				[],
		),
)
const gameSettingsModal = ref<InstanceType<typeof GameSettingsModal> | null>(null)
const hotbarResolutionModal = ref<InstanceType<typeof NewModal> | null>(null)
const previewingOption = ref<InstanceSyncedOption | null>(null)
const previewExcluded = ref<Partial<Record<InstanceSyncedOption, boolean>>>({})

const gameSettingsInstanceId = computed(() =>
	overviewQuery.data.value?.global_options.game_options && enabled('game_options')
		? undefined
		: instance.value.id,
)
const eligibleGameOptionSourceIds = computed(
	() =>
		new Set(
			(gameOptionSourcesQuery.data.value ?? [])
				.filter((source) => source.eligible)
				.map((source) => source.source_id),
		),
)
const hasSyncedGameOptionsToEdit = computed(
	() =>
		initializedOptionsQuery.data.value?.game_options === true ||
		(instancesQuery.data.value ?? []).some(
			(candidate) =>
				candidate.synced_options.game_options &&
				eligibleGameOptionSourceIds.value.has(candidate.id),
		),
)
const hasGameOptionsToEdit = computed(() =>
	gameSettingsInstanceId.value
		? eligibleGameOptionSourceIds.value.has(gameSettingsInstanceId.value)
		: hasSyncedGameOptionsToEdit.value,
)
const gameOptionsAvailabilityPending = computed(
	() =>
		gameOptionSourcesQuery.isPending.value ||
		(gameSettingsInstanceId.value === undefined &&
			(initializedOptionsQuery.isPending.value || instancesQuery.isPending.value)),
)
const gameSettingsTooltip = computed(() =>
	!gameOptionsAvailabilityPending.value && !hasGameOptionsToEdit.value
		? formatMessage(messages.noGameOptionsToEdit)
		: formatMessage(messages.editGameSettings),
)

function excluded(option: InstanceSyncedOption): boolean {
	const preview = previewExcluded.value[option]
	if (preview !== undefined) return preview

	return (
		overviewQuery.data.value?.global_options[option] === true &&
		!instance.value.synced_options[option]
	)
}

function enabled(option: InstanceSyncedOption): boolean {
	return instance.value.synced_options[option]
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

function syncedDataDisabledReason(
	option: 'multiplayer_servers' | 'command_history',
): string | undefined {
	const reason = disabledReason(option)
	if (reason) return reason
	if (!enabled(option)) return formatMessage(messages.syncedDataOverride)
	if (option === 'multiplayer_servers' && !syncedServersQuery.data.value?.length) {
		return formatMessage(messages.noServersSyncedYet)
	}
	if (!initializedOptionsQuery.data.value?.[option]) {
		return formatMessage(messages.noSyncedDataToEdit)
	}
}

function showAppSyncedOptions(): void {
	if (closeModal) {
		closeModal(openAppSettingsSyncedOptions)
	} else {
		openAppSettingsSyncedOptions()
	}
}

function openGameSettings(): void {
	if (!hasGameOptionsToEdit.value || gameOptionsAvailabilityPending.value) return
	gameSettingsModal.value?.show()
}

async function handleGameSettingsSaved(): Promise<void> {
	await Promise.all([
		queryClient.invalidateQueries({ queryKey: instanceKeys.all }),
		queryClient.invalidateQueries({ queryKey: ['instance-synced-options'] }),
		queryClient.invalidateQueries({ queryKey: syncedOptionsKeys.initialized }),
		queryClient.invalidateQueries({ queryKey: syncedOptionsKeys.gameSources }),
	])
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
		if (variables.option === 'game_options') {
			await queryClient.invalidateQueries({ queryKey: syncedOptionsKeys.gameSources })
		}
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
	if (!isSyncedOptionAvailable(option)) return
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
		<SyncedPacksModal ref="syncedPacksModal" />
		<CommandHistoryModal ref="commandHistoryModal" />
		<SyncedServersModal ref="syncedServersModal" />
		<GameSettingsModal
			ref="gameSettingsModal"
			:instance-id="gameSettingsInstanceId"
			@saved="handleGameSettingsSaved"
		/>

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
			<div
				v-for="row in availableRows"
				:key="row.option"
				class="flex items-center justify-between gap-6"
			>
				<div class="flex min-w-0 flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages[row.title]) }}
					</h2>
					<p v-if="row.description" class="m-0 text-secondary">
						{{ formatMessage(messages[row.description]) }}
					</p>
				</div>
				<div class="flex shrink-0 items-center gap-2">
					<span v-if="row.option === 'game_options'" v-tooltip="gameSettingsTooltip" class="flex">
						<IconButton
							type="outlined"
							circular
							:disabled="
								mutation.isPending.value ||
								overviewQuery.isPending.value ||
								gameOptionsAvailabilityPending ||
								!hasGameOptionsToEdit
							"
							:label="formatMessage(messages.editGameSettings)"
							@click="openGameSettings"
						>
							<EditIcon />
						</IconButton>
					</span>
					<span
						v-if="row.option === 'multiplayer_servers' || row.option === 'command_history'"
						v-tooltip="
							syncedDataDisabledReason(row.option) ?? formatMessage(commonMessages.editButton)
						"
						class="flex"
					>
						<IconButton
							type="outlined"
							circular
							:label="formatMessage(commonMessages.editButton)"
							:disabled="
								mutation.isPending.value ||
								overviewQuery.isPending.value ||
								initializedOptionsQuery.isPending.value ||
								(row.option === 'multiplayer_servers' && syncedServersQuery.isPending.value) ||
								!!syncedDataDisabledReason(row.option)
							"
							@click="
								row.option === 'multiplayer_servers'
									? syncedServersModal?.show()
									: commandHistoryModal?.show()
							"
						>
							<EditIcon aria-hidden="true" />
						</IconButton>
					</span>
					<span
						v-if="row.option === 'resource_packs' || row.option === 'data_packs'"
						v-tooltip="
							disabledReason(row.option) ??
							formatMessage(enabled(row.option) ? messages.editPacks : messages.packsOverride)
						"
						class="flex"
					>
						<IconButton
							type="outlined"
							circular
							:label="formatMessage(messages.editPacks)"
							:disabled="
								mutation.isPending.value ||
								overviewQuery.isPending.value ||
								!!disabledReason(row.option) ||
								!enabled(row.option)
							"
							@click="
								syncedPacksModal?.show(
									row.option === 'resource_packs' ? 'resourcepack' : 'datapack',
								)
							"
						>
							<EditIcon />
						</IconButton>
					</span>
					<span v-tooltip="disabledReason(row.option)" class="flex">
						<Toggle
							:id="`exclude-${row.option}`"
							:aria-label="formatMessage(messages[row.title])"
							:model-value="excluded(row.option)"
							:disabled="
								previewingOption !== null ||
								overviewQuery.isPending.value ||
								(!!disabledReason(row.option) && !enabled(row.option))
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
