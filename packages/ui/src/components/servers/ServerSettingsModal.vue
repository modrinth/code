<script setup lang="ts">
import { type Archon, clearNodeAuthState, setNodeAuthState } from '@modrinth/api-client'
import { ChevronRightIcon } from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, onUnmounted, reactive, ref } from 'vue'

import type { TabbedModalTab } from '#ui/components'
import { TabbedModal } from '#ui/components'
import { defineMessage, defineMessages, useVIntl } from '#ui/composables/i18n'
import {
	ServerSettingsAdvancedPage,
	ServerSettingsGeneralPage,
	ServerSettingsInstallationPage,
	ServerSettingsNetworkPage,
	ServerSettingsPropertiesPage,
	serverSettingsTabDefinitions,
	type ServerSettingsTabId,
} from '#ui/layouts/shared/server-settings'
import { provideServerSettings } from '#ui/layouts/shared/server-settings/providers/server-settings'
import { injectModrinthClient, injectNotificationManager } from '#ui/providers'
import { type BusyReason, provideModrinthServerContext } from '#ui/providers/server-context'
import { commonMessages } from '#ui/utils/common-messages'

type ShowOptions = {
	serverId: string
	tabIndex?: number
	tabId?: ServerSettingsTabId
}

const props = defineProps<{
	resolveViewer: () => Promise<{ userId: string | null; userRole: string | null }>
	browseModpacks?: (args: { serverId: string; worldId: string | null; from: 'reset-server' }) => void | Promise<void>
}>()

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

function browseModpacks(args: { serverId: string; worldId: string | null; from: 'reset-server' }) {
	props.browseModpacks?.(args)
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
	stats: ref({
		current: {
			cpu_percent: 0,
			ram_usage_bytes: 0,
			ram_total_bytes: 1,
			storage_usage_bytes: 0,
			storage_total_bytes: 0,
		},
		past: {
			cpu_percent: 0,
			ram_usage_bytes: 0,
			ram_total_bytes: 1,
			storage_usage_bytes: 0,
			storage_total_bytes: 0,
		},
		graph: { cpu: [], ram: [] },
	}),
	isWsAuthIncorrect: ref(false),
	powerStateDetails: ref(undefined),
	uptimeSeconds: ref(0),
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
				href: tab.href ? `https://modrinth.com${tab.href(ctx)}` : undefined,
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

async function fetchViewer() {
	currentUserId.value = null
	currentUserRole.value = null

	const result = await props.resolveViewer()
	currentUserId.value = result.userId
	currentUserRole.value = result.userRole
}

async function show({ serverId, tabIndex, tabId }: ShowOptions) {
	try {
		currentServerId.value = serverId

		const cachedServer = queryClient.getQueryData<Archon.Servers.v0.Server>([
			'servers',
			'detail',
			serverId,
		])
		const cachedFull = queryClient.getQueryData<Archon.Servers.v1.Server>([
			'servers',
			'v1',
			'detail',
			serverId,
		])

		if (cachedServer) server.value = cachedServer
		if (cachedFull) {
			const activeWorld = cachedFull.worlds.find((world) => world.is_active)
			worldId.value = activeWorld?.id ?? cachedFull.worlds[0]?.id ?? null
		}

		modal.value?.show()
		const visibleTabs = tabs.value.filter((tab) => tab.shown !== false)
		let requestedTab = tabIndex ?? 0
		if (tabId) {
			const defIndex = serverSettingsTabDefinitions.findIndex((d) => d.id === tabId)
			if (defIndex >= 0) {
				const visibleIndex = visibleTabs.findIndex(
					(_, i) => tabs.value.indexOf(visibleTabs[i]) === defIndex,
				)
				if (visibleIndex >= 0) requestedTab = visibleIndex
			}
		}
		const clampedTab = Math.min(Math.max(requestedTab, 0), Math.max(visibleTabs.length - 1, 0))
		nextTick(() => modal.value?.setTab(clampedTab))

		const fetchPromises: Promise<unknown>[] = [fetchViewer()]

		if (!cachedServer || !cachedFull) {
			fetchPromises.push(
				queryClient
					.fetchQuery({
						queryKey: ['servers', 'detail', serverId],
						queryFn: () => client.archon.servers_v0.get(serverId),
					})
					.then((data) => {
						server.value = data
					}),
				queryClient
					.fetchQuery({
						queryKey: ['servers', 'v1', 'detail', serverId],
						queryFn: () => client.archon.servers_v1.get(serverId),
					})
					.then((data) => {
						const activeWorld = data.worlds.find((world) => world.is_active)
						worldId.value = activeWorld?.id ?? data.worlds[0]?.id ?? null
					}),
			)
		}

		await Promise.all(fetchPromises)

		setNodeAuthState(() => fsAuth.value, refreshFsAuth)
		refreshFsAuth().catch(() => {})

		if (worldId.value) {
			queryClient.prefetchQuery({
				queryKey: ['servers', 'properties', 'v1', serverId, worldId.value],
				queryFn: () => client.archon.properties_v1.getProperties(serverId, worldId.value!),
			})
			queryClient.prefetchQuery({
				queryKey: ['content', 'list', 'v1', serverId],
				queryFn: () =>
					client.archon.content_v1.getAddons(serverId, worldId.value!, {
						from_modpack: false,
					}),
			})
			queryClient.prefetchQuery({
				queryKey: ['servers', 'startup', 'v1', serverId, worldId.value],
				queryFn: () => client.archon.options_v1.getStartup(serverId, worldId.value!),
			})
		}
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
