<script setup lang="ts">
import { EditIcon } from '@modrinth/assets'
import {
	commonMessages,
	defineMessages,
	IconButton,
	injectNotificationManager,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, onScopeDispose, ref } from 'vue'

import {
	type GlobalSyncedOptions,
	isSyncedOptionAvailable,
	set_global_synced_option,
	type SyncedOption,
} from '@/helpers/instance'
import { appSettingsKeys } from '@/helpers/settings'
import {
	gameOptionsSyncSourcesQueryOptions,
	globalSyncedOptionsQueryOptions,
	initializedSyncedOptionsQueryOptions,
	syncedOptionsKeys,
	syncedServersQueryOptions,
} from '@/helpers/synced-options'
import { syncedPackKeys, syncedPackQueryOptions } from '@/helpers/synced-packs'
import { instanceKeys, instanceListQueryOptions } from '@/pages/instance/query-options'

import GameSettingsModal from '../game-settings-modal/index.vue'
import SyncedPacksModal from '../SyncedPacksModal.vue'
import SyncSourceModal from '../SyncSourceModal.vue'
import CommandHistoryModal from './command-history-modal.vue'
import LaunchOptions from './launch-options.vue'
import SyncedServersModal from './servers-modal.vue'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const queryClient = useQueryClient()
const syncedPacksModal = ref<InstanceType<typeof SyncedPacksModal>>()
const commandHistoryModal = ref<InstanceType<typeof CommandHistoryModal>>()
const syncedServersModal = ref<InstanceType<typeof SyncedServersModal>>()

const messages = defineMessages({
	resourcePacks: {
		id: 'app.settings.synced-options.resource-packs',
		defaultMessage: 'Sync resource packs',
	},
	resourcePacksDescription: {
		id: 'app.settings.synced-options.resource-packs.description',
		defaultMessage: 'Use the same resource packs across your instances',
	},
	dataPacks: { id: 'app.settings.synced-options.data-packs', defaultMessage: 'Sync data packs' },
	dataPacksDescription: {
		id: 'app.settings.synced-options.data-packs.description',
		defaultMessage: 'Use the same datapacks across your instances',
	},
	packsDisabled: {
		id: 'app.settings.synced-options.packs.disabled',
		defaultMessage: 'Enable syncing before editing shared packs.',
	},
	gameSettings: {
		id: 'app.settings.synced-options.game-settings',
		defaultMessage: 'Sync game options',
	},
	gameSettingsDescription: {
		id: 'app.settings.synced-options.game-settings.description',
		defaultMessage: 'Use the same game options across your instances',
	},
	gameSettingsButton: {
		id: 'app.settings.synced-options.game-settings.button',
		defaultMessage: 'Edit game settings',
	},
	gameSettingsDisabledTooltip: {
		id: 'app.settings.synced-options.game-settings.disabled-tooltip',
		defaultMessage: 'Enable game settings sync before choosing individual settings.',
	},
	noGameOptionsToEdit: {
		id: 'app.settings.synced-options.game-settings.no-options-to-edit',
		defaultMessage: "You haven't got any options yet to edit",
	},
	multiplayerServers: {
		id: 'app.settings.synced-options.multiplayer-servers',
		defaultMessage: 'Sync multiplayer servers',
	},
	multiplayerServersDescription: {
		id: 'app.settings.synced-options.multiplayer-servers.description',
		defaultMessage: 'Use the same multiplayer servers across your instances.',
	},
	commandHistory: {
		id: 'app.settings.synced-options.command-history',
		defaultMessage: 'Sync command history',
	},
	commandHistoryDescription: {
		id: 'app.settings.synced-options.command-history.description',
		defaultMessage: 'Use the same command history across your instances.',
	},
	creativeHotbars: {
		id: 'app.settings.synced-options.creative-hotbars',
		defaultMessage: 'Sync saved creative hotbars',
	},
	creativeHotbarsDescription: {
		id: 'app.settings.synced-options.creative-hotbars.description',
		defaultMessage: 'Use the same saved creative hotbars across your instances.',
	},
	multiplayerServersSyncSourceDescription: {
		id: 'app.settings.synced-options.choose-sync-source.multiplayer-servers-description',
		defaultMessage: 'Pick the instance whose multiplayer servers become the shared copy.',
	},
	commandHistorySyncSourceDescription: {
		id: 'app.settings.synced-options.choose-sync-source.command-history-description',
		defaultMessage: 'Pick the instance whose command history becomes the shared copy.',
	},
	creativeHotbarsSyncSourceDescription: {
		id: 'app.settings.synced-options.choose-sync-source.creative-hotbars-description',
		defaultMessage: 'Pick the instance whose saved creative hotbars become the shared copy.',
	},
	gameSettingsSyncSourceDescription: {
		id: 'app.settings.synced-options.choose-sync-source.game-settings-description',
		defaultMessage: 'Choose which instance to copy game settings from.',
	},
	resourcePacksSyncSourceDescription: {
		id: 'app.sync-instances-update.choose-source.resource-packs',
		defaultMessage: 'Choose which instance to copy resource packs from.',
	},
	dataPacksSyncSourceDescription: {
		id: 'app.sync-instances-update.choose-source.data-packs',
		defaultMessage: 'Choose which instance to copy data packs from.',
	},
	noServersSyncedYet: {
		id: 'app.settings.synced-options.multiplayer-servers.none-synced-yet',
		defaultMessage: "You haven't synced any servers yet",
	},
	noResourcePacksSyncedYet: {
		id: 'app.settings.synced-options.resource-packs.none-synced-yet',
		defaultMessage: "You haven't synced any resource packs yet",
	},
	noDataPacksSyncedYet: {
		id: 'app.settings.synced-options.data-packs.none-synced-yet',
		defaultMessage: "You haven't synced any data packs yet",
	},
	noInstancesToEdit: {
		id: 'app.settings.synced-options.edit.no-instances',
		defaultMessage: 'Add an instance before editing synced data.',
	},
	noSyncedDataToEdit: {
		id: 'app.settings.synced-options.edit.no-synced-data',
		defaultMessage: 'Choose a sync source before editing this setting.',
	},
})

const globalRows: Array<{
	option: SyncedOption
	title: keyof typeof messages
	description?: keyof typeof messages
	editable?: 'game-settings' | 'servers' | 'commands' | 'resourcepack' | 'datapack'
}> = [
	{
		option: 'game_options',
		title: 'gameSettings',
		description: 'gameSettingsDescription',
		editable: 'game-settings',
	},
	{
		option: 'multiplayer_servers',
		title: 'multiplayerServers',
		description: 'multiplayerServersDescription',
		editable: 'servers',
	},
	{
		option: 'resource_packs',
		title: 'resourcePacks',
		description: 'resourcePacksDescription',
		editable: 'resourcepack',
	},
	{
		option: 'data_packs',
		title: 'dataPacks',
		description: 'dataPacksDescription',
		editable: 'datapack',
	},
	{
		option: 'command_history',
		title: 'commandHistory',
		description: 'commandHistoryDescription',
		editable: 'commands',
	},
	{
		option: 'creative_hotbars',
		title: 'creativeHotbars',
		description: 'creativeHotbarsDescription',
	},
]

const availableGlobalRows = globalRows.filter((row) => isSyncedOptionAvailable(row.option))

const defaultGlobalOptions: GlobalSyncedOptions = {
	resource_packs: false,
	data_packs: false,
	game_options: false,
	command_history: false,
	multiplayer_servers: false,
	creative_hotbars: false,
	screenshots: false,
}

const globalOptionsQuery = useQuery(globalSyncedOptionsQueryOptions())
const initializedOptionsQuery = useQuery(initializedSyncedOptionsQueryOptions())
const gameOptionSourcesQuery = useQuery(gameOptionsSyncSourcesQueryOptions())
const resourcePacksQuery = useQuery(syncedPackQueryOptions('resourcepack'))
const dataPacksQuery = useQuery(syncedPackQueryOptions('datapack'))
const globalOptions = computed(() => globalOptionsQuery.data.value ?? defaultGlobalOptions)
const initializedOptions = computed(
	() => initializedOptionsQuery.data.value ?? defaultGlobalOptions,
)
const gameOptionSources = computed(() => gameOptionSourcesQuery.data.value ?? [])
const instancesQuery = useQuery(instanceListQueryOptions())
const instances = computed(() => instancesQuery.data.value ?? [])
const serversQuery = useQuery(syncedServersQueryOptions())
const syncedServers = computed(() => serversQuery.data.value ?? [])
const eligibleGameOptionSourceIds = computed(
	() =>
		new Set(
			gameOptionSources.value.filter((source) => source.eligible).map((source) => source.source_id),
		),
)
const hasGameOptionsToEdit = computed(
	() =>
		initializedOptions.value.game_options ||
		instances.value.some(
			(instance) =>
				instance.synced_options.game_options && eligibleGameOptionSourceIds.value.has(instance.id),
		),
)
const baseOption = ref<SyncedOption | null>(null)
const baseInstanceId = ref('')
const baseModal = ref<InstanceType<typeof SyncSourceModal> | null>(null)
const gameSettingsModal = ref<InstanceType<typeof GameSettingsModal> | null>(null)
const baseSourcesLoading = computed(() =>
	baseOption.value === null
		? false
		: baseOption.value === 'game_options'
			? gameOptionSourcesQuery.isFetching.value
			: instancesQuery.isFetching.value,
)
const baseSourcesError = computed(() =>
	baseOption.value === 'game_options'
		? gameOptionSourcesQuery.isError.value
		: instancesQuery.isError.value,
)
let baseSourceGeneration = 0

const baseInstanceDescription = computed(() => {
	switch (baseOption.value) {
		case 'game_options':
			return formatMessage(messages.gameSettingsSyncSourceDescription)
		case 'multiplayer_servers':
			return formatMessage(messages.multiplayerServersSyncSourceDescription)
		case 'command_history':
			return formatMessage(messages.commandHistorySyncSourceDescription)
		case 'creative_hotbars':
			return formatMessage(messages.creativeHotbarsSyncSourceDescription)
		case 'resource_packs':
			return formatMessage(messages.resourcePacksSyncSourceDescription)
		case 'data_packs':
			return formatMessage(messages.dataPacksSyncSourceDescription)
		default:
			return ''
	}
})

const baseInstances = computed(() =>
	baseOption.value === 'game_options'
		? gameOptionSources.value.map((source) => ({
				id: source.source_id,
				name: source.name,
				icon_path: source.icon_path,
				eligible: source.eligible,
			}))
		: instances.value.map((instance) => ({
				id: instance.id,
				name: instance.name,
				icon_path: instance.icon_path,
				eligible: !instance.quarantined,
			})),
)

async function invalidateSyncedOptions() {
	await Promise.all([
		queryClient.invalidateQueries({ queryKey: instanceKeys.all }),
		queryClient.invalidateQueries({ queryKey: ['instance-synced-options'] }),
		queryClient.invalidateQueries({ queryKey: syncedOptionsKeys.global }),
		queryClient.invalidateQueries({ queryKey: syncedOptionsKeys.initialized }),
		queryClient.invalidateQueries({ queryKey: syncedOptionsKeys.gameSources }),
		queryClient.invalidateQueries({ queryKey: ['worlds'] }),
		queryClient.invalidateQueries({ queryKey: syncedPackKeys.all }),
	])
}

function emptySyncedContentMessage(row: (typeof globalRows)[number]) {
	if (
		row.editable === 'servers' &&
		(instances.value.length === 0 || syncedServers.value.length === 0)
	) {
		return messages.noServersSyncedYet
	}
	if (row.editable === 'resourcepack' && !resourcePacksQuery.data.value?.length) {
		return messages.noResourcePacksSyncedYet
	}
	if (row.editable === 'datapack' && !dataPacksQuery.data.value?.length) {
		return messages.noDataPacksSyncedYet
	}
}

function canEditGlobalOption(row: (typeof globalRows)[number]): boolean {
	if (!row.editable || !globalOptions.value[row.option]) return false
	if (emptySyncedContentMessage(row)) return false
	if (row.editable === 'resourcepack' || row.editable === 'datapack') return true
	if (row.editable === 'game-settings') return hasGameOptionsToEdit.value
	return instances.value.length > 0 && initializedOptions.value[row.option]
}

function editGlobalOptionTooltip(row: (typeof globalRows)[number]): string {
	const emptyMessage = emptySyncedContentMessage(row)
	if (emptyMessage) return formatMessage(emptyMessage)
	if (row.editable === 'resourcepack' || row.editable === 'datapack') {
		return formatMessage(
			globalOptions.value[row.option] ? commonMessages.editButton : messages.packsDisabled,
		)
	}
	if (row.editable === 'game-settings') {
		if (!hasGameOptionsToEdit.value) return formatMessage(messages.noGameOptionsToEdit)
		if (!globalOptions.value[row.option]) {
			return formatMessage(messages.gameSettingsDisabledTooltip)
		}
		return formatMessage(messages.gameSettingsButton)
	}
	if (instances.value.length === 0) {
		return formatMessage(messages.noInstancesToEdit)
	}
	if (!initializedOptions.value[row.option]) {
		return formatMessage(messages.noSyncedDataToEdit)
	}
	return formatMessage(commonMessages.editButton)
}

type GlobalOptionMutationVariables = {
	option: SyncedOption
	enabled: boolean
	baseInstanceId?: string
}

const globalOptionMutation = useMutation({
	mutationKey: syncedOptionsKeys.set,
	mutationFn: ({ option, enabled, baseInstanceId }: GlobalOptionMutationVariables) =>
		set_global_synced_option(option, enabled, baseInstanceId),
	onMutate: async ({ option, enabled }) => {
		await queryClient.cancelQueries({ queryKey: syncedOptionsKeys.global })
		const previous = globalOptions.value[option]

		if (option !== 'game_options' || !enabled) {
			queryClient.setQueryData<GlobalSyncedOptions>(syncedOptionsKeys.global, (current) => ({
				...(current ?? defaultGlobalOptions),
				[option]: enabled,
			}))
		}

		return { previous }
	},
	onError: (error, { option }, context) => {
		queryClient.setQueryData<GlobalSyncedOptions>(syncedOptionsKeys.global, (current) => ({
			...(current ?? defaultGlobalOptions),
			[option]: context?.previous ?? defaultGlobalOptions[option],
		}))
		handleError(error)
	},
	onSuccess: async (options, { option, enabled }) => {
		queryClient.setQueryData(syncedOptionsKeys.global, options)
		if (option === 'game_options') {
			await refreshSettings()
		}
		if (enabled && option === 'multiplayer_servers') {
			await queryClient.invalidateQueries({ queryKey: syncedOptionsKeys.servers })
		}
	},
	onSettled: async () => {
		if (queryClient.isMutating({ mutationKey: syncedOptionsKeys.set }) === 1) {
			await invalidateSyncedOptions()
		}
	},
})

const canToggleGlobalOptions = computed(
	() =>
		!!globalOptionsQuery.data.value &&
		!globalOptionMutation.isPending.value &&
		!baseSourcesLoading.value,
)

async function chooseBaseInstance(option: SyncedOption) {
	const generation = ++baseSourceGeneration
	baseOption.value = option
	baseInstanceId.value = ''

	try {
		if (option === 'game_options') {
			await queryClient.fetchQuery(gameOptionsSyncSourcesQueryOptions())
		} else {
			await queryClient.fetchQuery({ ...instanceListQueryOptions(), staleTime: 0 })
		}
	} catch (error) {
		if (generation !== baseSourceGeneration) return
		handleError(error)
		baseModal.value?.show()
		return
	}

	if (generation !== baseSourceGeneration) return
	const eligibleSources = baseInstances.value.filter((source) => source.eligible)
	baseInstanceId.value = eligibleSources[0]?.id ?? ''
	if (eligibleSources.length > 1) {
		baseModal.value?.show()
		return
	}

	try {
		await globalOptionMutation.mutateAsync({
			option,
			enabled: true,
			baseInstanceId: eligibleSources[0]?.id,
		})
		await nextTick()
		if (!baseModal.value?.hide()) clearBaseSource()
	} catch {
		return
	}
}

function toggleGlobalOption(option: SyncedOption, enabled: boolean) {
	if (!isSyncedOptionAvailable(option) || !canToggleGlobalOptions.value) return
	if (enabled && option !== 'screenshots') {
		void chooseBaseInstance(option)
		return
	}
	globalOptionMutation.mutate({ option, enabled })
}

async function confirmBaseInstance() {
	if (!baseOption.value || !baseInstanceId.value) return
	try {
		await globalOptionMutation.mutateAsync({
			option: baseOption.value,
			enabled: true,
			baseInstanceId: baseInstanceId.value,
		})
		await nextTick()
		baseModal.value?.hide()
	} catch {
		return
	}
}

function editGlobalOption(row: (typeof globalRows)[number]) {
	if (!isSyncedOptionAvailable(row.option)) return
	if (row.editable === 'resourcepack' || row.editable === 'datapack') {
		void syncedPacksModal.value?.show(row.editable)
	} else if (row.editable === 'game-settings') {
		openGameSettings()
	} else if (row.editable === 'commands') {
		void commandHistoryModal.value?.show()
	} else {
		void syncedServersModal.value?.show()
	}
}

function openGameSettings() {
	if (!hasGameOptionsToEdit.value) return
	gameSettingsModal.value?.show()
}

function refreshSettings() {
	return queryClient.invalidateQueries({ queryKey: appSettingsKeys.all })
}

async function handleGameSettingsSaved() {
	await Promise.all([invalidateSyncedOptions(), refreshSettings()])
}

function clearBaseSource() {
	baseSourceGeneration++
	baseOption.value = null
}

onScopeDispose(clearBaseSource)
</script>

<template>
	<div>
		<SyncedPacksModal ref="syncedPacksModal" />
		<CommandHistoryModal ref="commandHistoryModal" />
		<SyncedServersModal ref="syncedServersModal" />
		<GameSettingsModal ref="gameSettingsModal" @saved="handleGameSettingsSaved" />

		<SyncSourceModal
			ref="baseModal"
			v-model="baseInstanceId"
			:description="baseInstanceDescription"
			:sources="baseInstances"
			:loading="baseSourcesLoading"
			:error="baseSourcesError"
			:pending="globalOptionMutation.isPending.value"
			@confirm="confirmBaseInstance"
			@close="clearBaseSource"
			@retry="baseOption && chooseBaseInstance(baseOption)"
		/>

		<section class="border-0 border-b border-solid border-surface-4 pb-6">
			<div class="flex flex-col gap-6">
				<div class="flex flex-col gap-4">
					<div
						v-for="row in availableGlobalRows"
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
							<span v-if="row.editable" v-tooltip="editGlobalOptionTooltip(row)" class="flex">
								<IconButton
									type="outlined"
									circular
									:disabled="
										!canEditGlobalOption(row) ||
										initializedOptionsQuery.isPending.value ||
										globalOptionMutation.isPending.value
									"
									:label="
										formatMessage(
											row.editable === 'game-settings'
												? messages.gameSettingsButton
												: commonMessages.editButton,
										)
									"
									@click="editGlobalOption(row)"
								>
									<EditIcon aria-hidden="true" />
								</IconButton>
							</span>
							<Toggle
								:id="`global-sync-${row.option}`"
								:model-value="globalOptions[row.option]"
								:disabled="!canToggleGlobalOptions"
								:aria-label="formatMessage(messages[row.title])"
								@update:model-value="(enabled) => toggleGlobalOption(row.option, enabled)"
							/>
						</div>
					</div>
				</div>
			</div>
		</section>

		<LaunchOptions />
	</div>
</template>
