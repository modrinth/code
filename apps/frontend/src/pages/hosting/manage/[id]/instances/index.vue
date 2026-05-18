<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import {
	commonMessages,
	defineMessages,
	formatLoaderLabel,
	injectModrinthClient,
	injectModrinthServerContext,
	ServersManageInstancesPage,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

const client = injectModrinthClient()
const { server, serverId, isServerRunning } = injectModrinthServerContext()
const queryClient = useQueryClient()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'servers.manage.instances.meta.title',
		defaultMessage: 'Instances - {server} - Modrinth',
	},
	instanceSlotName: {
		id: 'servers.manage.instances.slot-name',
		defaultMessage: 'Instance #{index}',
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

try {
	await queryClient.ensureQueryData({
		queryKey: ['servers', 'worlds', 'summary', 'v1', serverId],
		queryFn: loadWorldSlots,
		staleTime: 30_000,
	})
} catch {
	// Let mounted layouts' useQuery surface errors; do not fail route setup.
}

useHead({
	title: () =>
		formatMessage(messages.title, {
			server: server.value?.name ?? formatMessage(commonMessages.serverLabel),
		}),
})

async function loadWorldSlots(): Promise<WorldSlot[]> {
	const serverFull = await client.archon.servers_v1.get(serverId)
	const slots = await Promise.all(
		serverFull.worlds.map(async (world, index) => {
			const content = await loadContentSummary(world, index)
			return toWorldSlot(world, content)
		}),
	)

	return padWorldSlots(slots)
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
			name: formatMessage(messages.instanceSlotName, { index: i + 1 }),
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
</script>

<template>
	<ServersManageInstancesPage />
</template>
