<script setup lang="ts">
import {
	type Archon,
	clearNodeAuthState,
	type Labrinth,
	setNodeAuthState,
} from '@modrinth/api-client'
import { ChevronRightIcon } from '@modrinth/assets'
import {
	type BusyReason,
	commonMessages,
	defineMessage,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	provideModrinthServerContext,
	provideServerSettings,
	ServerSettingsAdvancedPage,
	ServerSettingsGeneralPage,
	ServerSettingsInstallationPage,
	ServerSettingsNetworkPage,
	ServerSettingsPropertiesPage,
	serverSettingsTabDefinitions,
	TabbedModal,
	type TabbedModalTab,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, onUnmounted, reactive, ref } from 'vue'

import { get_user } from '@/helpers/cache'
import { get as getCreds } from '@/helpers/mr_auth'

type ShowOptions = {
	serverId: string
	tabIndex?: number
}

const { formatMessage } = useVIntl()
const queryClient = useQueryClient()
const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()

const messages = defineMessages({
	failedToLoadServer: {
		id: 'app.server-settings.failed-to-load-server',
		defaultMessage: 'Failed to load server settings',
	},
})

const modal = ref<InstanceType<typeof TabbedModal> | null>(null)

const currentServerId = ref('')
const worldId = ref<string | null>(null)
const server = ref<Archon.Servers.v0.Server>({} as Archon.Servers.v0.Server)

const currentUserId = ref<string | null>(null)
const currentUserRole = ref<string | null>(null)

const isApp = ref(true)

function browseModpacks() {
	// Stub for app browse-modpacks flow. Intentionally no-op for now.
}

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
	if (!currentServerId.value) {
		fsAuth.value = null
		return
	}
	fsAuth.value = await queryClient.fetchQuery({
		queryKey: ['servers', 'filesystem-auth', currentServerId.value],
		queryFn: () => client.archon.servers_v0.getFilesystemAuth(currentServerId.value),
	})
}

function markBackupCancelled(_backupId: string) {}

const serverSettingsTabComponentMap = {
	general: ServerSettingsGeneralPage,
	installation: ServerSettingsInstallationPage,
	network: ServerSettingsNetworkPage,
	properties: ServerSettingsPropertiesPage,
	advanced: ServerSettingsAdvancedPage,
} as const

provideServerSettings({
	isApp,
	currentUserId,
	currentUserRole,
	browseModpacks,
})

provideModrinthServerContext({
	get serverId() {
		return currentServerId.value
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

const ownerId = computed(() => server.value?.owner_id ?? 'Ghost')
const isOwner = computed(() => currentUserId.value != null && currentUserId.value === ownerId.value)
const isAdmin = computed(() => currentUserRole.value === 'admin')

const tabs = computed<TabbedModalTab[]>(() =>
	serverSettingsTabDefinitions.map((tab) => {
		const ctx = {
			serverId: currentServerId.value,
			ownerId: ownerId.value,
			serverStatus: server.value?.status,
			isOwner: isOwner.value,
			isAdmin: isAdmin.value,
		}
		const name = defineMessage({
			id: `server.settings.tabs.${tab.id}`,
			defaultMessage: tab.label,
		})
		const shown = tab.shown ? tab.shown(ctx) : true

		if (tab.external) {
			return {
				name,
				icon: tab.icon,
				href: `https://modrinth.com${tab.href(ctx)}`,
				shown,
			}
		}

		return {
			name,
			icon: tab.icon,
			content: serverSettingsTabComponentMap[tab.id as keyof typeof serverSettingsTabComponentMap],
			shown,
		}
	}),
)

async function resolveViewer() {
	currentUserId.value = null
	currentUserRole.value = null

	const credentials = await getCreds().catch(() => null)
	if (!credentials?.user_id) {
		return
	}

	currentUserId.value = credentials.user_id

	const user = await get_user(credentials.user_id, 'bypass').catch(() => null)
	const typedUser = user as Labrinth.Users.v2.User | null
	currentUserRole.value = typedUser?.role ?? null
}

async function show({ serverId, tabIndex }: ShowOptions) {
	try {
		const [serverData, serverFull] = await Promise.all([
			queryClient.fetchQuery({
				queryKey: ['servers', 'detail', serverId],
				queryFn: () => client.archon.servers_v0.get(serverId),
			}),
			queryClient.fetchQuery({
				queryKey: ['servers', 'v1', 'detail', serverId],
				queryFn: () => client.archon.servers_v1.get(serverId),
			}),
			resolveViewer(),
		])

		currentServerId.value = serverId
		server.value = serverData
		const activeWorld = serverFull.worlds.find((world) => world.is_active)
		worldId.value = activeWorld?.id ?? serverFull.worlds[0]?.id ?? null

		setNodeAuthState(() => fsAuth.value, refreshFsAuth)
		await refreshFsAuth().catch(() => {})

		modal.value?.show()
		const visibleTabsCount = tabs.value.filter((tab) => tab.shown !== false).length
		const requestedTab = tabIndex ?? 0
		const clampedTab = Math.min(Math.max(requestedTab, 0), Math.max(visibleTabsCount - 1, 0))
		nextTick(() => modal.value?.setTab(clampedTab))
	} catch (error) {
		console.error(error)
		addNotification({
			type: 'error',
			title: formatMessage(messages.failedToLoadServer),
		})
	}
}

function hide() {
	modal.value?.hide()
}

function onHide() {
	clearNodeAuthState()
}

onUnmounted(() => {
	clearNodeAuthState()
})

defineExpose({ show, hide })
</script>

<template>
	<TabbedModal
		ref="modal"
		:tabs="tabs"
		:on-hide="onHide"
		:max-width="'min(980px, calc(95vw - 10rem))'"
		:width="'min(980px, calc(95vw - 10rem))'"
	>
		<template #title>
			<span class="flex items-center gap-2 text-lg font-semibold text-primary">
				{{ server.name || 'Server' }} <ChevronRightIcon />
				<span class="font-extrabold text-contrast">{{
					formatMessage(commonMessages.settingsLabel)
				}}</span>
			</span>
		</template>
	</TabbedModal>
</template>
