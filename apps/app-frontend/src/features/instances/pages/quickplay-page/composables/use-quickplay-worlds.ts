import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, onScopeDispose, watch } from 'vue'

import { useAppEvent } from '@/composables/use-app-event'
import { getServerAddress, normalizeServerAddress } from '@/features/servers/address'
import {
	ensureManagedServerWorldExists,
	resolveManagedServerWorld,
} from '@/features/worlds/managed-server'
import { instanceWorldsQueryOptions, worldKeys } from '@/features/worlds/queries'
import { sortWorlds } from '@/features/worlds/world-utils'
import { injectInstancePage } from '@/pages/instance/instance-context'
import { getSingleplayerWorld } from '@/platform/app-lib/worlds/commands'
import type { ServerWorld, SingleplayerWorld, World } from '@/platform/app-lib/worlds/types'

const LOCKED_WORLD_REFRESH_DELAY_MS = 1000

function dedupeServers(worlds: World[], isManaged: (world: World) => boolean) {
	const visibleWorlds: World[] = []
	const serverIndexByAddress = new Map<string, number>()

	for (const world of worlds) {
		if (world.type !== 'server') {
			visibleWorlds.push(world)
			continue
		}

		const addressKey = normalizeServerAddress(world.address) || `server-${world.index}`
		const existingIndex = serverIndexByAddress.get(addressKey)

		if (existingIndex == null) {
			serverIndexByAddress.set(addressKey, visibleWorlds.length)
			visibleWorlds.push(world)
			continue
		}

		const existingWorld = visibleWorlds[existingIndex]
		if (existingWorld && !isManaged(existingWorld) && isManaged(world)) {
			visibleWorlds[existingIndex] = world
		}
	}

	return visibleWorlds
}

function matchesJoinedServer(world: ServerWorld, host: string, port: number) {
	return world.address === `${host}:${port}` || (port === 25565 && world.address === host)
}

export function useQuickplayWorlds() {
	const instancePage = injectInstancePage()
	const queryClient = useQueryClient()

	const instanceId = instancePage.instanceId
	const worldsKey = computed(() => worldKeys.instance(instanceId.value))

	const worldsQuery = useQuery(
		computed(() => ({
			...instanceWorldsQueryOptions(instanceId.value),
			enabled: !!instanceId.value,
		})),
	)

	const worlds = computed(() => worldsQuery.data.value ?? [])

	const managedServer = computed(() => {
		const project = instancePage.linkedProject.value
		if (project?.minecraft_server == null) return null
		const address = getServerAddress(project.minecraft_java_server)
		return address ? { name: project.name, address } : null
	})

	const managedServerWorld = computed(() =>
		resolveManagedServerWorld(
			worlds.value,
			managedServer.value?.name,
			managedServer.value?.address,
		),
	)

	function isManagedServerWorld(world: World): world is ServerWorld {
		return world.type === 'server' && managedServerWorld.value?.index === world.index
	}

	const visibleWorlds = computed(() => dedupeServers(worlds.value, isManagedServerWorld))

	const serverWorlds = computed(() =>
		visibleWorlds.value.filter((world): world is ServerWorld => world.type === 'server'),
	)

	const singleplayerWorlds = computed(() =>
		visibleWorlds.value.filter(
			(world): world is SingleplayerWorld => world.type === 'singleplayer',
		),
	)

	function updateWorlds(id: string, update: (worlds: World[]) => World[]) {
		queryClient.setQueryData<World[]>(worldKeys.instance(id), (current) => {
			if (!current) return current
			const next = update(current)
			sortWorlds(next)
			return next
		})
	}

	function invalidateWorlds() {
		return queryClient.invalidateQueries({ queryKey: worldsKey.value })
	}

	watch(
		() => [instanceId.value, managedServer.value?.name, managedServer.value?.address] as const,
		async ([id, name, address]) => {
			if (!id || !name || !address) return
			await ensureManagedServerWorldExists(id, name, address)
			await queryClient.invalidateQueries({ queryKey: worldKeys.instance(id) })
		},
		{ immediate: true },
	)

	async function refreshSingleplayerWorld(id: string, path: string) {
		try {
			const world = await getSingleplayerWorld(id, path)
			updateWorlds(id, (current) => [
				...current.filter((existing) => existing.type !== 'singleplayer' || existing.path !== path),
				world,
			])
		} catch {
			await queryClient.invalidateQueries({ queryKey: worldKeys.instance(id) })
		}
	}

	useAppEvent('instance', async (event) => {
		if (event.instance_id !== instanceId.value) return

		if (event.event === 'servers_updated') {
			await invalidateWorlds()
		} else if (event.event === 'world_updated') {
			await refreshSingleplayerWorld(event.instance_id, event.world)
		} else if (event.event === 'server_joined') {
			updateWorlds(event.instance_id, (current) =>
				current.map((world) =>
					world.type === 'server' && matchesJoinedServer(world, event.host, event.port)
						? { ...world, last_played: event.timestamp }
						: world,
				),
			)
		}
	})

	let lockedWorldRefreshTimer: ReturnType<typeof setTimeout> | undefined

	watch(instancePage.playing, (playing) => {
		if (playing) return
		clearTimeout(lockedWorldRefreshTimer)
		lockedWorldRefreshTimer = setTimeout(() => {
			if (singleplayerWorlds.value.some((world) => world.locked)) void invalidateWorlds()
		}, LOCKED_WORLD_REFRESH_DELAY_MS)
	})

	onScopeDispose(() => clearTimeout(lockedWorldRefreshTimer))

	return {
		worldsQuery,
		worldsKey,
		serverWorlds,
		singleplayerWorlds,
		isManagedServerWorld,
		invalidateWorlds,
	}
}
