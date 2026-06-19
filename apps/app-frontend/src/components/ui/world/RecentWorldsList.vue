<script setup lang="ts">
import { LoaderCircleIcon } from '@modrinth/assets'
import type { GameVersion } from '@modrinth/ui'
import { GAME_MODES, HeadingLink, injectNotificationManager } from '@modrinth/ui'
import { platform } from '@tauri-apps/plugin-os'
import type { Dayjs } from 'dayjs'
import dayjs from 'dayjs'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import InstanceItem from '@/components/ui/world/InstanceItem.vue'
import WorldItem from '@/components/ui/world/WorldItem.vue'
import { trackEvent } from '@/helpers/analytics'
import { instance_listener, process_listener } from '@/helpers/events'
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
import { handleSevereError } from '@/store/error'
import { useTheming } from '@/store/theme.ts'

const { handleError } = injectNotificationManager()

const props = defineProps<{
	recentInstances: GameInstance[]
}>()

const theme = useTheming()

const jumpBackInItems = ref<JumpBackInItem[]>([])
const loading = ref(true)
const serverData = ref<Record<string, ServerData>>({})
const protocolVersions = ref<Record<string, ProtocolVersion | null>>({})
const gameVersions = ref<GameVersion[]>(await get_game_versions().catch(() => []))

const MIN_JUMP_BACK_IN = 3
const MAX_JUMP_BACK_IN = 6
const TWO_WEEKS_AGO = dayjs().subtract(14, 'day')
const MAX_LINUX_POPULATES = 3

// Track populate calls on Linux to prevent server ping spam
const isLinux = platform() === 'linux'
const linuxPopulateCount = ref(0)

type BaseJumpBackInItem = {
	last_played: Dayjs
	instance: GameInstance
}

type InstanceJumpBackInItem = BaseJumpBackInItem & {
	type: 'instance'
}

type WorldJumpBackInItem = BaseJumpBackInItem & {
	type: 'world'
	world: WorldWithInstance
}

type JumpBackInItem = InstanceJumpBackInItem | WorldJumpBackInItem

const showWorlds = computed(() => theme.getFeatureFlag('worlds_in_home'))

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
				last_played: dayjs(world.last_played ?? 0),
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
		if ((worldItem && worldItem.last_played.isAfter(TWO_WEEKS_AGO)) || !instance.last_played) {
			continue
		}

		instanceItems.push({
			type: 'instance',
			last_played: dayjs(instance.last_played ?? 0),
			instance: instance,
		})
	}

	const items: JumpBackInItem[] = [...worldItems, ...instanceItems]
	items.sort((a, b) => dayjs(b.last_played ?? 0).diff(dayjs(a.last_played ?? 0)))
	jumpBackInItems.value = items
		.filter((item, index) => index < MIN_JUMP_BACK_IN || item.last_played.isAfter(TWO_WEEKS_AGO))
		.slice(0, MAX_JUMP_BACK_IN)
}

function refreshServer(address: string, instanceId: string) {
	refreshServerData(serverData.value[address], protocolVersions.value[instanceId], address)
}

async function joinWorld(world: WorldWithInstance, instance?: GameInstance) {
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

const unlistenProcesses = await process_listener(async () => {
	await checkProcesses()
})

const unlistenInstances = await instance_listener(async () => {
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
	unlistenProcesses()
	unlistenInstances()
})
</script>

<template>
	<div v-if="loading" class="flex flex-col gap-2">
		<span class="flex mt-1 mb-3 leading-none items-center gap-1 text-primary text-lg font-bold">
			Jump back in
		</span>
		<div class="text-center py-4">
			<LoaderCircleIcon class="mx-auto size-8 animate-spin text-contrast" />
		</div>
	</div>
	<div v-else-if="jumpBackInItems.length > 0" class="flex flex-col gap-2">
		<HeadingLink v-if="theme.getFeatureFlag('worlds_tab')" to="/worlds" class="mt-1">
			Jump back in
		</HeadingLink>
		<span
			v-else
			class="flex mt-1 mb-3 leading-none items-center gap-1 text-primary text-lg font-bold"
		>
			Jump back in
		</span>
		<div class="grid-when-huge flex flex-col w-full gap-2">
			<template
				v-for="item in jumpBackInItems"
				:key="`${item.instance.id}-${item.type === 'world' ? getWorldIdentifier(item.world) : 'instance'}`"
			>
				<WorldItem
					v-if="item.type === 'world'"
					:world="item.world"
					:playing-instance="runningInstances.includes(item.instance.id)"
					:playing-world="
						currentInstance === item.instance.id && currentWorld === getWorldIdentifier(item.world)
					"
					:refreshing="
						item.world.type === 'server'
							? serverData[item.world.address].refreshing && !serverData[item.world.address].status
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
				<InstanceItem v-else :instance="item.instance" :last_played="item.last_played" />
			</template>
		</div>
	</div>
</template>
<style scoped lang="scss">
.grid-when-huge {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(670px, 1fr));
}
</style>
