<template>
	<AddServerModal
		ref="addServerModal"
		:instance="instance"
		@submit="
			(server, start) => {
				addServer(server)
				if (start) {
					joinWorld(server)
				}
			}
		"
	/>
	<EditServerModal ref="editServerModal" :instance="instance" @submit="editServer" />
	<EditWorldModal ref="editWorldModal" :instance="instance" @submit="editWorld" />
	<ConfirmModalWrapper
		ref="removeServerModal"
		:title="`Are you sure you want to remove ${serverToRemove?.name ?? 'this server'}?`"
		:description="`'${serverToRemove?.name}'${serverToRemove?.address === serverToRemove?.name ? ' ' : ` (${serverToRemove?.address})`} will be removed from your list, including in-game, and there will be no way to recover it.`"
		:markdown="false"
		@proceed="proceedRemoveServer"
	/>
	<ConfirmModalWrapper
		ref="deleteWorldModal"
		:title="`Are you sure you want to permanently delete this world?`"
		:description="`'${worldToDelete?.name}' will be **permanently deleted**, and there will be no way to recover it.`"
		@proceed="proceedDeleteWorld"
	/>
	<div v-if="dedupedWorlds.length > 0" class="flex flex-col gap-4">
		<div class="flex flex-wrap items-center gap-2">
			<StyledInput
				v-model="searchFilter"
				:icon="SearchIcon"
				type="text"
				autocomplete="off"
				:spellcheck="false"
				input-class="!h-10"
				wrapper-class="flex-1 min-w-0"
				clearable
				:placeholder="`Search ${dedupedWorlds.length} worlds...`"
			/>
			<div class="flex gap-2">
				<ButtonStyled type="outlined">
					<button
						class="!h-10 !border-button-bg !border-[1px]"
						@click="addServerModal?.show()"
					>
						<PlusIcon class="size-5" />
						Add server
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button
						class="!h-10 flex items-center gap-2"
						@click="router.push({ path: '/browse/server', query: { i: instance.path, from: 'worlds' } })"
					>
						<CompassIcon class="size-5" />
						<span>Browse servers</span>
					</button>
				</ButtonStyled>
			</div>
		</div>
		<div class="flex flex-wrap items-center justify-between gap-2">
			<div class="flex flex-wrap items-center gap-1.5">
				<FilterIcon class="size-5 text-secondary" />
				<button
					class="cursor-pointer rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]"
					:class="
						selectedFilters.length === 0
							? 'border-green bg-brand-highlight text-brand'
							: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5'
					"
					@click="selectedFilters = []"
				>
					All
				</button>
				<button
					v-for="option in filterOptions"
					:key="option.id"
					class="cursor-pointer rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]"
					:class="
						selectedFilters.includes(option.id)
							? 'border-green bg-brand-highlight text-brand'
							: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5'
					"
					@click="toggleFilter(option.id)"
				>
					{{ option.label }}
				</button>
			</div>
			<ButtonStyled type="transparent" hover-color-fill="none">
				<button :disabled="refreshingAll" @click="refreshAllWorlds">
					<RefreshCwIcon :class="refreshingAll ? 'animate-spin' : ''" />
					Refresh
				</button>
			</ButtonStyled>
		</div>
		<div class="flex flex-col w-full gap-2">
			<WorldItem
				v-for="world in filteredWorlds"
				:key="`world-${world.type}-${world.type == 'singleplayer' ? world.path : `${world.address}-${world.index}`}`"
				:world="world"
				:managed="world.type === 'server' ? isManagedServerWorld(world) : false"
				:highlighted="highlightedWorld === getWorldIdentifier(world)"
				:supports-server-quick-play="supportsServerQuickPlay"
				:supports-world-quick-play="supportsWorldQuickPlay"
				:current-protocol="protocolVersion"
				:playing-instance="playing"
				:playing-world="worldsMatch(world, worldPlaying)"
				:starting-instance="startingInstance"
				:refreshing="world.type === 'server' ? serverData[world.address]?.refreshing : undefined"
				:server-status="world.type === 'server' ? serverData[world.address]?.status : undefined"
				:rendered-motd="
					world.type === 'server' ? serverData[world.address]?.renderedMotd : undefined
				"
				:game-mode="world.type === 'singleplayer' ? GAME_MODES[world.game_mode] : undefined"
				@play="() => joinWorld(world)"
				@stop="() => emit('stop')"
				@refresh="() => refreshServer((world as ServerWorld).address)"
				@edit="
					() =>
						world.type === 'singleplayer'
							? editWorldModal?.show(world)
							: isManagedServerWorld(world)
								? undefined
								: editServerModal?.show(world)
				"
				@delete="() => !isManagedServerWorld(world) && promptToRemoveWorld(world)"
				@open-folder="(world: SingleplayerWorld) => showWorldInFolder(instance.path, world.path)"
			/>
		</div>
	</div>
	<EmptyState v-else type="empty-inbox" heading="You don't have any worlds yet.">
		<template #actions>
			<ButtonStyled type="outlined">
				<button class="!h-10 !border-button-bg !border-[1px]" @click="addServerModal?.show()">
					<PlusIcon class="size-5" />
					Add a server
				</button>
			</ButtonStyled>
			<ButtonStyled color="brand">
				<button
					class="!h-10 flex items-center gap-2"
					@click="router.push({ path: '/browse/server', query: { i: instance.path, from: 'worlds' } })"
				>
					<CompassIcon class="size-5" />
					<span>Browse servers</span>
				</button>
			</ButtonStyled>
		</template>
	</EmptyState>
</template>
<script setup lang="ts">
import { CompassIcon, FilterIcon, PlusIcon, RefreshCwIcon, SearchIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	EmptyState,
	GAME_MODES,
	type GameVersion,
	injectNotificationManager,
	StyledInput,
} from '@modrinth/ui'
import { platform } from '@tauri-apps/plugin-os'
import { computed, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import type ContextMenu from '@/components/ui/ContextMenu.vue'
import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import AddServerModal from '@/components/ui/world/modal/AddServerModal.vue'
import EditServerModal from '@/components/ui/world/modal/EditServerModal.vue'
import EditWorldModal from '@/components/ui/world/modal/EditSingleplayerWorldModal.vue'
import WorldItem from '@/components/ui/world/WorldItem.vue'
import { trackEvent } from '@/helpers/analytics'
import { get_project, get_project_v3 } from '@/helpers/cache.js'
import { profile_listener } from '@/helpers/events'
import { get_game_versions } from '@/helpers/tags'
import type { GameInstance } from '@/helpers/types'
import {
	delete_world,
	get_profile_protocol_version,
	getServerDomainKey,
	getWorldIdentifier,
	handleDefaultProfileUpdateEvent,
	hasServerQuickPlaySupport,
	hasWorldQuickPlaySupport,
	normalizeServerAddress,
	type ProfileEvent,
	type ProtocolVersion,
	refreshServerData,
	refreshServers,
	refreshWorld,
	refreshWorlds,
	remove_server_from_profile,
	resolveManagedServerWorld,
	type ServerData,
	type ServerWorld,
	showWorldInFolder,
	type SingleplayerWorld,
	sortWorlds,
	start_join_server,
	start_join_singleplayer_world,
	type World,
} from '@/helpers/worlds.ts'
import { injectServerInstall } from '@/providers/server-install'
import { ensureManagedServerWorldExists, getServerAddress } from '@/store/install'

const { handleError } = injectNotificationManager()
const { playServerProject } = injectServerInstall()
const route = useRoute()
const router = useRouter()

const addServerModal = ref<InstanceType<typeof AddServerModal>>()
const editServerModal = ref<InstanceType<typeof EditServerModal>>()
const editWorldModal = ref<InstanceType<typeof EditWorldModal>>()
const removeServerModal = ref<InstanceType<typeof ConfirmModalWrapper>>()
const deleteWorldModal = ref<InstanceType<typeof ConfirmModalWrapper>>()

const serverToRemove = ref<ServerWorld>()
const worldToDelete = ref<SingleplayerWorld>()

const emit = defineEmits<{
	(event: 'play', world: World): void
	(event: 'stop'): void
}>()

const props = defineProps<{
	instance: GameInstance
	options: InstanceType<typeof ContextMenu> | null
	offline: boolean
	playing: boolean
	installed: boolean
}>()

const instance = computed(() => props.instance)
const playing = computed(() => props.playing)

function play(world: World) {
	emit('play', world)
}

const selectedFilters = ref<string[]>([])
const searchFilter = ref('')

function toggleFilter(id: string) {
	const idx = selectedFilters.value.indexOf(id)
	if (idx >= 0) {
		selectedFilters.value.splice(idx, 1)
	} else {
		selectedFilters.value.push(id)
		if (id === 'singleplayer') {
			selectedFilters.value = selectedFilters.value.filter(
				(f) => f !== 'online' && f !== 'offline',
			)
		} else if (id === 'online' || id === 'offline') {
			selectedFilters.value = selectedFilters.value.filter((f) => f !== 'singleplayer')
		}
	}
}

const refreshingAll = ref(false)
const hadNoWorlds = ref(true)
const startingInstance = ref(false)
const worldPlaying = ref<World>()

const worlds = ref<World[]>([])
const serverData = ref<Record<string, ServerData>>({})

// Track servers_updated calls on Linux to prevent server ping spam
const MAX_LINUX_REFRESHES = 3
const isLinux = platform() === 'linux'
const linuxRefreshCount = ref(0)

const protocolVersion = ref<ProtocolVersion | null>(
	await get_profile_protocol_version(instance.value.path),
)
const managedServerName = ref<string | null>(null)
const managedServerAddress = ref<string | null>(null)

const managedServerWorld = computed(() =>
	resolveManagedServerWorld(worlds.value, managedServerName.value, managedServerAddress.value),
)

function isManagedServerWorld(world: World): world is ServerWorld {
	return world.type === 'server' && managedServerWorld.value?.index === world.index
}

async function refreshManagedServerMetadata() {
	await ensureManagedServerWorldExists(
		instance.value.path,
		managedServerName.value,
		managedServerAddress.value,
	)

	const projectId = instance.value.linked_data?.project_id
	if (!projectId) {
		managedServerName.value = null
		managedServerAddress.value = null
		return
	}

	try {
		const [project, projectV3] = await Promise.all([
			get_project(projectId, 'bypass'),
			get_project_v3(projectId, 'bypass'),
		])

		if (projectV3?.minecraft_server == null) {
			managedServerName.value = null
			managedServerAddress.value = null
			return
		}

		const serverAddress = getServerAddress(projectV3.minecraft_java_server)
		if (!serverAddress) {
			managedServerName.value = null
			managedServerAddress.value = null
			return
		}

		managedServerName.value = project.title
		managedServerAddress.value = serverAddress
	} catch (err) {
		console.error(
			`Failed to resolve managed server metadata for profile: ${instance.value.path}`,
			err,
		)
		managedServerName.value = null
		managedServerAddress.value = null
	}
}

watch(
	() => instance.value.linked_data?.project_id,
	async () => {
		await refreshManagedServerMetadata()
	},
	{ immediate: true },
)

const unlistenProfile = await profile_listener(async (e: ProfileEvent) => {
	if (e.profile_path_id !== instance.value.path) return

	console.info(`Handling profile event '${e.event}' for profile: ${e.profile_path_id}`)

	if (e.event === 'servers_updated') {
		if (isLinux && linuxRefreshCount.value >= MAX_LINUX_REFRESHES) return
		if (isLinux) linuxRefreshCount.value++

		await refreshAllWorlds()
	}

	await handleDefaultProfileUpdateEvent(worlds.value, instance.value.path, e)
})

await refreshAllWorlds()

async function refreshServer(address: string) {
	if (!serverData.value[address]) {
		serverData.value[address] = {
			refreshing: true,
		}
	}
	await refreshServerData(serverData.value[address], protocolVersion.value, address)
}

async function refreshAllWorlds() {
	if (refreshingAll.value) {
		console.log(`Already refreshing, cancelling refresh.`)
		return
	}
	await refreshManagedServerMetadata()

	refreshingAll.value = true

	worlds.value = await refreshWorlds(instance.value.path).finally(
		() => (refreshingAll.value = false),
	)
	refreshServers(worlds.value, serverData.value, protocolVersion.value)

	const hasNoWorlds = worlds.value.length === 0

	if (hadNoWorlds.value && hasNoWorlds) {
		setTimeout(() => {
			refreshingAll.value = false
		}, 1000)
	} else {
		refreshingAll.value = false
	}

	hadNoWorlds.value = hasNoWorlds
}

async function addServer(server: ServerWorld) {
	worlds.value.push(server)
	sortWorlds(worlds.value)
	await refreshServer(server.address)
}

async function editServer(server: ServerWorld) {
	const index = worlds.value.findIndex((w) => w.type === 'server' && w.index === server.index)
	if (index !== -1) {
		const oldServer = worlds.value[index] as ServerWorld
		worlds.value[index] = server
		sortWorlds(worlds.value)
		if (oldServer.address !== server.address) {
			await refreshServer(server.address)
		}
	} else {
		handleError(new Error(`Error refreshing server, refreshing all worlds`))
		await refreshAllWorlds()
	}
}

async function removeServer(server: ServerWorld) {
	await remove_server_from_profile(instance.value.path, server.index).catch(handleError)
	worlds.value = worlds.value.filter((w) => w.type !== 'server' || w.index !== server.index)
}

async function editWorld(path: string, name: string, removeIcon: boolean) {
	const world = worlds.value.find((world) => world.type === 'singleplayer' && world.path === path)
	if (world) {
		world.name = name
		if (removeIcon) {
			world.icon = undefined
		}
		sortWorlds(worlds.value)
	} else {
		handleError(new Error(`Error finding world in list, refreshing all worlds`))
		await refreshAllWorlds()
	}
}

async function deleteWorld(world: SingleplayerWorld) {
	await delete_world(instance.value.path, world.path).catch(handleError)
	worlds.value = worlds.value.filter((w) => w.type !== 'singleplayer' || w.path !== world.path)
}

function handleJoinError(err: Error) {
	handleError(err)
	startingInstance.value = false
	worldPlaying.value = undefined
}

async function joinWorld(world: World) {
	console.log(`Joining world ${getWorldIdentifier(world)}`)
	startingInstance.value = true
	worldPlaying.value = world
	if (world.type === 'server') {
		const managedProjectId = instance.value.linked_data?.project_id
		if (managedProjectId && isManagedServerWorld(world)) {
			await playServerProject(managedProjectId).catch(handleJoinError)
			trackEvent('InstanceStart', {
				loader: instance.value.loader,
				game_version: instance.value.game_version,
				source: 'WorldsPage',
			})
			startingInstance.value = false
			return
		}
		await start_join_server(instance.value.path, world.address).catch(handleJoinError)
		trackEvent('InstanceStart', {
			loader: instance.value.loader,
			game_version: instance.value.game_version,
			source: 'WorldsPage',
		})
	} else if (world.type === 'singleplayer') {
		await start_join_singleplayer_world(instance.value.path, world.path).catch(handleJoinError)
	}
	play(world)
	startingInstance.value = false
}

watch(
	() => playing.value,
	(playing) => {
		if (!playing) {
			worldPlaying.value = undefined

			setTimeout(async () => {
				for (const world of worlds.value) {
					if (world.type === 'singleplayer' && world.locked) {
						await refreshWorld(worlds.value, instance.value.path, world.path)
					}
				}
			}, 1000)
		}
	},
)

function worldsMatch(world: World, other: World | undefined) {
	if (world.type === 'server' && other?.type === 'server') {
		return world.address === other.address
	} else if (world.type === 'singleplayer' && other?.type === 'singleplayer') {
		return world.path === other.path
	}
	return false
}

const gameVersions = ref<GameVersion[]>(await get_game_versions().catch(() => []))
const supportsServerQuickPlay = computed(() =>
	hasServerQuickPlaySupport(gameVersions.value, instance.value.game_version),
)
const supportsWorldQuickPlay = computed(() =>
	hasWorldQuickPlaySupport(gameVersions.value, instance.value.game_version),
)

const dedupedWorlds = computed(() => {
	const visibleWorlds: World[] = []
	const serverIndexByDomain = new Map<string, number>()

	for (const world of worlds.value) {
		if (world.type !== 'server') {
			visibleWorlds.push(world)
			continue
		}

		const domainKey =
			getServerDomainKey(world.address) ||
			normalizeServerAddress(world.address) ||
			`server-${world.index}`
		const existingIndex = serverIndexByDomain.get(domainKey)

		if (existingIndex == null) {
			serverIndexByDomain.set(domainKey, visibleWorlds.length)
			visibleWorlds.push(world)
			continue
		}

		// replace world with managed world if applicable
		const existingWorld = visibleWorlds[existingIndex]
		if (
			existingWorld?.type === 'server' &&
			!isManagedServerWorld(existingWorld) &&
			isManagedServerWorld(world)
		) {
			visibleWorlds[existingIndex] = world
		}
	}

	return visibleWorlds
})

const filterOptions = computed(() => {
	const options: { id: string; label: string }[] = []
	const hasSingleplayer = dedupedWorlds.value.some((x) => x.type === 'singleplayer')
	const hasServer = dedupedWorlds.value.some((x) => x.type === 'server')

	const hasStatusFilter =
		selectedFilters.value.includes('online') || selectedFilters.value.includes('offline')

	if (hasSingleplayer && hasServer && !hasStatusFilter) {
		options.push({ id: 'singleplayer', label: 'Singleplayer' })
	}

	if (hasServer) {
		options.push({ id: 'vanilla', label: 'Vanilla' })
		options.push({ id: 'modded', label: 'Modded' })
		if (!selectedFilters.value.includes('singleplayer')) {
			options.push({ id: 'online', label: 'Online' })
			options.push({ id: 'offline', label: 'Offline' })
		}
	}

	return options
})

const filteredWorlds = computed(() =>
	dedupedWorlds.value.filter((x) => {
		if (searchFilter.value && !x.name.toLowerCase().includes(searchFilter.value.toLowerCase())) {
			return false
		}

		if (selectedFilters.value.length === 0) return true

		const hasSingleplayerFilter = selectedFilters.value.includes('singleplayer')
		const typeFilters = selectedFilters.value.filter((f) => f === 'vanilla' || f === 'modded')
		const statusFilters = selectedFilters.value.filter((f) => f === 'online' || f === 'offline')

		if (x.type === 'singleplayer') {
			return hasSingleplayerFilter || (typeFilters.length === 0 && statusFilters.length === 0)
		}

		if (hasSingleplayerFilter && typeFilters.length === 0 && statusFilters.length === 0) {
			return false
		}

		let passesType = true
		if (typeFilters.length > 0) {
			const isModded = isManagedServerWorld(x)
			passesType =
				(typeFilters.includes('modded') && isModded) ||
				(typeFilters.includes('vanilla') && !isModded)
		}

		let passesStatus = true
		if (statusFilters.length > 0) {
			const isOnline = !!serverData.value[x.address]?.status
			passesStatus =
				(statusFilters.includes('online') && isOnline) ||
				(statusFilters.includes('offline') && !isOnline)
		}

		return passesType && passesStatus
	}),
)

const highlightedWorld = ref(route.query.highlight)

function promptToRemoveWorld(world: World): boolean {
	if (world.type === 'server') {
		serverToRemove.value = world
		removeServerModal.value?.show()
		return !!removeServerModal.value
	} else {
		worldToDelete.value = world
		deleteWorldModal.value?.show()
		return !!deleteWorldModal.value
	}
}

async function proceedRemoveServer() {
	if (!serverToRemove.value) {
		handleError(new Error(`Error removing server, no server marked for removal.`))
		return
	}
	await removeServer(serverToRemove.value)
	serverToRemove.value = undefined
}

async function proceedDeleteWorld() {
	if (!worldToDelete.value) {
		handleError(new Error(`Error deleting world, no world marked for removal.`))
		return
	}
	await deleteWorld(worldToDelete.value)
	worldToDelete.value = undefined
}

onUnmounted(() => {
	unlistenProfile()
})

</script>
