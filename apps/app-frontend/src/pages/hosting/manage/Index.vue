<template>
	<div class="p-6">
		<div v-if="isLoading" class="py-8">
			<LoadingIndicator />
		</div>
		<div v-else-if="hasError" class="py-8 text-secondary">Failed to load server.</div>
		<template v-else>
			<div class="mb-4">
				<h1 class="m-0 text-3xl font-bold text-contrast">{{ server.name || 'Server' }}</h1>
			</div>
			<NavTabs :links="tabs" />
			<div class="pt-4">
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
import { BoxesIcon, DatabaseBackupIcon, FolderOpenIcon } from '@modrinth/assets'
import {
	type BusyReason,
	defineMessage,
	injectModrinthClient,
	LoadingIndicator,
	NavTabs,
	provideModrinthServerContext,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, onUnmounted, reactive, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const client = injectModrinthClient()
const queryClient = useQueryClient()

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
