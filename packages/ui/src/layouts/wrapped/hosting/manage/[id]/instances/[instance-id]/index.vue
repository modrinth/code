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
			:actions="headerActions"
		/>

		<NavTabs :links="worldTabLinks" replace />

		<slot />
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import {
	BoxesIcon,
	DatabaseBackupIcon,
	FolderOpenIcon,
	LoaderCircleIcon,
	PlayIcon,
	SettingsIcon,
	SlashIcon,
	StopCircleIcon,
	UpdatedIcon,
} from '@modrinth/assets'
import { useQuery } from '@tanstack/vue-query'
import { computed, watch } from 'vue'
import { useRouter } from 'vue-router'

import NavTabs from '#ui/components/base/NavTabs.vue'
import { WorldManageHeader } from '#ui/components/servers/server-header'
import { useServerPowerAction } from '#ui/components/servers/server-header/use-server-power-action'
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
const {
	isInstalling,
	isStopping,
	showRestartButton,
	busyTooltip,
	canTakeAction,
	canKill,
	primaryActionText,
	initiateAction,
	handlePrimaryAction,
} = useServerPowerAction()

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
	() => currentWorld.value?.name ?? formatMessage(messages.worldFallbackName),
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
const startActionText = computed(() =>
	primaryActionText.value === 'Start' ? 'Start instance' : primaryActionText.value,
)
const stopSplitActions = computed(() => [
	{
		id: 'stop',
		label: isStopping.value ? 'Stopping' : 'Stop',
		icon: StopCircleIcon,
		action: () => initiateAction('Stop'),
	},
	{
		id: 'kill_server',
		label: 'Kill server',
		icon: SlashIcon,
		action: () => initiateAction('Kill'),
	},
])
const powerActions = computed(() => {
	if (isInstalling.value) {
		return [
			{
				id: 'installing',
				label: 'Installing...',
				icon: LoaderCircleIcon,
				iconClass: 'animate-spin',
				color: 'brand' as const,
				disabled: true,
			},
		]
	}
	if (showRestartButton.value) {
		return [
			{
				id: 'restart',
				label: primaryActionText.value,
				icon: UpdatedIcon,
				color: 'orange' as const,
				tooltip: busyTooltip.value,
				disabled: !canTakeAction.value,
				onClick: handlePrimaryAction,
			},
			{
				id: 'stop',
				label: 'Stop instance',
				color: 'red' as const,
				joinedActions: stopSplitActions.value,
				primaryDisabled: !canTakeAction.value,
				dropdownDisabled: !canKill.value,
			},
		]
	}
	if (isStopping.value) {
		return [
			{
				id: 'stop',
				label: 'Stop instance',
				color: 'red' as const,
				joinedActions: stopSplitActions.value,
				primaryDisabled: true,
				dropdownDisabled: !canKill.value,
				primaryMuted: true,
			},
		]
	}
	return [
		{
			id: 'start',
			label: startActionText.value,
			icon: PlayIcon,
			color: 'brand' as const,
			tooltip: busyTooltip.value,
			disabled: !canTakeAction.value,
			onClick: handlePrimaryAction,
		},
	]
})
const headerActions = computed(() => [
	...powerActions.value,
	{
		id: 'settings',
		label: formatMessage(messages.instanceSettings),
		icon: SettingsIcon,
		labelHidden: true,
		tooltip: formatMessage(messages.instanceSettings),
		onClick: () => openServerSettings({ tabId: 'installation' }),
	},
])

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
