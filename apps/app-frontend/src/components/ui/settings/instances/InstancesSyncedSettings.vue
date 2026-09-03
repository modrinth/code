<script setup lang="ts">
import {
	ClipboardCopyIcon,
	EditIcon,
	// FolderOpenIcon,
	MoreVerticalIcon,
	NoSignalIcon,
	RefreshCwIcon,
	SaveIcon,
	SearchIcon,
	SignalIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	type ButtonMenuOption,
	commonMessages,
	defineMessages,
	FilterPills,
	IconButton,
	injectNotificationManager,
	Input,
	NewModal,
	Slider,
	Table,
	type TableColumn,
	TeleportOverflowMenu,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import type { Component } from 'vue'
import { computed, nextTick, ref, shallowRef, watch } from 'vue'

import GameSettingsModal from '@/components/ui/settings/instances/game-settings/GameSettingsModal.vue'
import SyncedPacksModal from '@/components/ui/settings/instances/SyncedPacksModal.vue'
import SyncSourceModal from '@/components/ui/settings/instances/SyncSourceModal.vue'
import useMemorySlider from '@/composables/useMemorySlider'
import {
	get_command_history,
	type GlobalSyncedOptions,
	list as listInstances,
	list_synced_servers,
	// open_synced_options_folder,
	remove_synced_server,
	set_command_history,
	set_global_synced_option,
	type SyncedOption,
	type SyncedServer,
	update_synced_server,
} from '@/helpers/instance'
import { get, parseEnvVars, serializeEnvVars, set } from '@/helpers/settings.ts'
import {
	gameOptionsSyncSourcesQueryOptions,
	globalSyncedOptionsQueryOptions,
	initializedSyncedOptionsQueryOptions,
	syncedOptionsKeys,
} from '@/helpers/synced-options'
import { syncedPackKeys, syncedPackQueryOptions } from '@/helpers/synced-packs'
import { copyToClipboard } from '@/helpers/utils'
import {
	refreshServerData,
	refreshServers,
	type ServerData,
	type ServerWorld,
} from '@/helpers/worlds.ts'
import { instanceKeys } from '@/pages/instance/query-options'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const queryClient = useQueryClient()

const syncedPacksModal = ref<InstanceType<typeof SyncedPacksModal>>()

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
	// syncedDescription: {
	// 	id: 'app.settings.synced-options.description',
	// 	defaultMessage:
	// 		'Sync options and config across instances so you don’t have to set them up every time.',
	// },
	// syncedFolder: {
	// 	id: 'app.settings.synced-options.folder',
	// 	defaultMessage: 'Synced folder',
	// },
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
	commandHistoryEditorTitle: {
		id: 'app.settings.synced-options.command-history.editor-title',
		defaultMessage: 'Edit command history',
	},
	serverEditorTitle: {
		id: 'app.settings.synced-options.multiplayer-servers.editor-title',
		defaultMessage: 'Edit synced servers',
	},
	editServerTitle: {
		id: 'instance.edit-server.title',
		defaultMessage: 'Edit server',
	},
	serverName: {
		id: 'app.settings.synced-options.multiplayer-servers.name',
		defaultMessage: 'Server name',
	},
	serverAddress: {
		id: 'app.settings.synced-options.multiplayer-servers.address',
		defaultMessage: 'Server address',
	},
	searchServers: {
		id: 'app.settings.synced-options.multiplayer-servers.search',
		defaultMessage:
			'Search {count, number} {count, plural, one {synced server} other {synced servers}}...',
	},
	serverStatus: {
		id: 'app.settings.synced-options.multiplayer-servers.status',
		defaultMessage: 'Status',
	},
	onlineFilter: {
		id: 'app.settings.synced-options.multiplayer-servers.filter-online',
		defaultMessage: 'Online',
	},
	offlineFilter: {
		id: 'app.settings.synced-options.multiplayer-servers.filter-offline',
		defaultMessage: 'Offline',
	},
	playersOnline: {
		id: 'app.settings.synced-options.multiplayer-servers.players-online',
		defaultMessage: '{count, number} online',
	},
	noMatchingServers: {
		id: 'app.settings.synced-options.multiplayer-servers.no-results',
		defaultMessage: 'No synced servers match your search or filters.',
	},
	serverCount: {
		id: 'app.settings.synced-options.multiplayer-servers.count',
		defaultMessage: '{count, plural, one {# server} other {# servers}}',
	},
	moreServerOptions: {
		id: 'app.settings.synced-options.multiplayer-servers.more-options',
		defaultMessage: 'More server options',
	},
	copyServerAddress: {
		id: 'app.settings.synced-options.multiplayer-servers.copy-address',
		defaultMessage: 'Copy address',
	},
	noSyncedServers: {
		id: 'app.settings.synced-options.multiplayer-servers.empty',
		defaultMessage: 'No user-added servers are currently synced.',
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
	windowSectionTitle: {
		id: 'app.settings.default-instance-options.window.title',
		defaultMessage: 'Window',
	},
	javaAndMemorySectionTitle: {
		id: 'app.settings.default-instance-options.java-and-memory.title',
		defaultMessage: 'Java and memory',
	},
	launchHooksSectionTitle: {
		id: 'app.settings.default-instance-options.launch-hooks.title',
		defaultMessage: 'Launch hooks',
	},
	fullscreenTitle: {
		id: 'app.settings.default-instance-options.fullscreen.title',
		defaultMessage: 'Fullscreen',
	},
	fullscreenDescription: {
		id: 'app.settings.default-instance-options.fullscreen.description',
		defaultMessage: 'Start instances in fullscreen by updating their options.txt file.',
	},
	widthTitle: {
		id: 'app.settings.default-instance-options.width.title',
		defaultMessage: 'Width',
	},
	widthDescription: {
		id: 'app.settings.default-instance-options.width.description',
		defaultMessage: 'The width of the game window when launched.',
	},
	widthPlaceholder: {
		id: 'app.settings.default-instance-options.width.placeholder',
		defaultMessage: 'Enter width...',
	},
	heightTitle: {
		id: 'app.settings.default-instance-options.height.title',
		defaultMessage: 'Height',
	},
	heightDescription: {
		id: 'app.settings.default-instance-options.height.description',
		defaultMessage: 'The height of the game window when launched.',
	},
	heightPlaceholder: {
		id: 'app.settings.default-instance-options.height.placeholder',
		defaultMessage: 'Enter height...',
	},
	memoryAllocationTitle: {
		id: 'app.settings.default-instance-options.memory-allocation.title',
		defaultMessage: 'Memory allocation',
	},
	memoryAllocationDescription: {
		id: 'app.settings.default-instance-options.memory-allocation.description',
		defaultMessage: 'Maximum memory available to each instance.',
	},
	javaArgumentsTitle: {
		id: 'app.settings.default-instance-options.java-arguments.title',
		defaultMessage: 'Java arguments',
	},
	javaArgumentsPlaceholder: {
		id: 'app.settings.default-instance-options.java-arguments.placeholder',
		defaultMessage: 'Enter Java arguments...',
	},
	javaArgumentsDescription: {
		id: 'app.settings.default-instance-options.java-arguments.description',
		defaultMessage: 'Arguments passed to Java when launching an instance.',
	},
	environmentVariablesTitle: {
		id: 'app.settings.default-instance-options.environment-variables.title',
		defaultMessage: 'Environment variables',
	},
	environmentVariablesPlaceholder: {
		id: 'app.settings.default-instance-options.environment-variables.placeholder',
		defaultMessage: 'Enter environment variables...',
	},
	environmentVariablesDescription: {
		id: 'app.settings.default-instance-options.environment-variables.description',
		defaultMessage: 'Environment variables set when launching an instance.',
	},
	preLaunchHookTitle: {
		id: 'app.settings.default-instance-options.pre-launch-hook.title',
		defaultMessage: 'Pre-launch hook',
	},
	preLaunchHookPlaceholder: {
		id: 'app.settings.default-instance-options.pre-launch-hook.placeholder',
		defaultMessage: 'Enter pre-launch command...',
	},
	preLaunchHookDescription: {
		id: 'app.settings.default-instance-options.pre-launch-hook.description',
		defaultMessage: 'Runs before the instance starts.',
	},
	wrapperHookTitle: {
		id: 'app.settings.default-instance-options.wrapper-hook.title',
		defaultMessage: 'Wrapper hook',
	},
	wrapperHookPlaceholder: {
		id: 'app.settings.default-instance-options.wrapper-hook.placeholder',
		defaultMessage: 'Enter wrapper command...',
	},
	wrapperHookDescription: {
		id: 'app.settings.default-instance-options.wrapper-hook.description',
		defaultMessage: 'Command used to wrap the Minecraft launch process.',
	},
	postExitHookTitle: {
		id: 'app.settings.default-instance-options.post-exit-hook.title',
		defaultMessage: 'Post-exit hook',
	},
	postExitHookPlaceholder: {
		id: 'app.settings.default-instance-options.post-exit-hook.placeholder',
		defaultMessage: 'Enter post-exit command...',
	},
	postExitHookDescription: {
		id: 'app.settings.default-instance-options.post-exit-hook.description',
		defaultMessage: 'Runs after the game closes.',
	},
	hookVariablesDescription: {
		id: 'instance.settings.tabs.hooks.variables.description',
		defaultMessage:
			'Hooks run in the working directory of the instance, with the following variables:',
	},
	instanceNameDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-name.description',
		defaultMessage: '$INST_NAME: The name of the instance',
	},
	instanceIdDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-id.description',
		defaultMessage: "$INST_ID: The name of the instance's folder",
	},
	instanceDirDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-dir.description',
		defaultMessage: "$INST_DIR: The absolute path to the instance's folder",
	},
	instanceMcDirDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-mc-dir.description',
		defaultMessage: '$INST_MC_DIR: An alias for $INST_DIR',
	},
	instanceJavaDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-java.description',
		defaultMessage: '$INST_JAVA: The absolute path to the java binary',
	},
	instanceJavaArgsDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-java-args.description',
		defaultMessage: '$INST_JAVA_ARGS: The JVM Arguments provided to the game',
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

const globalSyncedOptionsQueryKey = syncedOptionsKeys.global
const initializedSyncedOptionsQueryKey = syncedOptionsKeys.initialized
const gameOptionsSyncSourcesQueryKey = syncedOptionsKeys.gameSources
const globalSyncedOptionsMutationKey = syncedOptionsKeys.set
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
const instances = ref(await listInstances().catch(() => []))
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
const baseInstanceId = ref(instances.value[0]?.id ?? '')
const baseModal = ref<InstanceType<typeof SyncSourceModal> | null>(null)
const gameSettingsModal = ref<InstanceType<typeof GameSettingsModal> | null>(null)
const commandHistoryModal = ref<InstanceType<typeof NewModal> | null>(null)
const serverEditorModal = ref<InstanceType<typeof NewModal> | null>(null)
const editServerModal = ref<InstanceType<typeof NewModal> | null>(null)
const commandHistory = ref('')
const syncedServers = ref<SyncedServer[]>(
	(await list_synced_servers().catch((error) => {
		handleError(error)
		return []
	})) ?? [],
)
const editedServer = ref<SyncedServer | null>(null)
const serverData = ref<Record<string, ServerData>>({})
const serverSearch = ref('')
const selectedServerFilters = ref<string[]>([])
const refreshingAllSyncedServers = ref(false)
const editorComponent = shallowRef<Component | null>(null)
const baseSourcesLoading = ref(false)
let baseSourceGeneration = 0

type BaseSourceCandidate = {
	id: string
	name: string
	icon_path?: string | null
	eligible: boolean
}

const syncedServerCards = computed(() =>
	syncedServers.value.map((server, index) => ({
		server,
		world: {
			name: server.name,
			type: 'server',
			index,
			server_id: server.id,
			address: server.address,
			pack_status:
				server.accept_textures === true
					? 'enabled'
					: server.accept_textures === false
						? 'disabled'
						: 'prompt',
			display_status: 'normal',
		} satisfies ServerWorld,
	})),
)

type SyncedServerTableColumn = 'server' | 'status' | 'version' | 'actions'
type SyncedServerTableRow = SyncedServer &
	Record<SyncedServerTableColumn, unknown> & {
		server: string
		status: 'online' | 'offline'
		version: string
	}

const syncedServerColumns = computed<TableColumn<SyncedServerTableColumn>[]>(() => [
	{
		key: 'server',
		label: formatMessage(commonMessages.serverLabel),
		width: '45%',
		cellClass: '!h-16',
	},
	{
		key: 'status',
		label: formatMessage(messages.serverStatus),
		width: '25%',
		cellClass: '!h-16',
	},
	{
		key: 'version',
		label: formatMessage(commonMessages.versionLabel),
		width: '22%',
		cellClass: '!h-16',
	},
	{
		key: 'actions',
		label: formatMessage(commonMessages.actionsLabel),
		align: 'right',
		width: '6rem',
		cellClass: '!h-16',
	},
])

const syncedServerFilterOptions = computed(() => [
	{ id: 'online', label: formatMessage(messages.onlineFilter) },
	{ id: 'offline', label: formatMessage(messages.offlineFilter) },
])

const syncedServerRows = computed<SyncedServerTableRow[]>(() =>
	syncedServers.value.map((server) => ({
		...server,
		server: server.name,
		status: serverData.value[server.address]?.status ? 'online' : 'offline',
		version: serverData.value[server.address]?.status?.version?.name ?? '',
		actions: null,
	})),
)

const statusFilteredSyncedServerRows = computed(() => {
	if (selectedServerFilters.value.length === 0) return syncedServerRows.value
	return syncedServerRows.value.filter((server) =>
		selectedServerFilters.value.includes(server.status),
	)
})

const filteredSyncedServerRows = computed(() => {
	const search = serverSearch.value.trim().toLocaleLowerCase()
	if (!search) return statusFilteredSyncedServerRows.value
	return statusFilteredSyncedServerRows.value.filter(
		(server) =>
			server.name.toLocaleLowerCase().includes(search) ||
			server.address.toLocaleLowerCase().includes(search),
	)
})

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
		default:
			return ''
	}
})

const baseInstances = computed<BaseSourceCandidate[]>(() =>
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
				eligible: true,
			})),
)

async function invalidateSyncedOptions() {
	await Promise.all([
		queryClient.invalidateQueries({ queryKey: instanceKeys.all }),
		queryClient.invalidateQueries({ queryKey: ['instance-synced-options'] }),
		queryClient.invalidateQueries({ queryKey: globalSyncedOptionsQueryKey }),
		queryClient.invalidateQueries({ queryKey: initializedSyncedOptionsQueryKey }),
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
	return (
		instances.value.length > 0 &&
		initializedOptions.value[row.option] &&
		(row.editable !== 'servers' || syncedServers.value.length > 0)
	)
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
	mutationKey: globalSyncedOptionsMutationKey,
	mutationFn: ({ option, enabled, baseInstanceId }: GlobalOptionMutationVariables) =>
		set_global_synced_option(option, enabled, baseInstanceId),
	onMutate: async ({ option, enabled }) => {
		await queryClient.cancelQueries({ queryKey: globalSyncedOptionsQueryKey })
		const previous = globalOptions.value[option]

		if (option !== 'game_options' || !enabled) {
			queryClient.setQueryData<GlobalSyncedOptions>(globalSyncedOptionsQueryKey, (current) => ({
				...(current ?? defaultGlobalOptions),
				[option]: enabled,
			}))
		}

		return { previous }
	},
	onError: (error, { option }, context) => {
		queryClient.setQueryData<GlobalSyncedOptions>(globalSyncedOptionsQueryKey, (current) => ({
			...(current ?? defaultGlobalOptions),
			[option]: context?.previous ?? defaultGlobalOptions[option],
		}))
		handleError(error)
	},
	onSuccess: async (options, { option, enabled }) => {
		queryClient.setQueryData(globalSyncedOptionsQueryKey, options)
		if (option === 'game_options') {
			await refreshSettings()
		}
		if (enabled && option === 'multiplayer_servers') {
			syncedServers.value = await list_synced_servers().catch((error) => {
				handleError(error)
				return []
			})
		}
	},
	onSettled: async () => {
		if (queryClient.isMutating({ mutationKey: globalSyncedOptionsMutationKey }) === 1) {
			await invalidateSyncedOptions()
		}
	},
})

function applyGlobalOption(option: SyncedOption, enabled: boolean, baseInstanceId?: string) {
	globalOptionMutation.mutate({ option, enabled, baseInstanceId })
}

async function chooseBaseInstance(option: SyncedOption) {
	const generation = ++baseSourceGeneration
	baseOption.value = option
	baseSourcesLoading.value = option === 'game_options'

	if (option === 'game_options') {
		try {
			const sources = await queryClient.fetchQuery(gameOptionsSyncSourcesQueryOptions())
			if (generation !== baseSourceGeneration || baseOption.value !== option) return
			const eligibleSources = sources.filter((source) => source.eligible)
			baseInstanceId.value = eligibleSources[0]?.source_id ?? ''
			if (eligibleSources.length === 1) {
				applyGlobalOption(option, true, eligibleSources[0].source_id)
				return
			}
		} catch (error) {
			if (generation !== baseSourceGeneration || baseOption.value !== option) return
			queryClient.setQueryData(gameOptionsSyncSourcesQueryKey, [])
			baseInstanceId.value = ''
			handleError(error)
			return
		} finally {
			if (generation === baseSourceGeneration && baseOption.value === option) {
				baseSourcesLoading.value = false
			}
		}
		if (!baseInstanceId.value) {
			applyGlobalOption(option, true)
			return
		}
		baseModal.value?.show()
		return
	}

	if (instances.value.length === 0) {
		applyGlobalOption(option, true)
		return
	}
	baseInstanceId.value = instances.value[0]?.id ?? ''
	if (instances.value.length === 1) {
		applyGlobalOption(option, true, baseInstanceId.value)
		return
	}
	baseModal.value?.show()
}

function toggleGlobalOption(option: SyncedOption, enabled: boolean) {
	if (
		enabled &&
		option !== 'screenshots' &&
		option !== 'resource_packs' &&
		option !== 'data_packs'
	) {
		void chooseBaseInstance(option)
		return
	}
	applyGlobalOption(option, enabled)
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
	if (row.editable === 'resourcepack' || row.editable === 'datapack') {
		void syncedPacksModal.value?.show(row.editable)
	} else if (row.editable === 'commands') {
		void openCommandHistoryEditor()
	} else {
		void openServerEditor()
	}
}

function openGameSettings() {
	if (!hasGameOptionsToEdit.value) return
	gameSettingsModal.value?.show()
}

async function openCommandHistoryEditor() {
	commandHistory.value = await get_command_history().catch((error) => {
		handleError(error)
		return ''
	})
	if (!editorComponent.value) {
		const [editor] = await Promise.all([
			import('vue3-ace-editor'),
			import('@modrinth/ui/src/utils/ace-theme'),
			import('@modrinth/ui/src/utils/ace-mode-mcfunction'),
		])
		editorComponent.value = editor.VAceEditor
	}
	commandHistoryModal.value?.show()
}

async function saveCommandHistory() {
	try {
		commandHistory.value = await set_command_history(commandHistory.value)
		commandHistoryModal.value?.hide()
	} catch (error) {
		handleError(error)
	}
}

async function openServerEditor() {
	syncedServers.value = await list_synced_servers().catch((error) => {
		handleError(error)
		return []
	})
	serverSearch.value = ''
	selectedServerFilters.value = []
	serverData.value = {}
	serverEditorModal.value?.show()
	await refreshAllSyncedServers()
}

function openSyncedServerEditor(server: SyncedServer) {
	editedServer.value = { ...server }
	editServerModal.value?.show()
}

async function saveSyncedServer() {
	if (!editedServer.value) return
	const server = editedServer.value

	try {
		await update_synced_server(server)
		const index = syncedServers.value.findIndex(({ id }) => id === server.id)
		if (index !== -1) {
			syncedServers.value[index] = { ...server }
		}
		editServerModal.value?.hide()
		serverData.value[server.address] = { refreshing: true }
		await refreshServerData(serverData.value[server.address], null, server.address)
		await queryClient.invalidateQueries({ queryKey: ['worlds'] })
	} catch (error) {
		handleError(error)
	}
}

async function refreshSyncedServer(address: string) {
	serverData.value[address] ??= { refreshing: true }
	await refreshServerData(serverData.value[address], null, address)
}

async function refreshAllSyncedServers() {
	if (refreshingAllSyncedServers.value) return
	refreshingAllSyncedServers.value = true
	try {
		await refreshServers(
			syncedServerCards.value.map(({ world }) => world),
			serverData.value,
			null,
		)
	} finally {
		refreshingAllSyncedServers.value = false
	}
}

function syncedServerMenuOptions(server: SyncedServer): ButtonMenuOption[] {
	return [
		{
			id: 'refresh',
			label: formatMessage(commonMessages.refreshButton),
			icon: RefreshCwIcon,
			action: () => void refreshSyncedServer(server.address),
		},
		{
			id: 'copy-address',
			label: formatMessage(messages.copyServerAddress),
			icon: ClipboardCopyIcon,
			action: () => copyToClipboard(server.address),
		},
		{
			id: 'edit',
			label: formatMessage(commonMessages.editButton),
			icon: EditIcon,
			action: () => openSyncedServerEditor(server),
		},
		{ type: 'divider' },
		{
			id: 'remove',
			label: formatMessage(commonMessages.removeButton),
			icon: TrashIcon,
			tone: 'red',
			action: () => void removeSyncedServer(server.id),
		},
	]
}

async function removeSyncedServer(serverId: string) {
	try {
		await remove_synced_server(serverId)
		syncedServers.value = syncedServers.value.filter((server) => server.id !== serverId)
		await queryClient.invalidateQueries({ queryKey: ['worlds'] })
	} catch (error) {
		handleError(error)
	}
}

function addEditableSettingsFields(fetchSettings: Awaited<ReturnType<typeof get>>) {
	return Object.assign(fetchSettings, {
		launchArgs: fetchSettings.extra_launch_args.join(' '),
		envVars: serializeEnvVars(fetchSettings.custom_env_vars),
	})
}

const settings = ref(addEditableSettingsFields(await get()))
let applyingFetchedSettings = false

async function refreshSettings() {
	try {
		const refreshedSettings = addEditableSettingsFields(await get())
		applyingFetchedSettings = true
		settings.value = refreshedSettings
		await nextTick()
	} catch (error) {
		handleError(error)
	} finally {
		applyingFetchedSettings = false
	}
}

async function handleGameSettingsSaved() {
	await Promise.all([invalidateSyncedOptions(), refreshSettings()])
}

const { maxMemory, snapPoints } = (await useMemorySlider().catch(handleError)) as unknown as {
	maxMemory: number
	snapPoints: number[]
}

watch(
	settings,
	async () => {
		if (applyingFetchedSettings) return
		const setSettings = JSON.parse(JSON.stringify(settings.value))

		setSettings.extra_launch_args = setSettings.launchArgs.trim().split(/\s+/).filter(Boolean)
		setSettings.custom_env_vars = parseEnvVars(setSettings.envVars)
		delete setSettings.launchArgs
		delete setSettings.envVars

		if (!setSettings.custom_dir) {
			setSettings.custom_dir = null
		}

		await set(setSettings).catch(handleError)
	},
	{ deep: true },
)
</script>

<template>
	<SyncedPacksModal ref="syncedPacksModal" />
	<div>
		<GameSettingsModal ref="gameSettingsModal" @saved="handleGameSettingsSaved" />

		<SyncSourceModal
			ref="baseModal"
			v-model="baseInstanceId"
			:description="baseInstanceDescription"
			:sources="baseInstances"
			:loading="baseSourcesLoading"
			:pending="globalOptionMutation.isPending.value"
			@confirm="confirmBaseInstance"
		/>

		<NewModal
			ref="commandHistoryModal"
			:header="formatMessage(messages.commandHistoryEditorTitle)"
			no-padding
			actions-divider
			max-width="700px"
			width="700px"
		>
			<component
				:is="editorComponent"
				v-if="editorComponent"
				v-model:value="commandHistory"
				lang="mcfunction"
				theme="modrinth"
				:print-margin="false"
				class="command-history-editor ace-modrinth"
				style="height: 420px; font-size: 0.875rem"
			/>
			<template #actions>
				<div class="flex justify-end gap-2">
					<Button type="outlined" @click="commandHistoryModal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button type="colored" color="brand" @click="saveCommandHistory">
						<SaveIcon />
						{{ formatMessage(commonMessages.saveButton) }}
					</Button>
				</div>
			</template>
		</NewModal>

		<NewModal
			ref="serverEditorModal"
			no-padding
			:max-width="'min(928px, calc(95vw - 10rem))'"
			:width="'min(928px, calc(95vw - 10rem))'"
		>
			<template #title>
				<span class="text-lg font-extrabold text-contrast">
					{{ formatMessage(messages.serverEditorTitle) }}
				</span>
			</template>
			<div class="flex h-[min(600px,calc(95vh-10rem))] flex-col">
				<div
					class="flex shrink-0 flex-col gap-4 border-0 border-b border-solid border-surface-4 px-6 py-4"
				>
					<Input
						v-model="serverSearch"
						:icon="SearchIcon"
						type="search"
						autocomplete="off"
						:spellcheck="false"
						:placeholder="formatMessage(messages.searchServers, { count: syncedServers.length })"
						input-class="!h-10"
						clearable
					/>

					<div class="flex flex-wrap items-center justify-between gap-3">
						<FilterPills v-model="selectedServerFilters" :options="syncedServerFilterOptions">
							<template #all>
								{{ formatMessage(commonMessages.allProjectType) }}
							</template>
						</FilterPills>
						<Button
							type="quiet"
							:disabled="refreshingAllSyncedServers || syncedServers.length === 0"
							class="hover:!bg-transparent focus-visible:!bg-transparent"
							@click="refreshAllSyncedServers"
						>
							<RefreshCwIcon :class="{ 'animate-spin': refreshingAllSyncedServers }" />
							{{ formatMessage(commonMessages.refreshButton) }}
						</Button>
					</div>
				</div>

				<div class="flex min-h-0 flex-1 flex-col overflow-hidden">
					<div
						v-if="syncedServers.length === 0"
						class="flex flex-1 items-center justify-center p-8 text-center text-secondary"
					>
						{{ formatMessage(messages.noSyncedServers) }}
					</div>
					<div
						v-else-if="filteredSyncedServerRows.length === 0"
						class="flex flex-1 items-center justify-center p-8 text-center text-secondary"
					>
						{{ formatMessage(messages.noMatchingServers) }}
					</div>
					<div v-else class="min-h-0 flex-1 overflow-y-auto">
						<Table
							:columns="syncedServerColumns"
							:data="filteredSyncedServerRows"
							row-key="id"
							table-min-width="44rem"
							class="!rounded-none !border-0"
						>
							<template #cell-server="{ row }">
								<div class="flex min-w-0 items-center gap-3">
									<Avatar
										:src="serverData[row.address]?.status?.favicon"
										:alt="row.name"
										:tint-by="row.address"
										size="32px"
										no-shadow
										class="shrink-0 !rounded-lg"
									/>
									<div class="flex min-w-0 flex-col">
										<span class="truncate font-semibold text-contrast">{{ row.name }}</span>
										<span class="truncate text-sm text-secondary">{{ row.address }}</span>
									</div>
								</div>
							</template>
							<template #cell-status="{ row }">
								<span class="inline-flex items-center gap-1.5 font-medium">
									<RefreshCwIcon
										v-if="serverData[row.address]?.refreshing"
										class="size-4 shrink-0 animate-spin"
										aria-hidden="true"
									/>
									<SignalIcon
										v-else-if="serverData[row.address]?.status"
										class="size-4 shrink-0 text-green"
										aria-hidden="true"
									/>
									<NoSignalIcon v-else class="size-4 shrink-0" aria-hidden="true" />
									<template v-if="serverData[row.address]?.refreshing">
										{{ formatMessage(commonMessages.loadingLabel) }}
									</template>
									<template v-else-if="serverData[row.address]?.status">
										{{
											formatMessage(messages.playersOnline, {
												count: serverData[row.address]?.status?.players?.online ?? 0,
											})
										}}
									</template>
									<template v-else>{{ formatMessage(messages.offlineFilter) }}</template>
								</span>
							</template>
							<template #cell-version="{ row }">
								<span class="block truncate">{{ row.version || '—' }}</span>
							</template>
							<template #cell-actions="{ row }">
								<div class="flex justify-end">
									<TeleportOverflowMenu
										type="quiet"
										:label="formatMessage(messages.moreServerOptions)"
										:options="syncedServerMenuOptions(row)"
									>
										<MoreVerticalIcon aria-hidden="true" />
									</TeleportOverflowMenu>
								</div>
							</template>
						</Table>
					</div>
				</div>

				<div
					class="flex shrink-0 items-center border-0 border-t border-solid border-surface-4 px-6 py-4"
				>
					<span class="font-medium text-primary">
						{{ formatMessage(messages.serverCount, { count: syncedServers.length }) }}
					</span>
				</div>
			</div>
		</NewModal>

		<NewModal
			ref="editServerModal"
			:header="formatMessage(messages.editServerTitle)"
			max-width="500px"
			width="500px"
		>
			<div v-if="editedServer" class="flex flex-col gap-4">
				<label class="flex flex-col gap-2 font-semibold text-contrast">
					{{ formatMessage(messages.serverName) }}
					<Input v-model="editedServer.name" autocomplete="off" wrapper-class="w-full" />
				</label>
				<label class="flex flex-col gap-2 font-semibold text-contrast">
					{{ formatMessage(messages.serverAddress) }}
					<Input v-model="editedServer.address" autocomplete="off" wrapper-class="w-full" />
				</label>
			</div>
			<template #actions>
				<div class="flex justify-end gap-2">
					<Button type="outlined" @click="editServerModal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button
						type="colored"
						color="brand"
						:disabled="!editedServer?.address"
						@click="saveSyncedServer"
					>
						<SaveIcon />
						{{ formatMessage(commonMessages.saveChangesButton) }}
					</Button>
				</div>
			</template>
		</NewModal>

		<section class="border-0 border-b border-solid border-divider pb-6">
			<div class="flex flex-col gap-6">
				<!--
				<div class="flex items-center justify-between gap-4">
					<p class="m-0 text-secondary">{{ formatMessage(messages.syncedDescription) }}</p>
					<Button @click="open_synced_options_folder().catch(handleError)">
						<FolderOpenIcon />
						{{ formatMessage(messages.syncedFolder) }}
					</Button>
				</div>
				-->

				<div class="flex flex-col gap-4">
					<div
						v-for="row in globalRows"
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
							<span
								v-if="row.editable === 'game-settings'"
								v-tooltip="editGlobalOptionTooltip(row)"
								class="flex"
							>
								<IconButton
									type="outlined"
									circular
									:disabled="
										!canEditGlobalOption(row) ||
										initializedOptionsQuery.isPending.value ||
										globalOptionMutation.isPending.value
									"
									:label="formatMessage(messages.gameSettingsButton)"
									@click="openGameSettings"
								>
									<EditIcon />
								</IconButton>
							</span>
							<span v-else-if="row.editable" v-tooltip="editGlobalOptionTooltip(row)" class="flex">
								<IconButton
									type="outlined"
									circular
									:disabled="
										!canEditGlobalOption(row) ||
										initializedOptionsQuery.isPending.value ||
										globalOptionMutation.isPending.value
									"
									:label="formatMessage(commonMessages.editButton)"
									@click="editGlobalOption(row)"
								>
									<EditIcon />
								</IconButton>
							</span>
							<Toggle
								:id="`global-sync-${row.option}`"
								:model-value="globalOptions[row.option]"
								:disabled="globalOptionMutation.isPending.value"
								@update:model-value="(enabled) => toggleGlobalOption(row.option, enabled)"
							/>
						</div>
					</div>
				</div>
			</div>
		</section>

		<section class="mt-6">
			<h2 class="m-0 text-xl font-semibold text-contrast">
				{{ formatMessage(messages.windowSectionTitle) }}
			</h2>
			<div class="mt-4 flex flex-col gap-6">
				<div class="flex items-center justify-between gap-4">
					<div class="flex flex-col gap-1">
						<h3 class="m-0 text-lg font-semibold text-contrast">
							{{ formatMessage(messages.fullscreenTitle) }}
						</h3>
						<p class="m-0 leading-tight">
							{{ formatMessage(messages.fullscreenDescription) }}
						</p>
					</div>

					<Toggle id="fullscreen" v-model="settings.force_fullscreen" />
				</div>

				<div class="flex items-center justify-between gap-4">
					<div class="flex flex-col gap-1">
						<h3 class="m-0 text-lg font-semibold text-contrast">
							{{ formatMessage(messages.widthTitle) }}
						</h3>
						<p class="m-0 leading-tight">
							{{ formatMessage(messages.widthDescription) }}
						</p>
					</div>

					<Input
						id="width"
						v-model="settings.game_resolution[0]"
						:disabled="settings.force_fullscreen"
						autocomplete="off"
						type="number"
						:placeholder="formatMessage(messages.widthPlaceholder)"
					/>
				</div>

				<div class="flex items-center justify-between gap-4">
					<div class="flex flex-col gap-1">
						<h3 class="m-0 text-lg font-semibold text-contrast">
							{{ formatMessage(messages.heightTitle) }}
						</h3>
						<p class="m-0 leading-tight">
							{{ formatMessage(messages.heightDescription) }}
						</p>
					</div>

					<Input
						id="height"
						v-model="settings.game_resolution[1]"
						:disabled="settings.force_fullscreen"
						autocomplete="off"
						type="number"
						:placeholder="formatMessage(messages.heightPlaceholder)"
					/>
				</div>
			</div>
		</section>

		<section class="mt-8 border-0 border-t border-solid border-divider pt-6">
			<h2 class="m-0 text-xl font-semibold text-contrast">
				{{ formatMessage(messages.javaAndMemorySectionTitle) }}
			</h2>
			<div class="mt-4 flex flex-col gap-6">
				<div class="flex flex-col gap-2.5">
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.memoryAllocationTitle) }}
					</h3>
					<Slider
						id="max-memory"
						v-model="settings.memory.maximum"
						:min="512"
						:max="maxMemory"
						:step="64"
						:snap-points="snapPoints"
						:snap-range="512"
						unit="MB"
					/>
					<p class="m-0 mt-1 leading-tight">
						{{ formatMessage(messages.memoryAllocationDescription) }}
					</p>
				</div>

				<div class="flex flex-col gap-2.5">
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.javaArgumentsTitle) }}
					</h3>
					<Input
						id="java-args"
						v-model="settings.launchArgs"
						autocomplete="off"
						type="text"
						:placeholder="formatMessage(messages.javaArgumentsPlaceholder)"
						wrapper-class="w-full"
					/>
					<p class="m-0 leading-tight">
						{{ formatMessage(messages.javaArgumentsDescription) }}
					</p>
				</div>

				<div class="flex flex-col gap-2.5">
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.environmentVariablesTitle) }}
					</h3>
					<Input
						id="env-vars"
						v-model="settings.envVars"
						autocomplete="off"
						type="text"
						:placeholder="formatMessage(messages.environmentVariablesPlaceholder)"
						wrapper-class="w-full"
					/>
					<p class="m-0 leading-tight">
						{{ formatMessage(messages.environmentVariablesDescription) }}
					</p>
				</div>
			</div>
		</section>

		<section class="mt-8 border-0 border-t border-solid border-divider pt-6">
			<h2 class="m-0 text-xl font-semibold text-contrast">
				{{ formatMessage(messages.launchHooksSectionTitle) }}
			</h2>
			<div class="mt-4 flex flex-col gap-6">
				<div class="flex flex-col gap-2.5">
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.preLaunchHookTitle) }}
					</h3>
					<Input
						id="pre-launch"
						v-model="settings.hooks.pre_launch"
						autocomplete="off"
						type="text"
						:placeholder="formatMessage(messages.preLaunchHookPlaceholder)"
						wrapper-class="w-full"
					/>
					<p class="m-0 leading-tight">
						{{ formatMessage(messages.preLaunchHookDescription) }}
					</p>
				</div>

				<div class="flex flex-col gap-2.5">
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.wrapperHookTitle) }}
					</h3>
					<Input
						id="wrapper"
						v-model="settings.hooks.wrapper"
						autocomplete="off"
						type="text"
						:placeholder="formatMessage(messages.wrapperHookPlaceholder)"
						wrapper-class="w-full"
					/>
					<p class="m-0 leading-tight">
						{{ formatMessage(messages.wrapperHookDescription) }}
					</p>
				</div>

				<div class="flex flex-col gap-2.5">
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.postExitHookTitle) }}
					</h3>
					<Input
						id="post-exit"
						v-model="settings.hooks.post_exit"
						autocomplete="off"
						type="text"
						:placeholder="formatMessage(messages.postExitHookPlaceholder)"
						wrapper-class="w-full"
					/>
					<p class="m-0 leading-tight">
						{{ formatMessage(messages.postExitHookDescription) }}
					</p>
				</div>

				<div class="m-0 leading-tight">
					{{ formatMessage(messages.hookVariablesDescription) }}
					<ul>
						<li>{{ formatMessage(messages.instanceNameDescription) }}</li>
						<li>{{ formatMessage(messages.instanceIdDescription) }}</li>
						<li>{{ formatMessage(messages.instanceDirDescription) }}</li>
						<li>{{ formatMessage(messages.instanceMcDirDescription) }}</li>
						<li>{{ formatMessage(messages.instanceJavaDescription) }}</li>
						<li>{{ formatMessage(messages.instanceJavaArgsDescription) }}</li>
					</ul>
				</div>
			</div>
		</section>
	</div>
</template>

<style>
.command-history-editor.ace-modrinth {
	background-color: var(--surface-2);
}

.command-history-editor.ace-modrinth .ace_gutter {
	background: var(--surface-1);
}

.command-history-editor.ace-modrinth .ace_marker-layer .ace_active-line {
	background: var(--surface-2-5);
}

.command-history-editor.ace-modrinth .ace_gutter-active-line {
	background-color: var(--surface-1-5);
}

.command-history-editor.ace-modrinth.ace_multiselect .ace_selection.ace_start {
	box-shadow: 0 0 3px 0 var(--surface-2);
}
</style>
