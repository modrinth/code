<script setup lang="ts">
import { LoaderCircleIcon } from '@modrinth/assets'
import {
	Accordion,
	defineMessages,
	GAME_MODES,
	type GameVersion,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { platform } from '@tauri-apps/plugin-os'
import type { Dayjs } from 'dayjs'
import dayjs from 'dayjs'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import InstanceItem from '@/components/ui/world/InstanceItem.vue'
import WorldItem from '@/components/ui/world/WorldItem.vue'
import { useAppEvent } from '@/composables/use-app-event'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { handleSevereError } from '@/composables/use-error.js'
import { trackEvent } from '@/helpers/analytics'
import { kill, run } from '@/helpers/instance'
import { get_all } from '@/helpers/process'
import { get_game_versions } from '@/helpers/tags'
import type { GameInstance } from '@/helpers/types'
import {
	get_instance_protocol_version,
	get_recent_worlds,
	getWorldIdentifier,
	hasServerQuickPlaySupport,
	hasWorldQuickPlaySupport,
	type ProtocolVersion,
	refreshServerData,
	type ServerData,
	type ServerWorld,
	start_join_server,
	start_join_singleplayer_world,
	type WorldWithInstance,
} from '@/helpers/worlds.ts'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	jumpIn: { id: 'app.home.jump-back-in.title', defaultMessage: 'Jump in' },
	resize: { id: 'app.home.jump-back-in.resize', defaultMessage: 'Drag to resize' },
})

const props = defineProps<{
	recentInstances: GameInstance[]
}>()

const appSettings = useAppSettings()

const jumpBackInItems = ref<JumpBackInItem[]>([])
const loading = ref(true)
const serverData = ref<Record<string, ServerData>>({})
const protocolVersions = ref<Record<string, ProtocolVersion | null>>({})
const locallyPlayedInstances = ref<Record<string, Dayjs>>({})
const gameVersions = ref<GameVersion[]>(await get_game_versions().catch(() => []))

const MAX_JUMP_BACK_IN = 5
const MAX_NEW_INSTANCES = 3
const MAX_LINUX_POPULATES = 3
const ITEM_DRAG_DISTANCE = 92
const STORAGE_KEY = 'modrinth-jump-back-in-count'

const storedVisibleCount = Number.parseInt(localStorage.getItem(STORAGE_KEY) ?? '', 10)
const visibleItemLimit = ref(
	Number.isFinite(storedVisibleCount)
		? Math.max(1, Math.min(storedVisibleCount, MAX_JUMP_BACK_IN))
		: MAX_JUMP_BACK_IN,
)
const maxVisibleItems = computed(() => Math.min(jumpBackInItems.value.length, MAX_JUMP_BACK_IN))
const visibleItemCount = computed(() => Math.min(visibleItemLimit.value, maxVisibleItems.value))
const visibleJumpBackInItems = computed(() =>
	jumpBackInItems.value.slice(0, visibleItemCount.value),
)
const canResize = computed(() => maxVisibleItems.value > 1)
const resizing = ref(false)
const showOverdrag = ref(false)

function setVisibleItemLimit(count: number) {
	const clamped = Math.max(1, Math.min(count, maxVisibleItems.value))
	if (clamped >= maxVisibleItems.value) {
		visibleItemLimit.value = MAX_JUMP_BACK_IN
		localStorage.removeItem(STORAGE_KEY)
	} else {
		visibleItemLimit.value = clamped
		localStorage.setItem(STORAGE_KEY, String(clamped))
	}
}

function adjustVisibleItemLimit(delta: number) {
	const target = visibleItemCount.value + delta
	if (target < 1 || target > maxVisibleItems.value) {
		flashOverdrag()
	}
	setVisibleItemLimit(target)
}

let dragStartY = 0
let dragStartCount = 0
let wasOverdragging = false
let overdragTimeout: ReturnType<typeof setTimeout> | undefined

function clearOverdragFlash() {
	showOverdrag.value = false
	if (overdragTimeout) {
		clearTimeout(overdragTimeout)
		overdragTimeout = undefined
	}
}

function flashOverdrag() {
	showOverdrag.value = true
	if (overdragTimeout) clearTimeout(overdragTimeout)
	overdragTimeout = setTimeout(() => {
		showOverdrag.value = false
		overdragTimeout = undefined
	}, 500)
}

function onResizePointerDown(event: PointerEvent) {
	if (!canResize.value) return
	event.preventDefault()
	resizing.value = true
	wasOverdragging = false
	clearOverdragFlash()
	dragStartY = event.clientY
	dragStartCount = visibleItemCount.value
	document.body.classList.add('recent-worlds-resizing')
	const handle = event.currentTarget as HTMLElement
	handle.setPointerCapture(event.pointerId)
}

function onResizePointerMove(event: PointerEvent) {
	if (!resizing.value) return
	const target = dragStartCount + Math.round((event.clientY - dragStartY) / ITEM_DRAG_DISTANCE)
	const isOverdragging = target < 1 || target > maxVisibleItems.value
	if (isOverdragging && !wasOverdragging) {
		flashOverdrag()
	}
	wasOverdragging = isOverdragging
	setVisibleItemLimit(target)
}

function endResize(event?: PointerEvent) {
	if (!resizing.value) return
	resizing.value = false
	wasOverdragging = false
	clearOverdragFlash()
	document.body.classList.remove('recent-worlds-resizing')
	const target = event?.currentTarget as HTMLElement | undefined
	if (event && target?.hasPointerCapture(event.pointerId)) {
		target.releasePointerCapture(event.pointerId)
	}
}

// Track populate calls on Linux to prevent server ping spam
const isLinux = platform() === 'linux'
const linuxPopulateCount = ref(0)

type BaseJumpBackInItem = {
	sort_time: Dayjs
	instance: GameInstance
	newly_added?: boolean
}

type InstanceJumpBackInItem = BaseJumpBackInItem & {
	type: 'instance'
}

type WorldJumpBackInItem = BaseJumpBackInItem & {
	type: 'world'
	world: WorldWithInstance
}

type JumpBackInItem = InstanceJumpBackInItem | WorldJumpBackInItem

const showWorlds = computed(() => appSettings.getFeatureFlag('worlds_in_home'))

watch([() => props.recentInstances, () => showWorlds.value], async () => {
	await populateJumpBackIn().catch(() => {
		console.error('Failed to populate jump back in')
	})
})

await populateJumpBackIn()
	.catch(() => {
		console.error('Failed to populate jump back in')
	})
	.finally(() => {
		loading.value = false
	})

async function populateJumpBackIn() {
	// On Linux, limit automatic populates to prevent server ping spam
	if (isLinux && linuxPopulateCount.value >= MAX_LINUX_POPULATES) return
	if (isLinux) linuxPopulateCount.value++

	console.info('Repopulating jump back in...')

	const worldItems: WorldJumpBackInItem[] = []

	if (showWorlds.value) {
		const worlds = await get_recent_worlds(MAX_JUMP_BACK_IN, ['normal', 'favorite'])

		worlds.forEach((world) => {
			const instance = props.recentInstances.find((instance) => instance.id === world.instance_id)

			if (!instance || !world.last_played) {
				return
			}

			worldItems.push({
				type: 'world',
				sort_time: dayjs(world.last_played ?? 0),
				world: world,
				instance: instance,
			})
		})

		const servers: {
			instanceId: string
			address: string
		}[] = worldItems
			.filter((item) => item.world.type === 'server' && item.instance)
			.map((item) => ({
				instanceId: item.instance.id,
				address: (item.world as ServerWorld).address,
			}))

		// fetch protocol versions for all unique MC versions with server worlds
		const uniqueServerInstances = new Set<string>(servers.map((x) => x.instanceId))
		await Promise.all(
			[...uniqueServerInstances].map((instanceId) =>
				get_instance_protocol_version(instanceId)
					.then((protoVer) => (protocolVersions.value[instanceId] = protoVer))
					.catch(() => {
						console.error(`Failed to get instance protocol for: ${instanceId} `)
					}),
			),
		)

		// initialize server data
		servers.forEach(({ address }) => {
			if (!serverData.value[address]) {
				serverData.value[address] = {
					refreshing: true,
				}
			}
		})

		servers.forEach(({ instanceId, address }) =>
			refreshServerData(serverData.value[address], protocolVersions.value[instanceId], address),
		)
	}

	const instanceItems: InstanceJumpBackInItem[] = []
	for (const instance of props.recentInstances) {
		const worldItem = worldItems.find((item) => item.instance.id === instance.id)
		const lastPlayed = instance.last_played
			? dayjs(instance.last_played)
			: locallyPlayedInstances.value[instance.id]
		const newlyAdded = !lastPlayed
		if (worldItem && (!lastPlayed || !lastPlayed.isAfter(worldItem.sort_time))) continue
		if (worldItem) worldItems.splice(worldItems.indexOf(worldItem), 1)

		instanceItems.push({
			type: 'instance',
			sort_time: lastPlayed ?? dayjs(instance.created),
			instance: instance,
			newly_added: newlyAdded,
		})
	}

	const items: JumpBackInItem[] = [...worldItems, ...instanceItems]
	items.sort((a, b) => b.sort_time.diff(a.sort_time))
	let newInstanceCount = 0
	jumpBackInItems.value = items
		.filter((item) => {
			if (item.type !== 'instance' || !item.newly_added) return true
			if (newInstanceCount >= MAX_NEW_INSTANCES) return false
			newInstanceCount++
			return true
		})
		.slice(0, MAX_JUMP_BACK_IN)
}

function markInstancePlayed(item: InstanceJumpBackInItem) {
	const lastPlayed = dayjs()
	locallyPlayedInstances.value = {
		...locallyPlayedInstances.value,
		[item.instance.id]: lastPlayed,
	}
	item.sort_time = lastPlayed
	item.newly_added = false
}

function refreshServer(address: string, instanceId: string) {
	refreshServerData(serverData.value[address], protocolVersions.value[instanceId], address)
}

async function joinWorld(world: WorldWithInstance, instance?: GameInstance) {
	if (instance?.quarantined) return
	console.log(`Joining world ${getWorldIdentifier(world)}`)
	if (world.type === 'server') {
		await start_join_server(world.instance_id, world.address).catch(handleError)
		if (instance) {
			trackEvent('InstanceStart', {
				loader: instance.loader,
				game_version: instance.game_version,
				source: 'WorldItem',
			})
		}
	} else if (world.type === 'singleplayer') {
		await start_join_singleplayer_world(world.instance_id, world.path).catch(handleError)
	}
}

async function playInstance(instance: GameInstance) {
	if (instance.quarantined) return
	await run(instance.id)
		.catch((err) => handleSevereError(err, { instanceId: instance.id }))
		.finally(() => {
			trackEvent('InstanceStart', {
				loader: instance.loader,
				game_version: instance.game_version,
				source: 'WorldItem',
			})
		})
}

async function stopInstance(path: string) {
	await kill(path).catch(handleError)
	trackEvent('InstanceStop', {
		source: 'RecentWorldsList',
	})
}

const currentInstance = ref<string>()
const currentWorld = ref<string>()

useAppEvent('process', async () => {
	await checkProcesses()
})

useAppEvent('instance', async () => {
	await populateJumpBackIn().catch(() => {
		console.error('Failed to populate jump back in')
	})
})

const runningInstances = ref<string[]>([])

type ProcessMetadata = {
	uuid: string
	instance_id: string
	start_time: string
}

const checkProcesses = async () => {
	const runningProcesses: ProcessMetadata[] = await get_all().catch(handleError)

	const runningPaths = runningProcesses.map((x) => x.instance_id)

	const stoppedInstances = runningInstances.value.filter((x) => !runningPaths.includes(x))
	if (currentInstance.value && stoppedInstances.includes(currentInstance.value)) {
		currentInstance.value = undefined
		currentWorld.value = undefined
	}

	runningInstances.value = runningPaths
}

onMounted(() => {
	checkProcesses()
	linuxPopulateCount.value = 0
})

onUnmounted(() => {
	document.body.classList.remove('recent-worlds-resizing')
	clearOverdragFlash()
})
</script>

<template>
	<Accordion
		v-if="loading || jumpBackInItems.length > 0"
		open-by-default
		button-class="group mt-1 mb-3 flex w-fit cursor-pointer items-center border-0 bg-transparent p-0 text-left"
	>
		<template #title>
			<span class="flex items-center gap-1 text-2xl font-semibold leading-none text-contrast mr-1">
				{{ formatMessage(messages.jumpIn) }}
			</span>
		</template>
		<div v-if="loading" class="text-center py-4">
			<LoaderCircleIcon class="mx-auto size-8 animate-spin text-contrast" />
		</div>
		<div v-else class="grid-when-huge relative flex w-full flex-col gap-3">
			<TransitionGroup name="jump-back-in-item">
				<div
					v-for="item in visibleJumpBackInItems"
					:key="`${item.instance.id}-${item.type === 'world' ? getWorldIdentifier(item.world) : 'instance'}`"
					class="jump-back-in-item min-w-0"
				>
					<WorldItem
						v-if="item.type === 'world'"
						:world="item.world"
						:playing-instance="runningInstances.includes(item.instance.id)"
						:playing-world="
							currentInstance === item.instance.id &&
							currentWorld === getWorldIdentifier(item.world)
						"
						:refreshing="
							item.world.type === 'server'
								? serverData[item.world.address].refreshing &&
									!serverData[item.world.address].status
								: undefined
						"
						:supports-server-quick-play="
							item.world.type === 'server' &&
							hasServerQuickPlaySupport(gameVersions, item.instance.game_version || '')
						"
						:supports-world-quick-play="
							item.world.type === 'singleplayer' &&
							hasWorldQuickPlaySupport(gameVersions, item.instance.game_version || '')
						"
						:quarantined="item.instance.quarantined"
						:server-status="
							item.world.type === 'server' ? serverData[item.world.address].status : undefined
						"
						:rendered-motd="
							item.world.type === 'server' ? serverData[item.world.address].renderedMotd : undefined
						"
						:current-protocol="protocolVersions[item.instance.id]"
						:game-mode="
							item.world.type === 'singleplayer' ? GAME_MODES[item.world.game_mode] : undefined
						"
						:instance-id="item.instance.id"
						:instance-name="item.instance.name"
						:instance-icon="item.instance.icon_path"
						@refresh="
							() =>
								item.world.type === 'server'
									? refreshServer(item.world.address, item.instance.id)
									: {}
						"
						@update="() => populateJumpBackIn()"
						@play="
							() => {
								currentInstance = item.instance.id
								currentWorld = getWorldIdentifier(item.world)
								joinWorld(item.world, item.instance)
							}
						"
						@play-instance="
							() => {
								currentInstance = item.instance.id
								playInstance(item.instance)
							}
						"
						@stop="() => stopInstance(item.instance.id)"
					/>
					<InstanceItem
						v-else
						:instance="item.instance"
						:last_played="item.sort_time"
						:newly-added="item.newly_added"
						@play="() => markInstancePlayed(item)"
					/>
				</div>
			</TransitionGroup>
			<div
				v-if="canResize"
				v-tooltip="resizing ? null : formatMessage(messages.resize)"
				role="separator"
				tabindex="0"
				aria-orientation="horizontal"
				:aria-label="formatMessage(messages.resize)"
				:aria-valuemin="1"
				:aria-valuemax="maxVisibleItems"
				:aria-valuenow="visibleItemCount"
				class="group/resize relative inset-x-0 bottom-0 flex h-6 -mt-3 w-1/3 mx-auto cursor-ns-resize touch-none select-none items-center justify-center"
				@pointerdown="onResizePointerDown"
				@pointermove="onResizePointerMove"
				@pointerup="endResize"
				@pointercancel="endResize"
				@keydown.up.prevent="adjustVisibleItemLimit(-1)"
				@keydown.down.prevent="adjustVisibleItemLimit(1)"
				@keydown.home.prevent="setVisibleItemLimit(1)"
				@keydown.end.prevent="setVisibleItemLimit(maxVisibleItems)"
			>
				<div class="flex flex-col gap-0.5">
					<span
						class="h-px w-6 transition-colors"
						:class="showOverdrag ? 'bg-red' : 'bg-secondary'"
					/>
					<span
						class="h-px w-6 transition-colors"
						:class="showOverdrag ? 'bg-red' : 'bg-secondary'"
					/>
				</div>
			</div>
		</div>
	</Accordion>
</template>
<style scoped lang="scss">
.grid-when-huge {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(670px, 1fr));
}

.jump-back-in-item {
	height: 5rem;
	overflow: hidden;
}

.jump-back-in-item-enter-active,
.jump-back-in-item-leave-active {
	transition:
		opacity 0.25s ease,
		transform 0.25s ease,
		height 0.25s ease;
}

.jump-back-in-item-enter-from,
.jump-back-in-item-leave-to {
	opacity: 0;
	transform: scale(0.98);
	height: 0;
}

@media (prefers-reduced-motion: reduce) {
	.jump-back-in-item-enter-active,
	.jump-back-in-item-leave-active {
		transition: none;
	}

	.jump-back-in-item-enter-from,
	.jump-back-in-item-leave-to {
		opacity: 1;
		transform: none;
		height: 5rem;
	}
}
</style>

<style lang="scss">
body.recent-worlds-resizing,
body.recent-worlds-resizing * {
	cursor: ns-resize !important;
}
</style>
