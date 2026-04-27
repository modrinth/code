<template>
	<div class="flex flex-col gap-4">
		<div
			v-if="worldsPending"
			class="grid grid-cols-[repeat(auto-fit,minmax(min(100%,20.25rem),1fr))] gap-6"
		>
			<div
				v-for="slot in WORLD_SLOT_COUNT"
				:key="`world-card-skeleton-${slot}`"
				class="min-h-[19.75rem] animate-pulse rounded-2xl border border-solid border-surface-5 bg-bg-raised shadow-xl"
			/>
		</div>
		<div v-else class="grid grid-cols-[repeat(auto-fit,minmax(min(100%,20.25rem),1fr))] gap-6">
			<WorldCard
				v-for="world in worldSlots"
				:key="world.id"
				:world="world"
				@create="handleCreateWorld"
				@edit="handleEditWorld"
				@settings="handleWorldSettings"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'
import { useRouter } from 'vue-router'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectServerSettingsModal,
} from '#ui/providers'
import { formatLoaderLabel } from '#ui/utils/loaders'

import WorldCard from './components/WorldCard.vue'

const messages = defineMessages({
	worldSlotName: {
		id: 'servers.manage.worlds.slot-name',
		defaultMessage: 'World #{index}',
	},
})

type LinkedModpack = {
	name: string
	iconUrl: string | null
	link: string | null
}

type WorldSlot =
	| {
			type: 'world'
			id: string
			name: string
			active: boolean
			gameVersion: string | null
			loaderLabel: string | null
			linkedModpack: LinkedModpack | null
			installedContentCount: number | null
			lastActiveAt: string | null
			createdAt: string | null
	  }
	| {
			type: 'empty'
			id: string
			name: string
	  }

type ContentSummary = {
	gameVersion: string | null
	loader: string | null
	loaderVersion: string | null
	linkedModpack: LinkedModpack | null
	installedContentCount: number | null
}

const WORLD_SLOT_COUNT = 3

const client = injectModrinthClient()
const { serverId, server, isServerRunning } = injectModrinthServerContext()
const { openServerSettings } = injectServerSettingsModal()
const { formatMessage } = useVIntl()
const router = useRouter()

const worldsQuery = useQuery({
	queryKey: computed(() => ['servers', 'worlds', 'summary', 'v1', serverId]),
	queryFn: loadWorldSlots,
	staleTime: 30_000,
})

const worldsPending = computed(() => worldsQuery.isLoading.value && !worldsQuery.data.value)
const worldSlots = computed(() => worldsQuery.data.value ?? [])

async function loadWorldSlots(): Promise<WorldSlot[]> {
	try {
		const serverFull = await client.archon.servers_v1.get(serverId)
		const slots = await Promise.all(
			serverFull.worlds.map(async (world, index) => {
				const content = await loadContentSummary(world, index)
				return toWorldSlot(world, content)
			}),
		)

		return padWorldSlots(slots)
	} catch {
		return createDummyWorldSlots()
	}
}

async function loadContentSummary(
	world: Archon.Servers.v1.WorldFull,
	index: number,
): Promise<ContentSummary> {
	try {
		const content = await client.archon.content_v1.getAddons(serverId, world.id, {
			addons: true,
			updates: false,
		})

		return {
			gameVersion: content.game_version ?? world.content?.game_version ?? null,
			loader: content.modloader ?? world.content?.modloader ?? null,
			loaderVersion: content.modloader_version ?? world.content?.modloader_version ?? null,
			linkedModpack: getLinkedModpack(content.modpack),
			installedContentCount: content.addons?.length ?? 0,
		}
	} catch {
		return createDummyContentSummary(world, index)
	}
}

function toWorldSlot(world: Archon.Servers.v1.WorldFull, content: ContentSummary): WorldSlot {
	return {
		type: 'world',
		id: world.id,
		name: world.name,
		active: world.is_active,
		gameVersion: content.gameVersion,
		loaderLabel: getLoaderLabel(content.loader, content.loaderVersion),
		linkedModpack: content.linkedModpack,
		installedContentCount: content.installedContentCount,
		lastActiveAt: getLatestKnownActivity(world),
		createdAt: world.created_at,
	}
}

function padWorldSlots(slots: WorldSlot[]): WorldSlot[] {
	const padded = [...slots]
	for (let i = padded.length; i < WORLD_SLOT_COUNT; i++) {
		padded.push({
			type: 'empty',
			id: `empty-world-slot-${i + 1}`,
			name: formatMessage(messages.worldSlotName, { index: i + 1 }),
		})
	}
	return padded
}

function getLinkedModpack(modpack: Archon.Content.v1.ModpackFields | null): LinkedModpack | null {
	if (!modpack) return null

	const name =
		modpack.title ??
		(modpack.spec.platform === 'local_file' ? modpack.spec.name : modpack.spec.project_id)

	return {
		name,
		iconUrl: modpack.icon_url ?? null,
		link: modpack.spec.platform === 'modrinth' ? `/project/${modpack.spec.project_id}` : null,
	}
}

function getLoaderLabel(loader: string | null, loaderVersion: string | null): string | null {
	if (!loader) return null
	const normalizedLoader = loader.toLowerCase()
	return [formatLoaderLabel(normalizedLoader), loaderVersion].filter(Boolean).join(' ')
}

function getLatestKnownActivity(world: Archon.Servers.v1.WorldFull): string | null {
	const latestBackup = latestDate(world.backups.map((backup) => backup.created_at))
	if (latestBackup) return latestBackup
	if (world.is_active && isServerRunning.value) return new Date().toISOString()
	return world.created_at ?? null
}

function latestDate(dates: string[]): string | null {
	let latest = 0
	let latestIso: string | null = null
	for (const date of dates) {
		const timestamp = new Date(date).getTime()
		if (!Number.isFinite(timestamp) || timestamp <= latest) continue
		latest = timestamp
		latestIso = date
	}
	return latestIso
}

function createDummyContentSummary(
	world: Archon.Servers.v1.WorldFull,
	index: number,
): ContentSummary {
	const gameVersion = world.content?.game_version ?? server.value?.mc_version ?? '1.20.4'
	const loader = world.content?.modloader ?? server.value?.loader?.toLowerCase() ?? 'fabric'
	const loaderVersion = world.content?.modloader_version ?? server.value?.loader_version ?? '0.16.6'

	if (index === 0) {
		return {
			gameVersion,
			loader,
			loaderVersion,
			linkedModpack: {
				name: 'Cobblemon Official',
				iconUrl: null,
				link: null,
			},
			installedContentCount: 47,
		}
	}

	return {
		gameVersion,
		loader,
		loaderVersion,
		linkedModpack: null,
		installedContentCount: 13,
	}
}

function createDummyWorldSlots(): WorldSlot[] {
	const now = Date.now()
	const twoHoursAgo = new Date(now - 2 * 60 * 60 * 1000).toISOString()
	const sixteenDaysAgo = new Date(now - 16 * 24 * 60 * 60 * 1000).toISOString()
	const createdAt = new Date(now - 197 * 24 * 60 * 60 * 1000).toISOString()

	return [
		{
			type: 'world',
			id: 'dummy-world-1',
			name: 'Cobbletown',
			active: true,
			gameVersion: '1.20.4',
			loaderLabel: 'Fabric 0.16.6',
			linkedModpack: {
				name: 'Cobblemon Official',
				iconUrl: null,
				link: null,
			},
			installedContentCount: 47,
			lastActiveAt: twoHoursAgo,
			createdAt,
		},
		{
			type: 'world',
			id: 'dummy-world-2',
			name: 'SMP Season 4',
			active: false,
			gameVersion: '1.20.4',
			loaderLabel: 'Fabric 0.16.6',
			linkedModpack: null,
			installedContentCount: 13,
			lastActiveAt: sixteenDaysAgo,
			createdAt,
		},
		{
			type: 'empty',
			id: 'empty-world-slot-3',
			name: formatMessage(messages.worldSlotName, { index: 3 }),
		},
	]
}

function handleEditWorld(worldId: string) {
	router.push(
		`/hosting/manage/${encodeURIComponent(serverId)}/worlds/${encodeURIComponent(worldId)}`,
	)
}

function handleWorldSettings() {
	openServerSettings({ tabId: 'installation' })
}

function handleCreateWorld() {
	openServerSettings({ tabId: 'installation' })
}
</script>
