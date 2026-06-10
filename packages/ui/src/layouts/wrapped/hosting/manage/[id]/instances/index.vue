<template>
	<div class="flex flex-col gap-4">
		<div
			v-if="worldsPending"
			class="grid grid-cols-[repeat(auto-fit,minmax(min(100%,20.25rem),1fr))] gap-6"
		>
			<div
				v-for="slot in WORLD_SLOT_COUNT"
				:key="`instance-card-skeleton-${slot}`"
				class="min-h-[19.75rem] animate-pulse rounded-2xl border border-solid border-surface-5 bg-bg-raised shadow-xl"
			/>
		</div>
		<div v-else class="grid grid-cols-[repeat(auto-fit,minmax(min(100%,20.25rem),1fr))] gap-6">
			<InstanceCard
				v-for="world in worldSlots"
				:key="world.id"
				:world="world"
				@create="handleCreateWorld"
				@edit="handleEditWorld"
				@settings="handleWorldSettings"
			/>
		</div>
	</div>

	<CreationFlowModal
		ref="createWorldModalRef"
		type="world"
		:available-loaders="SERVER_LOADERS"
		:show-snapshot-toggle="true"
		:search-modpacks="searchModpacks"
		:get-project-versions="getProjectVersions"
		:finish-disabled="!canSetup"
		:finish-disabled-tooltip="!canSetup ? permissionDeniedMessage : undefined"
		@browse-modpacks="handleBrowseModpacks"
		@create="onCreateWorld"
	/>

	<UploadProgressModal ref="uploadProgressModal" />
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, onBeforeUnmount, onMounted, useTemplateRef } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import {
	type CreationFlowContextValue,
	CreationFlowModal,
	UploadProgressModal,
} from '#ui/components'
import InstanceCard from '#ui/components/servers/instances/InstanceCard.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerPermissions } from '#ui/composables/server-permissions'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	injectServerSettingsModal,
} from '#ui/providers'
import { formatLoaderLabel } from '#ui/utils/loaders'

const messages = defineMessages({
	instanceSlotName: {
		id: 'servers.manage.instances.slot-name',
		defaultMessage: 'Instance #{index}',
	},
	createSuccessTitle: {
		id: 'servers.manage.instances.create.success.title',
		defaultMessage: 'Instance created',
	},
	createSuccessText: {
		id: 'servers.manage.instances.create.success.text',
		defaultMessage: '{instance} is ready to configure.',
	},
	createError: {
		id: 'servers.manage.instances.create.error',
		defaultMessage: 'Failed to create instance',
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
const SERVER_LOADERS = ['vanilla', 'fabric', 'neoforge', 'forge', 'quilt', 'paper', 'purpur']

const client = injectModrinthClient()
const { serverId, server, isServerRunning } = injectModrinthServerContext()
const { openServerInstanceSettings } = injectServerSettingsModal()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const router = useRouter()
const route = useRoute()
const queryClient = useQueryClient()
const { canSetup, permissionDeniedMessage } = useServerPermissions()
const createWorldModalRef =
	useTemplateRef<InstanceType<typeof CreationFlowModal>>('createWorldModalRef')
const uploadProgressModal =
	useTemplateRef<InstanceType<typeof UploadProgressModal>>('uploadProgressModal')

const worldsQuery = useQuery({
	queryKey: computed(() => ['servers', 'worlds', 'summary', 'v1', serverId]),
	queryFn: loadWorldSlots,
	staleTime: 30_000,
})

const worldsPending = computed(() => worldsQuery.isLoading.value && !worldsQuery.data.value)
const worldSlots = computed(() => worldsQuery.data.value ?? [])

onMounted(() => {
	if (route.query.resumeModal !== 'create-instance') return
	router.replace({ query: { ...route.query, resumeModal: undefined } })
	createWorldModalRef.value?.show()
})

onBeforeUnmount(() => createWorldModalRef.value?.hide())

async function searchModpacks(query: string, limit: number = 10) {
	return client.labrinth.projects_v2.search({
		query: query || undefined,
		new_filters:
			'project_types = "modpack" AND (client_side = "optional" OR client_side = "required") AND server_side = "required"',
		limit,
	})
}

async function getProjectVersions(projectId: string) {
	const versions = await client.labrinth.versions_v3.getProjectVersions(projectId)
	return versions.map((version) => ({ id: version.id }))
}

async function loadWorldSlots(): Promise<WorldSlot[]> {
	try {
		const serverFull = await client.archon.servers_v1.get(serverId)
		const slots = await Promise.all(
			serverFull.worlds.map(async (world, index) => {
				const content = await loadContentSummary(world, index)
				return toWorldSlot(world, content)
			}),
		)

		return padWorldSlots(sortWorldSlotsByCreatedAt(slots))
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
			name: formatMessage(messages.instanceSlotName, { index: i + 1 }),
		})
	}
	return padded
}

function sortWorldSlotsByCreatedAt(slots: WorldSlot[]): WorldSlot[] {
	return [...slots].sort((a, b) => getCreatedAtTimestamp(a) - getCreatedAtTimestamp(b))
}

function getCreatedAtTimestamp(slot: WorldSlot): number {
	if (slot.type !== 'world') return Number.POSITIVE_INFINITY
	const timestamp = new Date(slot.createdAt ?? '').getTime()
	return Number.isFinite(timestamp) ? timestamp : Number.POSITIVE_INFINITY
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
			name: formatMessage(messages.instanceSlotName, { index: 3 }),
		},
	]
}

function handleEditWorld(worldId: string) {
	router.push(
		`/hosting/manage/${encodeURIComponent(serverId)}/instances/${encodeURIComponent(worldId)}`,
	)
}

function handleWorldSettings(worldId: string) {
	openServerInstanceSettings({ tabId: 'general', worldId })
}

function handleCreateWorld() {
	createWorldModalRef.value?.show()
}

function handleBrowseModpacks() {
	if (!canSetup.value) return

	router.push({
		path: '/discover/modpacks',
		query: { sid: serverId, from: 'create-instance' },
	})
}

async function onCreateWorld(ctx: CreationFlowContextValue) {
	if (!canSetup.value) {
		ctx.loading.value = false
		return
	}

	const name = ctx.worldName.value.trim()
	const properties = ctx.buildProperties()
	let createdWorldId: string | null = null

	try {
		if (ctx.setupType.value === 'modpack' && ctx.modpackFile.value) {
			const createdWorld = await client.archon.servers_v1.createWorld(serverId, {
				name,
				properties,
				content: createBareWorldContent(ctx),
			})
			createdWorldId = createdWorld.id
			createWorldModalRef.value?.hide()
			const handle = client.kyros.content_v1.uploadModpackFile(
				createdWorld.id,
				ctx.modpackFile.value,
				properties,
				{ softOverride: false },
			)
			if (uploadProgressModal.value) {
				await uploadProgressModal.value.track(handle)
			} else {
				await handle.promise
			}
		} else {
			const createdWorld = await client.archon.servers_v1.createWorld(serverId, {
				name,
				properties,
				content: createWorldContent(ctx),
			})
			createdWorldId = createdWorld.id
		}

		if (!createdWorldId) {
			throw new Error(formatMessage(messages.createError))
		}

		server.value.status = 'installing'
		await handleCreateWorldSuccess(createdWorldId, name)
		createWorldModalRef.value?.hide()
	} catch (error) {
		addNotification({
			type: 'error',
			text: error instanceof Error ? error.message : formatMessage(messages.createError),
		})
		if (createdWorldId) {
			await invalidateWorldQueries()
		}
	} finally {
		ctx.loading.value = false
	}
}

function createWorldContent(ctx: CreationFlowContextValue): Archon.Servers.v1.WorldContent {
	if (ctx.setupType.value === 'modpack' && ctx.modpackSelection.value) {
		return {
			content_variant: 'modpack',
			spec: {
				platform: 'modrinth',
				project_id: ctx.modpackSelection.value.projectId,
				version_id: ctx.modpackSelection.value.versionId,
			},
		}
	}

	return createBareWorldContent(ctx)
}

function createBareWorldContent(ctx: CreationFlowContextValue): Archon.Servers.v1.WorldContent {
	const loader =
		ctx.setupType.value === 'vanilla' ? 'vanilla' : toApiLoader(ctx.selectedLoader.value)
	return {
		content_variant: 'bare',
		loader,
		version: loader === 'vanilla' ? '' : (ctx.selectedLoaderVersion.value ?? ''),
		game_version: ctx.selectedGameVersion.value ?? server.value?.mc_version ?? null,
	}
}

async function handleCreateWorldSuccess(worldId: string, name: string) {
	await invalidateWorldQueries()
	addNotification({
		type: 'success',
		title: formatMessage(messages.createSuccessTitle),
		text: formatMessage(messages.createSuccessText, { instance: name }),
	})
	await router.push(
		`/hosting/manage/${encodeURIComponent(serverId)}/instances/${encodeURIComponent(worldId)}`,
	)
}

async function invalidateWorldQueries() {
	await Promise.all([
		queryClient.invalidateQueries({
			queryKey: ['servers', 'worlds', 'summary', 'v1', serverId],
		}),
		queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
		queryClient.invalidateQueries({ queryKey: ['servers', 'v1', 'detail', serverId] }),
	])
	await queryClient.fetchQuery({
		queryKey: ['servers', 'v1', 'detail', serverId],
		queryFn: () => client.archon.servers_v1.get(serverId),
	})
}

function toApiLoader(loader: string | null | undefined): Archon.Content.v1.Modloader {
	const normalized = loader?.toLowerCase().replace(/[\s_-]/g, '') ?? 'vanilla'
	if (normalized === 'neoforge') return 'neo_forge'
	if (normalized === 'vanilla') return 'vanilla'
	return normalized as Archon.Content.v1.Modloader
}
</script>
