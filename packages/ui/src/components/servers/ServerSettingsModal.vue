<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { ChevronRightIcon } from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, ref } from 'vue'

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
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

type ShowOptions = {
	serverId: string
	tabIndex?: number
	tabId?: ServerSettingsTabId
}

const props = defineProps<{
	resolveViewer: () => Promise<{ userId: string | null; userRole: string | null }>
	browseModpacks?: (args: {
		serverId: string
		worldId: string | null
		from: 'reset-server'
	}) => void | Promise<void>
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

const { serverId: currentServerId, worldId, server } = injectModrinthServerContext()

const currentUserId = ref<string | null>(null)
const currentUserRole = ref<string | null>(null)

const isApp = ref(true)

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
	browseModpacks: props.browseModpacks ?? (() => {}),
	closeModal: () => hide(),
})

const ownerId = computed(() => server.value?.owner_id ?? 'Ghost')
const isOwner = computed(() => currentUserId.value != null && currentUserId.value === ownerId.value)
const isAdmin = computed(() => currentUserRole.value === 'admin')

const tabs = computed<TabbedModalTab[]>(() =>
	serverSettingsTabDefinitions.map((tab) => {
		const ctx = {
			serverId: currentServerId,
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
		const targetServerId = currentServerId
		if (serverId !== targetServerId) {
			console.warn(
				`[ServerSettingsModal] Ignoring mismatched serverId "${serverId}" in favor of context "${targetServerId}"`,
			)
		}

		const cachedServer = queryClient.getQueryData<Archon.Servers.v0.Server>([
			'servers',
			'detail',
			targetServerId,
		])
		const cachedFull = queryClient.getQueryData<Archon.Servers.v1.ServerFull>([
			'servers',
			'v1',
			'detail',
			targetServerId,
		])

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

		if (!cachedServer) {
			fetchPromises.push(
				queryClient.fetchQuery({
					queryKey: ['servers', 'detail', targetServerId],
					queryFn: () => client.archon.servers_v0.get(targetServerId),
				}),
			)
		}

		if (!cachedFull) {
			fetchPromises.push(
				queryClient.fetchQuery({
					queryKey: ['servers', 'v1', 'detail', targetServerId],
					queryFn: () => client.archon.servers_v1.get(targetServerId),
				}),
			)
		}

		await Promise.all(fetchPromises)

		if (worldId.value) {
			queryClient.prefetchQuery({
				queryKey: ['servers', 'properties', 'v1', targetServerId, worldId.value],
				queryFn: () => client.archon.properties_v1.getProperties(targetServerId, worldId.value!),
			})
			queryClient.prefetchQuery({
				queryKey: ['content', 'list', 'v1', targetServerId],
				queryFn: () =>
					client.archon.content_v1.getAddons(targetServerId, worldId.value!, {
						from_modpack: false,
					}),
			})
			queryClient.prefetchQuery({
				queryKey: ['servers', 'startup', 'v1', targetServerId, worldId.value],
				queryFn: () => client.archon.options_v1.getStartup(targetServerId, worldId.value!),
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

defineExpose({ show, hide })
</script>

<template>
	<TabbedModal
		ref="modal"
		:tabs="tabs"
		:max-width="'min(980px, calc(95vw - 2rem))'"
		:width="'min(980px, calc(95vw - 2rem))'"
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
