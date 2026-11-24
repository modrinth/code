<script setup lang="ts">
import { LoaderCircleIcon } from '@modrinth/assets'
import {
	Accordion,
	defineMessages,
	GAME_MODES,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import type { Dayjs } from 'dayjs'
import dayjs from 'dayjs'
import { computed, ref, watch } from 'vue'

import { useAppEvent } from '@/composables/use-app-event'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { handleSevereError } from '@/composables/use-error.js'
import { instanceKeys, processKeys, processListQueryOptions } from '@/features/instances/queries'
import {
	gameVersionsQueryOptions,
	recentWorldsQueryOptions,
	worldKeys,
} from '@/features/worlds/queries'
import { useRecentWorldsRefresh } from '@/features/worlds/use-recent-worlds-refresh'
import {
	getWorldIdentifier,
	hasServerQuickPlaySupport,
	hasWorldQuickPlaySupport,
} from '@/features/worlds/world-utils'
import WorldItem from '@/features/worlds/WorldItem.vue'
import { trackEvent } from '@/helpers/analytics'
import { kill, run } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import {
	setWorldDisplayStatus,
	startJoinServer,
	startJoinSingleplayerWorld,
} from '@/platform/app-lib/worlds/commands'
import type { WorldWithInstance } from '@/platform/app-lib/worlds/types'

import InstanceItem from './instance-item.vue'
import { useJumpBackInResize } from './use-jump-back-in-resize'
import { useJumpBackInServers } from './use-jump-back-in-servers'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	jumpIn: { id: 'app.home.jump-back-in.title', defaultMessage: 'Jump in' },
	resize: { id: 'app.home.jump-back-in.resize', defaultMessage: 'Drag to resize' },
})

const props = defineProps<{
	recentInstances: GameInstance[]
}>()

const MAX_JUMP_BACK_IN = 5
const MAX_NEW_INSTANCES = 3

const appSettings = useAppSettings()
const queryClient = useQueryClient()
const gameVersionsQuery = useQuery(gameVersionsQueryOptions())
const gameVersions = computed(() => gameVersionsQuery.data.value ?? [])

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

const showWorlds = computed(() => appSettings.showJumpIn)

const recentWorldsOptions = recentWorldsQueryOptions(MAX_JUMP_BACK_IN)
const recentWorldsQuery = useQuery({
	...recentWorldsOptions,
	queryKey: [...recentWorldsOptions.queryKey] as const,
	enabled: showWorlds,
})
const loading = computed(() => showWorlds.value && recentWorldsQuery.isLoading.value)
useRecentWorldsRefresh(showWorlds)

function isSameRecentWorld(world: WorldWithInstance, other: WorldWithInstance) {
	return (
		world.instance_id === other.instance_id &&
		world.type === other.type &&
		getWorldIdentifier(world) === getWorldIdentifier(other)
	)
}

const hideFromHomeMutation = useMutation({
	mutationFn: (world: WorldWithInstance) =>
		setWorldDisplayStatus(world.instance_id, world.type, getWorldIdentifier(world), 'hidden'),
	networkMode: 'always',
	onMutate: async (world) => {
		await queryClient.cancelQueries({ queryKey: worldKeys.recent() })
		const previous = queryClient.getQueriesData<WorldWithInstance[]>({
			queryKey: worldKeys.recent(),
		})
		queryClient.setQueriesData<WorldWithInstance[]>({ queryKey: worldKeys.recent() }, (worlds) =>
			worlds?.filter((existing) => !isSameRecentWorld(existing, world)),
		)
		return { previous }
	},
	onError: (error, _world, context) => {
		for (const [key, data] of context?.previous ?? []) queryClient.setQueryData(key, data)
		handleError(error)
	},
	onSettled: () => queryClient.invalidateQueries({ queryKey: worldKeys.recent() }),
})
const worldItems = computed<WorldJumpBackInItem[]>(() => {
	if (!showWorlds.value) return []
	return (recentWorldsQuery.data.value ?? []).flatMap<WorldJumpBackInItem>((world) => {
		const instance = props.recentInstances.find((instance) => instance.id === world.instance_id)
		if (!instance || !world.last_played) return []
		return [{ type: 'world', sort_time: dayjs(world.last_played), world, instance }]
	})
})

const jumpBackInItems = computed<JumpBackInItem[]>(() => {
	const worlds = [...worldItems.value]
	const instanceItems: InstanceJumpBackInItem[] = []
	for (const instance of props.recentInstances) {
		const worldItem = worlds.find((item) => item.instance.id === instance.id)
		const lastPlayed = instance.last_played ? dayjs(instance.last_played) : undefined
		const newlyAdded = !lastPlayed
		if (worldItem && (!lastPlayed || !lastPlayed.isAfter(worldItem.sort_time))) continue
		if (worldItem) worlds.splice(worlds.indexOf(worldItem), 1)

		instanceItems.push({
			type: 'instance',
			sort_time: lastPlayed ?? dayjs(instance.created),
			instance: instance,
			newly_added: newlyAdded,
		})
	}

	const items: JumpBackInItem[] = [...worlds, ...instanceItems]
	items.sort((a, b) => b.sort_time.diff(a.sort_time))
	let newInstanceCount = 0
	return items
		.filter((item) => {
			if (item.type !== 'instance' || !item.newly_added) return true
			if (newInstanceCount >= MAX_NEW_INSTANCES) return false
			newInstanceCount++
			return true
		})
		.slice(0, MAX_JUMP_BACK_IN)
})

const itemsGrid = ref<HTMLElement>()
const {
	minVisibleItems,
	maxVisibleItems,
	visibleItemCount,
	canResize,
	resizing,
	showOverdrag,
	setVisibleItemLimit,
	adjustVisibleItemLimit,
	onResizePointerDown,
	onResizePointerMove,
	endResize,
} = useJumpBackInResize({
	grid: itemsGrid,
	itemCount: () => jumpBackInItems.value.length,
	maxItems: MAX_JUMP_BACK_IN,
})
const visibleJumpBackInItems = computed(() =>
	jumpBackInItems.value.slice(0, visibleItemCount.value),
)

function markInstancePlayed(item: InstanceJumpBackInItem) {
	const lastPlayed = new Date()
	queryClient.setQueryData<GameInstance[]>(instanceKeys.list(), (instances) =>
		instances?.map((instance) =>
			instance.id === item.instance.id ? { ...instance, last_played: lastPlayed } : instance,
		),
	)
}

const { protocolVersions, serverData, refreshServer } = useJumpBackInServers(
	() => jumpBackInItems.value,
)

watch(recentWorldsQuery.error, (error) => {
	if (error) handleError(error)
})

async function joinWorld(world: WorldWithInstance, instance?: GameInstance) {
	if (instance?.launch_blocked) return
	if (world.type === 'server') {
		await startJoinServer(world.instance_id, world.address).catch(handleError)
		if (instance) {
			trackEvent('InstanceStart', {
				loader: instance.loader,
				game_version: instance.game_version,
				source: 'WorldItem',
			})
		}
	} else if (world.type === 'singleplayer') {
		await startJoinSingleplayerWorld(world.instance_id, world.path).catch(handleError)
	}
}

async function playInstance(instance: GameInstance) {
	if (instance.launch_blocked) return
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

const processesQuery = useQuery(processListQueryOptions())
const runningInstances = computed(() =>
	(processesQuery.data.value ?? []).map((process) => process.instance_id),
)
watch(runningInstances, (running, previous) => {
	if (
		currentInstance.value &&
		previous.includes(currentInstance.value) &&
		!running.includes(currentInstance.value)
	) {
		currentInstance.value = undefined
		currentWorld.value = undefined
	}
})
useAppEvent('process', () => {
	void queryClient.invalidateQueries({ queryKey: processKeys.list() }, { cancelRefetch: false })
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
			<LoaderCircleIcon aria-hidden="true" class="mx-auto size-8 animate-spin text-contrast" />
		</div>
		<div v-else ref="itemsGrid" class="grid-when-huge relative flex w-full flex-col gap-3">
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
								? (serverData[item.world.address]?.refreshing ?? true) &&
									!serverData[item.world.address]?.status
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
						:launch-blocked="item.instance.launch_blocked"
						:server-status="
							item.world.type === 'server' ? serverData[item.world.address]?.status : undefined
						"
						:rendered-motd="
							item.world.type === 'server'
								? serverData[item.world.address]?.renderedMotd
								: undefined
						"
						:current-protocol="protocolVersions[item.instance.id]"
						:game-mode="
							item.world.type === 'singleplayer' ? GAME_MODES[item.world.game_mode] : undefined
						"
						:instance-id="item.instance.id"
						:instance-name="item.instance.name"
						:instance-icon="item.instance.icon_path"
						@refresh="() => (item.world.type === 'server' ? refreshServer(item.world.address) : {})"
						@hide-from-home="hideFromHomeMutation.mutate(item.world)"
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
				:aria-valuemin="minVisibleItems"
				:aria-valuemax="maxVisibleItems"
				:aria-valuenow="visibleItemCount"
				class="group/resize relative col-span-full inset-x-0 bottom-0 flex h-6 -mt-3 w-1/3 mx-auto cursor-ns-resize touch-none select-none items-center justify-center"
				@pointerdown="onResizePointerDown"
				@pointermove="onResizePointerMove"
				@pointerup="endResize"
				@pointercancel="endResize"
				@keydown.up.prevent="adjustVisibleItemLimit(-1)"
				@keydown.down.prevent="adjustVisibleItemLimit(1)"
				@keydown.home.prevent="setVisibleItemLimit(minVisibleItems)"
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
