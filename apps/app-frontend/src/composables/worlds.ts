import type { GameInstance } from '@/helpers/types'
import type { Ref } from 'vue'
import { onUnmounted, nextTick, computed, ref, watch } from 'vue'
import { profile_listener } from '@/helpers/events'
import {
  type SingleplayerWorld,
  type ServerStatus,
  type World,
  delete_world,
  get_server_status,
  remove_server_from_profile,
  type ServerWorld,
  start_join_server,
  start_join_singleplayer_world,
  getWorldIdentifier,
  get_singleplayer_world,
} from '@/helpers/worlds.ts'
import { get_profile_protocol_version, get_profile_worlds } from '@/helpers/worlds.ts'
import dayjs from 'dayjs'
import { defineMessage, defineMessages } from '@vintl/vintl'
import { BlocksIcon, CompassIcon, EyeIcon, PickaxeIcon, UnknownIcon } from '@modrinth/assets'
import type { FilterBarOption, GameVersion } from '@modrinth/ui'
import { autoToHTML } from '@geometrically/minecraft-motd-parser'
import { handleError } from '@/store/notifications'
import { get_game_versions } from '@/helpers/tags'

function sortWorlds(worlds: World[]) {
  worlds.sort((a, b) => {
    if (!a.last_played) {
      return 1
    }
    if (!b.last_played) {
      return -1
    }
    return dayjs(b.last_played).diff(dayjs(a.last_played))
  })
}

const messages = defineMessages({
  singleplayer: {
    id: 'instance.worlds.type.singleplayer',
    defaultMessage: 'Singleplayer',
  },
  server: {
    id: 'instance.worlds.type.server',
    defaultMessage: 'Server',
  },
  available: {
    id: 'instance.worlds.filter.available',
    defaultMessage: 'Available',
  },
})

export const GAME_MODES = {
  survival: {
    icon: PickaxeIcon,
    message: defineMessage({
      id: 'instance.worlds.game_mode.survival',
      defaultMessage: 'Survival mode',
    }),
  },
  creative: {
    icon: BlocksIcon,
    message: defineMessage({
      id: 'instance.worlds.game_mode.creative',
      defaultMessage: 'Creative mode',
    }),
  },
  adventure: {
    icon: CompassIcon,
    message: defineMessage({
      id: 'instance.worlds.game_mode.adventure',
      defaultMessage: 'Adventure mode',
    }),
  },
  spectator: {
    icon: EyeIcon,
    message: defineMessage({
      id: 'instance.worlds.game_mode.spectator',
      defaultMessage: 'Spectator mode',
    }),
  },
  unknown: {
    icon: UnknownIcon,
    message: defineMessage({
      id: 'instance.worlds.game_mode.unknown',
      defaultMessage: 'Unknown game mode',
    }),
  },
}

export async function useWorlds(
  instance: Ref<GameInstance>,
  playing: Ref<boolean>,
  play: (world: World) => void,
) {
  const filters = ref<string[]>([])
  const searchFilter = ref('')

  const refreshing = ref(false)
  const hadNoWorlds = ref(true)
  const startingInstance = ref(false)
  const worldPlaying = ref<World>()

  const worlds = ref<World[]>([])
  const serverStatus = ref<Record<string, ServerStatus>>({})
  const renderedMotds = ref<Record<string, string>>({})
  const refreshingServers = ref<string[]>([])

  const protocolVersion = ref<number | null>(null)
  protocolVersion.value = await get_profile_protocol_version(instance.value.path)

  type ProfileEvent = {
    event: 'servers_updated'
  } | {
    event: 'world_updated'
    world: string
  } | {
    event: 'server_joined'
    host: string
    port: number
    timestamp: string
  };
  const unlistenProfile = await profile_listener(async (e: ProfileEvent) => {
    if (e.event === 'servers_updated') {
      await refreshWorlds()
    }

    if (e.event === 'world_updated') {
      await updateWorld(e.world)
    }

    if (e.event === 'server_joined') {
      const world = worlds.value.find((w) => w.type === 'server' && (w.address === `${e.host}:${e.port}` || (e.port == 25565 && w.address == e.host)))
      if (world) {
        world.last_played = e.timestamp
        sortWorlds(worlds.value)
      }
    }
  })

  function unlistenWorldsListener() {
    unlistenProfile()
  }

  const filterOptions = computed(() => {
    const options: FilterBarOption[] = []

    if (worlds.value.some((x) => x.type === 'singleplayer')) {
      options.push({
        id: 'singleplayer',
        message: messages.singleplayer,
      })
    }

    if (worlds.value.some((x) => x.type === 'server')) {
      options.push({
        id: 'server',
        message: messages.server,
      })

      // add available filter if there's any offline ("unavailable") servers
      if (
        worlds.value.some(
          (x) =>
            x.type === 'server' &&
            !serverStatus.value[x.address] &&
            !refreshingServers.value.includes(x.address),
        )
      ) {
        options.push({
          id: 'available',
          message: messages.available,
        })
      }
    }

    return options
  })

  await refreshWorlds()

  function onRefreshError(err: unknown, addr: string | null = null) {
    console.error(`Refreshing addr: ${addr}`, err)
    refreshing.value = false
    if (addr) {
      refreshingServers.value = refreshingServers.value.filter((s) => s !== addr)
    }
  }

  async function refreshServer(address: string) {
    refreshingServers.value.push(address)
    await refreshServerPromise(address)
  }

  function refreshServerPromise(address: string): Promise<void> {
    return get_server_status(address, protocolVersion.value)
      .then((status) => {
        serverStatus.value[address] = status
        if (status.description) {
          renderedMotds.value[address] = autoToHTML(status.description)
        }
        refreshingServers.value = refreshingServers.value.filter((s) => s !== address)
      })
      .catch((error) => onRefreshError(error, address))
  }

  async function updateWorld(worldPath: string) {
    const newWorld = await get_singleplayer_world(instance.value.path, worldPath)

    worlds.value = worlds.value.map((w) =>
      w.type === 'singleplayer' && w.path === worldPath ? newWorld : w,
    )
    sortWorlds(worlds.value)
  }

  async function refreshWorlds() {
    if (refreshing.value) {
      console.log(`Already refreshing, cancelling refresh.`)
      return
    }

    refreshing.value = true

    worlds.value = (await get_profile_worlds(instance.value.path).catch(onRefreshError)) ?? []
    sortWorlds(worlds.value)

    const servers = worlds.value.filter((w) => w.type === 'server')
    refreshingServers.value = servers.map((server) => server.address)

    Promise.all(refreshingServers.value.map(refreshServerPromise))
    const hasNoWorlds = worlds.value.length === 0

    if (hadNoWorlds.value && hasNoWorlds) {
      setTimeout(() => {
        refreshing.value = false
      }, 1000)
    } else {
      refreshing.value = false
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
      worlds.value[index] = server
      sortWorlds(worlds.value)
      await refreshServer(server.address)
    } else {
      handleError(`Error refreshing server, refreshing all worlds`)
      await refreshWorlds()
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
      handleError(`Error finding world in list, refreshing all worlds`)
      await refreshWorlds()
    }
  }

  async function deleteWorld(world: SingleplayerWorld) {
    await delete_world(instance.value.path, world.path).catch(handleError)
    worlds.value = worlds.value.filter((w) => w.type !== 'singleplayer' || w.path !== world.path)
  }

  function handleJoinError(err: unknown) {
    handleError(err)
    startingInstance.value = false
    worldPlaying.value = undefined
  }

  async function joinWorld(world: World) {
    console.log(`Joining world ${getWorldIdentifier(world)}`)
    startingInstance.value = true
    worldPlaying.value = world
    if (world.type === 'server') {
      await start_join_server(instance.value.path, world.address).catch(handleJoinError)
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
              await updateWorld(world.path);
            }
          }
        }, 1000);
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

  const supportsQuickPlay = computed(() => {
    if (!gameVersions.value.length) {
      return false
    }

    const versionIndex = gameVersions.value.findIndex(
      (v) => v.version === instance.value.game_version,
    )
    const targetIndex = gameVersions.value.findIndex((v) => v.version === '23w14a')

    return versionIndex !== -1 && targetIndex !== -1 && versionIndex <= targetIndex
  })

  return {
    refreshing,
    startingInstance,
    filters,
    searchFilter,
    worlds,
    serverStatus,
    renderedMotds,
    refreshingServers,
    filterOptions,
    supportsQuickPlay,
    worldPlaying,
    protocolVersion,
    worldsMatch,
    addServer,
    editServer,
    removeServer,
    editWorld,
    deleteWorld,
    joinWorld,
    refreshWorlds,
    refreshServer,
    unlistenWorldsListener,
  }
}
