<template>
	<div class="flex min-h-[36rem] flex-col gap-6 text-primary">
		<div class="flex flex-col gap-2">
			<RouterLink
				:to="worldsPath"
				class="flex w-fit items-center gap-1 text-base font-medium text-blue hover:underline"
			>
				<ChevronLeftIcon class="size-4" aria-hidden="true" />
				{{ formatMessage(messages.allWorlds) }}
			</RouterLink>

			<div class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
				<div class="flex min-w-0 flex-col gap-1">
					<h1 class="m-0 truncate text-2xl font-semibold leading-8 text-contrast">
						{{ worldName }}
					</h1>
					<div class="flex flex-wrap items-center gap-2 text-base font-medium text-secondary">
						<template v-for="(item, index) in worldMetadata" :key="item">
							<span>{{ item }}</span>
							<BulletDivider v-if="index < worldMetadata.length - 1" />
						</template>
					</div>
				</div>

				<div class="flex shrink-0 items-center gap-2">
					<PanelServerActionButton size="standard" start-label="Start world" />
					<ButtonStyled circular>
						<button
							v-tooltip="formatMessage(messages.worldSettings)"
							@click="openServerSettings({ tabId: 'installation' })"
						>
							<SettingsIcon aria-hidden="true" />
						</button>
					</ButtonStyled>
				</div>
			</div>
		</div>

		<div class="h-px w-full bg-surface-5" />

		<NavTabs :links="worldTabLinks" replace />

		<slot />
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import {
	BoxesIcon,
	ChevronLeftIcon,
	DatabaseBackupIcon,
	FolderOpenIcon,
	SettingsIcon,
} from '@modrinth/assets'
import { useQuery } from '@tanstack/vue-query'
import { computed, watch } from 'vue'
import { RouterLink, useRouter } from 'vue-router'

import BulletDivider from '#ui/components/base/BulletDivider.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NavTabs from '#ui/components/base/NavTabs.vue'
import { PanelServerActionButton } from '#ui/components/servers/server-header'
import { useRelativeTime } from '#ui/composables'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectServerSettingsModal,
} from '#ui/providers'
import { formatLoaderLabel } from '#ui/utils/loaders'

interface Tab {
	label: string
	href: string
	icon?: object
	subpages?: string[]
}

const messages = defineMessages({
	allWorlds: {
		id: 'servers.manage.world.all-worlds',
		defaultMessage: 'All worlds',
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
		id: 'servers.manage.world.fallback-name',
		defaultMessage: 'World',
	},
	lastActive: {
		id: 'servers.manage.world.last-active',
		defaultMessage: 'Last active {time}',
	},
	worldSettings: {
		id: 'servers.manage.world.settings',
		defaultMessage: 'World settings',
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

const worldsPath = computed(() => `/hosting/manage/${encodeURIComponent(serverId)}/worlds`)
const worldPath = computed(() =>
	worldId.value ? `${worldsPath.value}/${encodeURIComponent(worldId.value)}` : worldsPath.value,
)

const currentWorld = computed(() => {
	const id = worldId.value
	if (!id) return null
	return serverFull.value?.worlds.find((world) => world.id === id) ?? null
})

const worldName = computed(
	() => currentWorld.value?.name ?? server.value?.name ?? formatMessage(messages.worldFallbackName),
)

const worldMetadata = computed(() =>
	[gameVersionLabel.value, loaderLabel.value, lastActiveLabel.value].filter(
		(item): item is string => !!item,
	),
)

const gameVersionLabel = computed(() => {
	const version = currentWorld.value?.content?.game_version ?? server.value?.mc_version
	return version ? `MC ${version}` : null
})

const loaderLabel = computed(() => {
	const loader = currentWorld.value?.content?.modloader ?? server.value?.loader?.toLowerCase()
	if (!loader) return null
	const loaderVersion =
		currentWorld.value?.content?.modloader_version ?? server.value?.loader_version ?? null
	return [formatLoaderLabel(loader), loaderVersion].filter(Boolean).join(' ')
})

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
			router.replace(worldsPath.value)
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
