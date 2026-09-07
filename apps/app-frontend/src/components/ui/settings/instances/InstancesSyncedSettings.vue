<script setup lang="ts">
import {
	EditIcon,
	// FolderOpenIcon,
	RefreshCwIcon,
	SaveIcon,
	SearchIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	CheckCircleButton,
	commonMessages,
	defineMessages,
	IconButton,
	injectNotificationManager,
	Input,
	NewModal,
	Slider,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import type { Component } from 'vue'
import { computed, ref, shallowRef, watch } from 'vue'

import WorldItem from '@/components/ui/world/WorldItem.vue'
import useMemorySlider from '@/composables/useMemorySlider'
import {
	get_command_history,
	get_global_synced_options,
	getInstanceIconUrl,
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
	refreshServerData,
	refreshServers,
	type ServerData,
	type ServerWorld,
} from '@/helpers/worlds.ts'
import { instanceKeys } from '@/pages/instance/query-options'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const queryClient = useQueryClient()

const messages = defineMessages({
	// syncedDescription: {
	// 	id: 'app.settings.synced-options.description',
	// 	defaultMessage:
	// 		'Sync options and config across instances so you don’t have to set them up every time.',
	// },
	// syncedFolder: {
	// 	id: 'app.settings.synced-options.folder',
	// 	defaultMessage: 'Synced folder',
	// },
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
	chooseSyncSourceTitle: {
		id: 'app.settings.synced-options.choose-sync-source.title',
		defaultMessage: 'Choose a sync source',
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
	searchInstance: {
		id: 'app.settings.synced-options.choose-sync-source.search-placeholder',
		defaultMessage: 'Search instance',
	},
	noInstancesFound: {
		id: 'app.settings.synced-options.choose-sync-source.no-instances-found',
		defaultMessage: 'No instances found',
	},
	syncButton: {
		id: 'app.settings.synced-options.choose-sync-source.sync',
		defaultMessage: 'Sync',
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
	noSyncedServers: {
		id: 'app.settings.synced-options.multiplayer-servers.empty',
		defaultMessage: 'No user-added servers are currently synced.',
	},
	noServersSyncedYet: {
		id: 'app.settings.synced-options.multiplayer-servers.none-synced-yet',
		defaultMessage: 'No servers synced yet',
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
	editable?: 'servers' | 'commands'
}> = [
	{
		option: 'multiplayer_servers',
		title: 'multiplayerServers',
		description: 'multiplayerServersDescription',
		editable: 'servers',
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

const globalSyncedOptionsQueryKey = ['global-synced-options'] as const
const globalSyncedOptionsMutationKey = ['global-synced-options', 'set'] as const
const defaultGlobalOptions: GlobalSyncedOptions = {
	command_history: false,
	multiplayer_servers: false,
	creative_hotbars: false,
	screenshots: false,
}

const globalOptionsQuery = useQuery({
	queryKey: globalSyncedOptionsQueryKey,
	queryFn: get_global_synced_options,
})
const globalOptions = computed(() => globalOptionsQuery.data.value ?? defaultGlobalOptions)
const instances = ref(await listInstances().catch(() => []))
const baseOption = ref<SyncedOption | null>(null)
const baseInstanceId = ref(instances.value[0]?.id ?? '')
const baseInstanceSearch = ref('')
const baseModal = ref<InstanceType<typeof NewModal> | null>(null)
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
const editorComponent = shallowRef<Component | null>(null)

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

const baseInstanceDescription = computed(() => {
	switch (baseOption.value) {
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

const filteredBaseInstances = computed(() => {
	const search = baseInstanceSearch.value.trim().toLowerCase()
	if (!search) return instances.value
	return instances.value.filter((instance) => instance.name.toLowerCase().includes(search))
})

async function invalidateSyncedOptions() {
	await Promise.all([
		queryClient.invalidateQueries({ queryKey: instanceKeys.all }),
		queryClient.invalidateQueries({ queryKey: ['instance-synced-options'] }),
		queryClient.invalidateQueries({ queryKey: globalSyncedOptionsQueryKey }),
	])
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

		queryClient.setQueryData<GlobalSyncedOptions>(globalSyncedOptionsQueryKey, (current) => ({
			...(current ?? defaultGlobalOptions),
			[option]: enabled,
		}))

		return { previous }
	},
	onError: (error, { option }, context) => {
		queryClient.setQueryData<GlobalSyncedOptions>(globalSyncedOptionsQueryKey, (current) => ({
			...(current ?? defaultGlobalOptions),
			[option]: context?.previous ?? defaultGlobalOptions[option],
		}))
		handleError(error)
	},
	onSuccess: async (_options, { option, enabled }) => {
		if (enabled) baseModal.value?.hide()
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

function toggleGlobalOption(option: SyncedOption, enabled: boolean) {
	if (!enabled) {
		applyGlobalOption(option, false)
		return
	}

	baseOption.value = option
	baseInstanceId.value = instances.value[0]?.id ?? ''
	baseInstanceSearch.value = ''
	baseModal.value?.show()
}

function confirmBaseInstance() {
	if (!baseOption.value || !baseInstanceId.value) return
	applyGlobalOption(baseOption.value, true, baseInstanceId.value)
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
	serverData.value = {}
	serverEditorModal.value?.show()
	await refreshServers(
		syncedServerCards.value.map(({ world }) => world),
		serverData.value,
		null,
	)
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

async function removeSyncedServer(serverId: string) {
	try {
		await remove_synced_server(serverId)
		syncedServers.value = syncedServers.value.filter((server) => server.id !== serverId)
		await queryClient.invalidateQueries({ queryKey: ['worlds'] })
	} catch (error) {
		handleError(error)
	}
}

const fetchSettings = await get()
fetchSettings.launchArgs = fetchSettings.extra_launch_args.join(' ')
fetchSettings.envVars = serializeEnvVars(fetchSettings.custom_env_vars)

const settings = ref(fetchSettings)

const { maxMemory, snapPoints } = (await useMemorySlider().catch(handleError)) as unknown as {
	maxMemory: number
	snapPoints: number[]
}

watch(
	settings,
	async () => {
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
	<div>
		<NewModal
			ref="baseModal"
			:header="formatMessage(messages.chooseSyncSourceTitle)"
			no-padding
			actions-divider
			max-width="560px"
			width="560px"
		>
			<p class="m-0 border-0 border-b border-solid border-surface-5 p-6 text-primary">
				{{ baseInstanceDescription }}
			</p>

			<div class="flex h-[400px] flex-col gap-3 overflow-y-auto bg-surface-2 px-6 py-4">
				<Input
					v-model="baseInstanceSearch"
					:icon="SearchIcon"
					type="search"
					autocomplete="off"
					:placeholder="formatMessage(messages.searchInstance)"
					class="shrink-0"
				/>

				<div
					v-if="filteredBaseInstances.length === 0"
					class="flex flex-1 items-center justify-center text-secondary"
				>
					{{ formatMessage(messages.noInstancesFound) }}
				</div>
				<div
					v-else
					role="radiogroup"
					:aria-label="formatMessage(messages.chooseSyncSourceTitle)"
					class="flex flex-col gap-1"
				>
					<CheckCircleButton
						v-for="instance in filteredBaseInstances"
						:key="instance.id"
						:checked="baseInstanceId === instance.id"
						class="h-10"
						@click="baseInstanceId = instance.id"
					>
						<span class="size-5 shrink-0 overflow-hidden rounded-[6px]">
							<Avatar
								:src="getInstanceIconUrl(instance.icon_path)"
								:alt="instance.name"
								:tint-by="instance.id"
								size="1.25rem"
								no-shadow
							/>
						</span>
						<span class="truncate">{{ instance.name }}</span>
					</CheckCircleButton>
				</div>
			</div>
			<template #actions>
				<div class="flex justify-end gap-2 p-2">
					<Button type="outlined" @click="baseModal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button
						type="colored"
						color="brand"
						:disabled="!baseInstanceId || globalOptionMutation.isPending.value"
						@click="confirmBaseInstance"
					>
						<RefreshCwIcon />
						{{ formatMessage(messages.syncButton) }}
					</Button>
				</div>
			</template>
		</NewModal>

		<NewModal
			ref="commandHistoryModal"
			:header="formatMessage(messages.commandHistoryEditorTitle)"
			class="command-history-modal"
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
				class="command-history-editor ace-modrinth rounded-[20px] !border !border-solid !border-surface-5"
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
			:header="formatMessage(messages.serverEditorTitle)"
			scrollable
			actions-divider
			no-padding
			max-content-height="34.5rem"
			max-width="750px"
			width="750px"
		>
			<p v-if="syncedServers.length === 0" class="m-0 px-6 py-4 text-secondary">
				{{ formatMessage(messages.noSyncedServers) }}
			</p>
			<div v-else class="flex flex-col gap-2 px-6 py-4">
				<WorldItem
					v-for="{ server, world } in syncedServerCards"
					:key="server.id"
					:world="world"
					card-background="surface-2"
					:show-play-button="false"
					:refreshing="serverData[server.address]?.refreshing"
					:server-status="serverData[server.address]?.status"
					:rendered-motd="serverData[server.address]?.renderedMotd"
					@refresh="refreshSyncedServer(server.address)"
					@edit="openSyncedServerEditor(server)"
					@delete="removeSyncedServer(server.id)"
				/>
			</div>
			<template #actions>
				<div class="flex justify-end">
					<Button type="outlined" @click="serverEditorModal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.closeButton) }}
					</Button>
				</div>
			</template>
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
								v-if="row.editable"
								v-tooltip="
									row.editable === 'servers' && syncedServers.length === 0
										? formatMessage(messages.noServersSyncedYet)
										: formatMessage(commonMessages.editButton)
								"
								class="flex"
							>
								<IconButton
									type="outlined"
									circular
									:disabled="
										!globalOptions[row.option] ||
										(row.editable === 'servers' && syncedServers.length === 0)
									"
									:label="formatMessage(commonMessages.editButton)"
									@click="
										row.editable === 'commands' ? openCommandHistoryEditor() : openServerEditor()
									"
								>
									<EditIcon />
								</IconButton>
							</span>
							<Toggle
								:id="`global-sync-${row.option}`"
								:model-value="globalOptions[row.option]"
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

.command-history-modal > [data-modal-content] {
	padding-bottom: 0;
}
</style>
