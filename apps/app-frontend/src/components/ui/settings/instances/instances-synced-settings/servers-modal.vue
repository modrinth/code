<script setup lang="ts">
import {
	ClipboardCopyIcon,
	EditIcon,
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
	injectNotificationManager,
	Input,
	NewModal,
	Table,
	type TableColumn,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, ref } from 'vue'

import { remove_synced_server, type SyncedServer, update_synced_server } from '@/helpers/instance'
import { syncedOptionsKeys, syncedServersQueryOptions } from '@/helpers/synced-options'
import { copyToClipboard } from '@/helpers/utils'
import {
	refreshServerData,
	refreshServers,
	type ServerData,
	type ServerWorld,
} from '@/helpers/worlds'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const queryClient = useQueryClient()
const serverEditorModal = ref<InstanceType<typeof NewModal> | null>(null)
const editServerModal = ref<InstanceType<typeof NewModal> | null>(null)
const editedServer = ref<SyncedServer | null>(null)
const serversQuery = useQuery(syncedServersQueryOptions())
const syncedServers = computed(() => serversQuery.data.value ?? [])
const serverData = ref<Record<string, ServerData>>({})
const serverSearch = ref('')
const selectedServerFilters = ref<string[]>([])

const messages = defineMessages({
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
})

const serverWorlds = computed<ServerWorld[]>(() =>
	syncedServers.value.map((server, index) => ({
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
	})),
)

type SyncedServerTableColumn = 'server' | 'status' | 'version' | 'actions'
type SyncedServerTableRow = SyncedServer & {
	server: string
	status: 'online' | 'offline'
	version: string
	actions: null
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

const refreshMutation = useMutation({
	mutationFn: () => refreshServers(serverWorlds.value, serverData.value, null),
	onError: handleError,
})
const refreshingAllSyncedServers = refreshMutation.isPending
const saveMutation = useMutation({
	mutationFn: update_synced_server,
	onSuccess: async (_, server) => {
		queryClient.setQueryData<SyncedServer[]>(syncedOptionsKeys.servers, (servers) =>
			servers?.map((current) => (current.id === server.id ? { ...server } : current)),
		)
		await queryClient.invalidateQueries({ queryKey: ['worlds'] })
		void refreshSyncedServer(server.address).catch(handleError)
	},
	onError: handleError,
})
const removeMutation = useMutation({
	mutationFn: remove_synced_server,
	onSuccess: async (_, serverId) => {
		queryClient.setQueryData<SyncedServer[]>(syncedOptionsKeys.servers, (servers) =>
			servers?.filter((server) => server.id !== serverId),
		)
		await queryClient.invalidateQueries({ queryKey: ['worlds'] })
	},
	onError: handleError,
})

async function show() {
	const result = await serversQuery.refetch()
	if (result.isError) {
		handleError(result.error)
		return
	}
	serverSearch.value = ''
	selectedServerFilters.value = []
	serverData.value = {}
	serverEditorModal.value?.show()
	refreshAllSyncedServers()
}

function openSyncedServerEditor(server: SyncedServer) {
	editedServer.value = { ...server }
	editServerModal.value?.show()
}

async function saveSyncedServer() {
	if (!editedServer.value || saveMutation.isPending.value) return
	try {
		await saveMutation.mutateAsync({ ...editedServer.value })
		await nextTick()
		editServerModal.value?.hide()
	} catch {
		return
	}
}

async function refreshSyncedServer(address: string) {
	serverData.value[address] ??= { refreshing: true }
	await refreshServerData(serverData.value[address], null, address)
}

function refreshAllSyncedServers() {
	if (!refreshingAllSyncedServers.value) refreshMutation.mutate()
}

function syncedServerMenuOptions(server: SyncedServer): ButtonMenuOption[] {
	return [
		{
			id: 'refresh',
			label: formatMessage(commonMessages.refreshButton),
			icon: RefreshCwIcon,
			action: () => refreshSyncedServer(server.address).catch(handleError),
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
			disabled: removeMutation.isPending.value,
			action: () => removeMutation.mutate(server.id),
		},
	]
}

defineExpose({ show })
</script>

<template>
	<NewModal
		ref="serverEditorModal"
		no-padding
		max-width="min(928px, calc(95vw - 10rem))"
		width="min(928px, calc(95vw - 10rem))"
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
					size="medium"
					:aria-label="formatMessage(messages.searchServers, { count: syncedServers.length })"
					:clear-label="formatMessage(commonMessages.clearButton)"
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
						interaction="none"
						@click="refreshAllSyncedServers"
					>
						<RefreshCwIcon
							:class="{ 'animate-spin': refreshingAllSyncedServers }"
							aria-hidden="true"
						/>
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
		:disable-close="saveMutation.isPending.value"
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
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					:disabled="!editedServer?.address"
					:loading="saveMutation.isPending.value"
					@click="saveSyncedServer"
				>
					<SaveIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.saveChangesButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
