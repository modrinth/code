<template>
	<div class="mx-auto w-full max-w-[1280px] px-6 pb-6">
		<div v-if="isLoading" class="py-8">
			<LoadingIndicator />
		</div>
		<div v-else-if="hasError" class="py-8 text-secondary">Failed to load server.</div>
		<template v-else>
			<ServerManageHeader
				:server="server"
				:server-image="serverImage"
				:server-project="serverProject"
				:server-project-link="serverProjectLink"
				breadcrumb-class="breadcrumb goto-link mt-6 mb-4 flex w-fit items-center"
				header-class="mb-4"
				:show-uptime="false"
			>
				<template #actions>
					<div class="flex gap-2" v-if="isConnected && !server.flows?.intro">
						<PanelServerActionButton />
						<ButtonStyled circular size="large">
							<button v-tooltip="'Server settings'" @click="openServerSettingsModal">
								<SettingsIcon />
							</button>
						</ButtonStyled>
						<PanelServerOverflowMenu :show-copy-id-action="themeStore.devMode" />
					</div>
				</template>
			</ServerManageHeader>

			<div class="mb-4">
				<NavTabs :links="tabs" />
			</div>

			<div class="pt-2">
				<Suspense>
					<ServerSettingsModal ref="serverSettingsModal" />
				</Suspense>
				<RouterView v-slot="{ Component }">
					<template v-if="Component">
						<Suspense>
							<component :is="Component" />
							<template #fallback>
								<LoadingIndicator />
							</template>
						</Suspense>
					</template>
				</RouterView>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import { type Archon, clearNodeAuthState, setNodeAuthState } from '@modrinth/api-client'
import { BoxesIcon, DatabaseBackupIcon, FolderOpenIcon, SettingsIcon } from '@modrinth/assets'
import {
	type BusyReason,
	ButtonStyled,
	defineMessage,
	injectModrinthClient,
	LoadingIndicator,
	NavTabs,
	PanelServerActionButton,
	PanelServerOverflowMenu,
	provideModrinthServerContext,
	ServerManageHeader,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, onUnmounted, reactive, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import ServerSettingsModal from '@/components/ui/modal/ServerSettingsModal.vue'
import { useTheming } from '@/store/theme'

const route = useRoute()
const themeStore = useTheming()
const client = injectModrinthClient()
const queryClient = useQueryClient()

const serverId = computed(() => {
	const rawId = route.params.id
	return Array.isArray(rawId) ? rawId[0] : (rawId ?? '')
})

const basePath = computed(() => `/hosting/manage/${encodeURIComponent(serverId.value)}`)

const server = ref<Archon.Servers.v0.Server>({} as Archon.Servers.v0.Server)
const worldId = ref<string | null>(null)
const serverSettingsModal = ref<InstanceType<typeof ServerSettingsModal> | null>(null)

const serverQuery = useQuery({
	queryKey: computed(() => ['servers', 'detail', serverId.value]),
	queryFn: () => client.archon.servers_v0.get(serverId.value),
	enabled: computed(() => !!serverId.value),
})

const serverV1Query = useQuery({
	queryKey: computed(() => ['servers', 'v1', 'detail', serverId.value]),
	queryFn: () => client.archon.servers_v1.get(serverId.value),
	enabled: computed(() => !!serverId.value),
})

watch(
	() => serverQuery.data.value,
	(serverData) => {
		if (serverData) {
			server.value = serverData
		}
	},
	{ immediate: true },
)

watch(
	() => serverV1Query.data.value,
	(serverData) => {
		if (!serverData) return
		const activeWorld = serverData.worlds.find((world) => world.is_active)
		worldId.value = activeWorld?.id ?? serverData.worlds[0]?.id ?? null
	},
	{ immediate: true },
)

const isLoading = computed(() => serverQuery.isPending.value || serverV1Query.isPending.value)
const hasError = computed(() => !!serverQuery.error.value || !!serverV1Query.error.value)

const { data: serverProject } = useQuery({
	queryKey: computed(() => ['server-project', server.value.upstream?.project_id]),
	queryFn: async () => {
		if (!server.value.upstream?.project_id) return null
		return await client.labrinth.projects_v2.get(server.value.upstream.project_id)
	},
	enabled: computed(() => !!server.value.upstream?.project_id),
})

const serverProjectLink = computed(() => {
	if (!serverProject.value) return ''
	return `/project/${serverProject.value.slug ?? serverProject.value.id}`
})

const isConnected = ref(false)
const powerState = ref<Archon.Websocket.v0.PowerState>('stopped')
const isServerRunning = computed(() => powerState.value === 'running')
const backupsState = reactive(new Map())
const isSyncingContent = ref(false)
const socketUnsubscribers = ref<(() => void)[]>([])
const connectedSocketServerId = ref<string | null>(null)

const busyReasons = computed<BusyReason[]>(() => {
	const reasons: BusyReason[] = []
	if (server.value?.status === 'installing') {
		reasons.push({
			reason: defineMessage({
				id: 'servers.busy.installing',
				defaultMessage: 'Server is installing',
			}),
		})
	}
	if (isSyncingContent.value) {
		reasons.push({
			reason: defineMessage({
				id: 'servers.busy.syncing-content',
				defaultMessage: 'Content sync in progress',
			}),
		})
	}
	return reasons
})

const fsAuth = ref<{ url: string; token: string } | null>(null)
const fsOps = ref<Archon.Websocket.v0.FilesystemOperation[]>([])
const fsQueuedOps = ref<Archon.Websocket.v0.QueuedFilesystemOp[]>([])

async function processImageBlob(blob: Blob, size: number): Promise<string> {
	return new Promise((resolve) => {
		const canvas = document.createElement('canvas')
		const ctx = canvas.getContext('2d')!
		const img = new Image()
		img.onload = () => {
			canvas.width = size
			canvas.height = size
			ctx.drawImage(img, 0, 0, size, size)
			const dataURL = canvas.toDataURL('image/png')
			URL.revokeObjectURL(img.src)
			resolve(dataURL)
		}
		img.src = URL.createObjectURL(blob)
	})
}

const { data: serverImage } = useQuery({
	queryKey: computed(() => ['server-icon', serverId.value, serverProject.value?.icon_url ?? null]),
	queryFn: async (): Promise<string | null> => {
		if (!serverId.value || server.value.status !== 'available') return null

		try {
			const blob = await client.kyros.files_v0.downloadFile('/server-icon-original.png')
			return await processImageBlob(blob, 512)
		} catch {
			if (serverProject.value?.icon_url) {
				return serverProject.value.icon_url
			}
			return null
		}
	},
	enabled: computed(() => !!serverId.value && server.value.status === 'available'),
})

async function refreshFsAuth() {
	if (!serverId.value) {
		fsAuth.value = null
		return
	}
	fsAuth.value = await queryClient.fetchQuery({
		queryKey: ['servers', 'filesystem-auth', serverId.value],
		queryFn: () => client.archon.servers_v0.getFilesystemAuth(serverId.value),
	})
}

function markBackupCancelled(backupId: string) {
	backupsState.delete(backupId)
}

function openServerSettingsModal() {
	if (!serverId.value) return
	serverSettingsModal.value?.show({ serverId: serverId.value })
}

function setPowerState(state: Archon.Websocket.v0.PowerState) {
	powerState.value = state
}

function handlePowerState(data: Archon.Websocket.v0.WSPowerStateEvent) {
	setPowerState(data.state)
}

function handleState(data: Archon.Websocket.v0.WSStateEvent) {
	const powerMap: Record<Archon.Websocket.v0.FlattenedPowerState, Archon.Websocket.v0.PowerState> =
		{
			not_ready: 'stopped',
			starting: 'starting',
			running: 'running',
			stopping: 'stopping',
			idle:
				data.was_oom || (data.exit_code != null && data.exit_code !== 0) ? 'crashed' : 'stopped',
		}
	setPowerState(powerMap[data.power_variant])
}

function disconnectSocket(targetServerId?: string) {
	for (const unsub of socketUnsubscribers.value) unsub()
	socketUnsubscribers.value = []

	if (targetServerId) {
		client.archon.sockets.disconnect(targetServerId)
	}
	connectedSocketServerId.value = null
	isConnected.value = false
	setPowerState('stopped')
}

async function connectSocket(targetServerId: string) {
	if (connectedSocketServerId.value === targetServerId && isConnected.value) {
		return
	}
	disconnectSocket(targetServerId)

	try {
		await client.archon.sockets.safeConnect(targetServerId, { force: true })
		connectedSocketServerId.value = targetServerId
		isConnected.value = true
		socketUnsubscribers.value = [
			client.archon.sockets.on(targetServerId, 'state', handleState),
			client.archon.sockets.on(targetServerId, 'power-state', handlePowerState),
			client.archon.sockets.on(targetServerId, 'auth-incorrect', () => {
				isConnected.value = false
			}),
			client.archon.sockets.on(targetServerId, 'auth-ok', () => {
				isConnected.value = true
			}),
		]
	} catch (error) {
		console.error('[hosting/manage] Failed to connect server socket:', error)
		isConnected.value = false
	}
}

watch(
	() => serverId.value,
	(newServerId, oldServerId) => {
		if (oldServerId && oldServerId !== newServerId) {
			disconnectSocket(oldServerId)
		}
		fsAuth.value = null
		void refreshFsAuth().catch(() => {})
	},
	{ immediate: true },
)

watch(
	() => [serverId.value, serverQuery.data.value] as const,
	([currentServerId, currentServer]) => {
		if (!currentServerId || !currentServer) return
		if (currentServer.status === 'suspended' || currentServer.node === null) {
			disconnectSocket(currentServerId)
			return
		}
		if (connectedSocketServerId.value === currentServerId && isConnected.value) {
			return
		}
		void connectSocket(currentServerId)
	},
	{ immediate: true },
)

provideModrinthServerContext({
	get serverId() {
		return serverId.value
	},
	worldId,
	server,
	isConnected,
	powerState,
	isServerRunning,
	backupsState,
	markBackupCancelled,
	isSyncingContent,
	busyReasons,
	fsAuth,
	fsOps,
	fsQueuedOps,
	refreshFsAuth,
})

setNodeAuthState(() => fsAuth.value, refreshFsAuth)

onUnmounted(() => {
	disconnectSocket(serverId.value || undefined)
	clearNodeAuthState()
})

const tabs = computed(() => [
	{
		label: 'Content',
		href: `${basePath.value}/content`,
		icon: BoxesIcon,
	},
	{
		label: 'Files',
		href: `${basePath.value}/files`,
		icon: FolderOpenIcon,
	},
	{
		label: 'Backups',
		href: `${basePath.value}/backups`,
		icon: DatabaseBackupIcon,
	},
])
</script>
