<template>
	<div class="mx-auto w-full max-w-[1280px] px-6 pb-6">
		<div v-if="isLoading" class="py-8">
			<LoadingIndicator />
		</div>
		<div v-else-if="hasError" class="py-8 text-secondary">Failed to load server.</div>
		<template v-else>
			<!-- TODO-SERVERS: make a shared server header component for website and app to share -->
			<RouterLink
				to="/hosting/manage"
				class="breadcrumb goto-link mt-6 mb-4 flex w-fit items-center"
			>
				<LeftArrowIcon />
				All servers
			</RouterLink>
			<ContentPageHeader class="mb-4">
				<template #icon>
					<ServerIcon
						:image="
							server.is_medal
								? 'https://cdn-raw.modrinth.com/medal_icon.webp'
								: (serverImage ?? undefined)
						"
					/>
				</template>
				<template #title>
					{{ server.name || 'Server' }}
				</template>
				<template #stats>
					<div
						v-if="server.flows?.intro"
						class="flex items-center gap-2 font-semibold text-secondary"
					>
						<SettingsIcon />
						Configuring server...
					</div>
					<div v-else class="flex flex-wrap items-center gap-2">
						<div v-if="server.loader" class="flex items-center gap-2 font-medium capitalize">
							<LoaderIcon :loader="server.loader" class="flex shrink-0 [&&]:size-5" />
							{{ server.loader }} {{ server.mc_version }}
						</div>

						<div
							v-if="server.loader && server.net?.domain"
							class="h-1.5 w-1.5 rounded-full bg-surface-5"
						/>

						<div
							v-if="server.net?.domain"
							v-tooltip="'Copy server address'"
							class="flex cursor-pointer items-center gap-2 font-medium hover:underline"
							@click="copyServerAddress"
						>
							<LinkIcon class="flex size-5 shrink-0" />
							{{ server.net.domain }}.modrinth.gg
						</div>

						<div
							v-if="serverProject && (server.loader || server.net?.domain)"
							class="h-1.5 w-1.5 rounded-full bg-surface-5"
						/>

						<div v-if="serverProject" class="flex items-center gap-1.5 font-medium text-primary">
							Linked to
							<Avatar :src="serverProject.icon_url" :alt="serverProject.title" size="24px" />
							<RouterLink :to="serverProjectLink" class="truncate text-primary hover:underline">
								{{ serverProject.title }}
							</RouterLink>
						</div>
					</div>
				</template>
			</ContentPageHeader>

			<div class="mb-4">
				<NavTabs :links="tabs" />
			</div>

			<div class="pt-2">
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
import {
	BoxesIcon,
	DatabaseBackupIcon,
	FolderOpenIcon,
	LeftArrowIcon,
	LinkIcon,
	SettingsIcon,
} from '@modrinth/assets'
import {
	Avatar,
	type BusyReason,
	ContentPageHeader,
	defineMessage,
	injectModrinthClient,
	injectNotificationManager,
	LoaderIcon,
	LoadingIndicator,
	NavTabs,
	provideModrinthServerContext,
	ServerIcon,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, onUnmounted, reactive, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'

const route = useRoute()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()

const serverId = computed(() => {
	const rawId = route.params.id
	return Array.isArray(rawId) ? rawId[0] : (rawId ?? '')
})

const basePath = computed(() => `/hosting/manage/${encodeURIComponent(serverId.value)}`)

const server = ref<Archon.Servers.v0.Server>({} as Archon.Servers.v0.Server)
const worldId = ref<string | null>(null)

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

const isConnected = ref(true)
const powerState = ref<Archon.Websocket.v0.PowerState>('stopped')
const isServerRunning = computed(() => powerState.value === 'running')
const backupsState = reactive(new Map())
const isSyncingContent = ref(false)

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

		const auth = await client.archon.servers_v0.getFilesystemAuth(serverId.value)

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

function copyServerAddress() {
	if (!server.value?.net?.domain) return
	navigator.clipboard.writeText(server.value.net.domain + '.modrinth.gg')
	addNotification({
		title: 'Server address copied',
		text: "Your server's address has been copied to your clipboard.",
		type: 'success',
	})
}

watch(
	() => serverId.value,
	() => {
		fsAuth.value = null
		void refreshFsAuth().catch(() => {})
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
