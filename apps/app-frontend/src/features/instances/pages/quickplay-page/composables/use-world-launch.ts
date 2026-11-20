import { ref, shallowRef, watch } from 'vue'

import { handleSevereError } from '@/composables/use-error.js'
import { getWorldIdentifier } from '@/features/worlds/world-utils'
import { trackEvent } from '@/helpers/analytics'
import { injectInstancePage } from '@/pages/instance/instance-context'
import { startJoinServer, startJoinSingleplayerWorld } from '@/platform/app-lib/worlds/commands'
import type { ServerWorld, World } from '@/platform/app-lib/worlds/types'
import { injectServerInstall } from '@/providers/server-install'

const STOP_SOURCE = 'InstanceWorlds'
const ANALYTICS_SOURCE = 'WorldsPage'

function isSameWorld(world: World, other: World | undefined) {
	if (world.type === 'server' && other?.type === 'server') return world.address === other.address
	if (world.type === 'singleplayer' && other?.type === 'singleplayer') {
		return world.path === other.path
	}
	return false
}

export function useWorldLaunch(isManagedServerWorld: (world: World) => world is ServerWorld) {
	const instancePage = injectInstancePage()
	const { playServerProject } = injectServerInstall()

	const startingInstance = ref(false)
	const worldPlaying = shallowRef<World>()

	function trackServerStart() {
		const instance = instancePage.instance.value
		trackEvent('InstanceStart', {
			loader: instance.loader,
			game_version: instance.game_version,
			source: ANALYTICS_SOURCE,
		})
	}

	async function launch(world: World) {
		const instance = instancePage.instance.value
		if (world.type === 'singleplayer') {
			await startJoinSingleplayerWorld(instance.id, world.path)
			return
		}

		const managedProjectId = instance.link?.project_id
		if (managedProjectId && isManagedServerWorld(world)) {
			await playServerProject(managedProjectId)
		} else {
			await startJoinServer(instance.id, world.address)
		}
		trackServerStart()
	}

	async function joinWorld(world: World) {
		if (instancePage.instance.value.launch_blocked) return
		console.info(`Joining world ${getWorldIdentifier(world)}`)
		startingInstance.value = true
		worldPlaying.value = world
		try {
			await launch(world)
			await instancePage.refreshPlayState()
		} catch (error) {
			handleSevereError(error, { instanceId: instancePage.instance.value.id })
			worldPlaying.value = undefined
		} finally {
			startingInstance.value = false
		}
	}

	function stop() {
		return instancePage.stop(STOP_SOURCE)
	}

	function isPlayingWorld(world: World) {
		return isSameWorld(world, worldPlaying.value)
	}

	watch(instancePage.playing, (playing) => {
		if (!playing) worldPlaying.value = undefined
	})

	return {
		startingInstance,
		joinWorld,
		stop,
		isPlayingWorld,
	}
}
