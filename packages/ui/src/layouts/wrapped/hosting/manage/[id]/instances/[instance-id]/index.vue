<template>
	<div class="flex min-h-[36rem] flex-col gap-4 text-primary">
		<WorldManageHeader
			:name="worldName"
			:game-version="gameVersion"
			:loader="loader"
			:loader-version="loaderVersion"
			:last-active="lastActiveLabel"
			:back-href="instancesPath"
			:back-label="formatMessage(messages.allInstances)"
			:fallback-name="formatMessage(messages.worldFallbackName)"
		>
			<template #actions>
				<PanelServerActionButton size="large" start-label="Start instance" />
				<ButtonStyled size="large" circular>
					<button
						v-tooltip="formatMessage(messages.instanceSettings)"
						@click="openServerSettings({ tabId: 'installation' })"
					>
						<SettingsIcon aria-hidden="true" />
					</button>
				</ButtonStyled>
			</template>
		</WorldManageHeader>

		<NavTabs :links="worldTabLinks" replace />

		<slot />
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { BoxesIcon, DatabaseBackupIcon, FolderOpenIcon, SettingsIcon } from '@modrinth/assets'
import { useQuery } from '@tanstack/vue-query'
import { computed, watch } from 'vue'
import { useRouter } from 'vue-router'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NavTabs from '#ui/components/base/NavTabs.vue'
import { PanelServerActionButton, WorldManageHeader } from '#ui/components/servers/server-header'
import { useRelativeTime } from '#ui/composables'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectServerSettingsModal,
} from '#ui/providers'

interface Tab {
	label: string
	href: string
	icon?: object
	subpages?: string[]
}

const messages = defineMessages({
	allInstances: {
		id: 'servers.manage.instance.all-instances',
		defaultMessage: 'All instances',
	},
	contentNav: {
		id: 'servers.manage.nav.content',
		defaultMessage: 'Content',
	},
	filesNav: {
		id: 'servers.manage.nav.files',
		defaultMessage: 'Files',
	},
	backupsNav: {
		id: 'servers.manage.nav.backups',
		defaultMessage: 'Backups',
	},
	worldFallbackName: {
		id: 'servers.manage.instance.fallback-name',
		defaultMessage: 'Instance',
	},
	lastActive: {
		id: 'servers.manage.instance.last-active',
		defaultMessage: 'Last active {time}',
	},
	instanceSettings: {
		id: 'servers.manage.instance.settings',
		defaultMessage: 'Instance settings',
	},
})

const client = injectModrinthClient()
const { serverId, server, worldId, isServerRunning } = injectModrinthServerContext()
const { openServerSettings } = injectServerSettingsModal()
const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const router = useRouter()

const { data: serverFull } = useQuery({
	queryKey: computed(() => ['servers', 'v1', 'detail', serverId]),
	queryFn: () => client.archon.servers_v1.get(serverId),
	staleTime: 30_000,
})

const instancesPath = computed(() => `/hosting/manage/${encodeURIComponent(serverId)}/instances`)
const worldPath = computed(() =>
	worldId.value
		? `${instancesPath.value}/${encodeURIComponent(worldId.value)}`
		: instancesPath.value,
)

const currentWorld = computed(() => {
	const id = worldId.value
	if (!id) return null
	return serverFull.value?.worlds.find((world) => world.id === id) ?? null
})

const worldName = computed(
	() => currentWorld.value?.name ?? server.value?.name ?? formatMessage(messages.worldFallbackName),
)

const gameVersion = computed(() => {
	const version = currentWorld.value?.content?.game_version ?? server.value?.mc_version
	return version ?? null
})

const loader = computed(
	() => currentWorld.value?.content?.modloader ?? server.value?.loader ?? null,
)

const loaderVersion = computed(
	() => currentWorld.value?.content?.modloader_version ?? server.value?.loader_version ?? null,
)

const lastActiveLabel = computed(() => {
	const latestBackup = currentWorld.value ? latestDate(currentWorld.value.backups) : null
	const lastActiveAt =
		latestBackup ??
		(currentWorld.value?.is_active && isServerRunning.value
			? new Date().toISOString()
			: currentWorld.value?.created_at)

	return lastActiveAt
		? formatMessage(messages.lastActive, { time: formatRelativeTime(lastActiveAt) })
		: null
})

const worldTabLinks = computed<Tab[]>(() => [
	{
		label: formatMessage(messages.contentNav),
		href: worldPath.value,
		icon: BoxesIcon,
		subpages: [],
	},
	{
		label: formatMessage(messages.filesNav),
		href: `${worldPath.value}/files`,
		icon: FolderOpenIcon,
		subpages: [],
	},
	{
		label: formatMessage(messages.backupsNav),
		href: `${worldPath.value}/backups`,
		icon: DatabaseBackupIcon,
		subpages: [],
	},
])

watch(
	() => [serverFull.value, currentWorld.value, worldId.value] as const,
	([full, world, id]) => {
		if (full && id && !world) {
			router.replace(instancesPath.value)
		}
	},
	{ immediate: true },
)

function latestDate(backups: Archon.Servers.v1.WorldFull['backups']): string | null {
	let latest = 0
	let latestIso: string | null = null
	for (const backup of backups) {
		const timestamp = new Date(backup.created_at).getTime()
		if (!Number.isFinite(timestamp) || timestamp <= latest) continue
		latest = timestamp
		latestIso = backup.created_at
	}
	return latestIso
}
</script>
