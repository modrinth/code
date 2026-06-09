<template>
	<div class="flex flex-col gap-4">
		<div
			v-if="!instanceInfoAdmonitionDismissed"
			class="grid grid-cols-[1.5rem_minmax(0,1fr)_auto] items-start gap-x-3 rounded-2xl border border-solid border-brand-blue bg-bg-blue p-5 pr-4 text-contrast"
		>
			<InfoIcon class="mt-0.5 size-6 text-brand-blue" aria-hidden="true" />
			<div class="flex min-w-0 flex-col gap-1">
				<h2 class="m-0 text-xl font-bold leading-7">
					{{ formatMessage(messages.instanceInfoHeader) }}
				</h2>
				<p class="m-0 text-lg leading-7 text-contrast/85">
					{{ formatMessage(messages.instanceInfoBody) }}
				</p>
			</div>
			<ButtonStyled circular type="transparent" color="blue" hover-color-fill="background">
				<button
					type="button"
					class="mt-0.5"
					:aria-label="formatMessage(messages.instanceInfoDismiss)"
					@click="dismissInstanceInfoAdmonition"
				>
					<XIcon aria-hidden="true" />
				</button>
			</ButtonStyled>
		</div>

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
				:switching="world.type === 'world' && switchingWorldId === world.id"
				@create="handleCreateWorld"
				@edit="handleEditWorld"
				@switch="handleSwitchWorld"
				@settings="handleWorldSettings"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { InfoIcon, XIcon } from '@modrinth/assets'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { useStorage } from '@vueuse/core'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import InstanceCard from '#ui/components/servers/instances/InstanceCard.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
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
	instanceInfoHeader: {
		id: 'servers.manage.instances.info.header',
		defaultMessage: 'What is a server instance?',
	},
	instanceInfoBody: {
		id: 'servers.manage.instances.info.body',
		defaultMessage:
			'An instance is a separate setup of your server with its own content, files, worlds, and settings. You can switch which instance your server runs at any time.',
	},
	instanceInfoDismiss: {
		id: 'servers.manage.instances.info.dismiss',
		defaultMessage: "Don't show this again",
	},
	switchSuccessTitle: {
		id: 'servers.manage.instances.switch.success.title',
		defaultMessage: 'Instance switched',
	},
	switchSuccessText: {
		id: 'servers.manage.instances.switch.success.text',
		defaultMessage: '{instance} is now the active instance.',
	},
	switchError: {
		id: 'servers.manage.instances.switch.error',
		defaultMessage: 'Failed to switch instance',
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
const INSTANCE_INFO_ADMONITION_KEY = 'server-instances-info-admonition-dismissed'

const client = injectModrinthClient()
const { serverId, server, isServerRunning } = injectModrinthServerContext()
const { openServerInstanceSettings } = injectServerSettingsModal()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const router = useRouter()
const queryClient = useQueryClient()
const instanceInfoAdmonitionDismissed = useStorage(INSTANCE_INFO_ADMONITION_KEY, false)
const switchingWorldId = ref<string | null>(null)

const worldsQuery = useQuery({
	queryKey: computed(() => ['servers', 'worlds', 'summary', 'v1', serverId]),
	queryFn: loadWorldSlots,
	staleTime: 30_000,
})

const worldsPending = computed(() => worldsQuery.isLoading.value && !worldsQuery.data.value)
const worldSlots = computed(() => worldsQuery.data.value ?? [])
const switchWorldMutation = useMutation({
	mutationFn: (worldId: string) => client.archon.servers_v1.switchWorld(serverId, worldId),
	onMutate: (worldId) => {
		switchingWorldId.value = worldId
	},
	onSuccess: async (_, worldId) => {
		updateActiveWorld(worldId)
		await Promise.all([
			queryClient.invalidateQueries({
				queryKey: ['servers', 'worlds', 'summary', 'v1', serverId],
			}),
			queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
			queryClient.invalidateQueries({ queryKey: ['servers', 'v1', 'detail', serverId] }),
		])
		updateActiveWorld(worldId)
		const worldName = getWorldSlotName(worldId)
		addNotification({
			type: 'success',
			title: formatMessage(messages.switchSuccessTitle),
			text: formatMessage(messages.switchSuccessText, { instance: worldName }),
		})
	},
	onError: (error) => {
		addNotification({
			type: 'error',
			text: error instanceof Error ? error.message : formatMessage(messages.switchError),
		})
	},
	onSettled: () => {
		switchingWorldId.value = null
	},
})

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

function handleSwitchWorld(worldId: string) {
	const world = worldSlots.value.find((slot) => slot.type === 'world' && slot.id === worldId)
	if (world?.type !== 'world' || world.active || switchWorldMutation.isPending.value) return
	switchWorldMutation.mutate(worldId)
}

function handleWorldSettings(worldId: string) {
	openServerInstanceSettings({ tabId: 'general', worldId })
}

function handleCreateWorld() {
	openServerInstanceSettings({ tabId: 'installation' })
}

function dismissInstanceInfoAdmonition() {
	instanceInfoAdmonitionDismissed.value = true
}

function updateActiveWorld(worldId: string) {
	queryClient.setQueryData<WorldSlot[]>(
		['servers', 'worlds', 'summary', 'v1', serverId],
		(current) =>
			current?.map((slot) =>
				slot.type === 'world'
					? {
							...slot,
							active: slot.id === worldId,
							lastActiveAt: slot.id === worldId ? new Date().toISOString() : slot.lastActiveAt,
						}
					: slot,
			),
	)
	queryClient.setQueryData<Archon.Servers.v1.ServerFull>(
		['servers', 'v1', 'detail', serverId],
		(current) =>
			current
				? {
						...current,
						worlds: current.worlds.map((world) => ({
							...world,
							is_active: world.id === worldId,
						})),
					}
				: current,
	)
}

function getWorldSlotName(worldId: string) {
	const world = worldSlots.value.find((slot) => slot.type === 'world' && slot.id === worldId)
	return world?.name ?? formatMessage(messages.instanceSlotName, { index: 1 })
}
</script>
