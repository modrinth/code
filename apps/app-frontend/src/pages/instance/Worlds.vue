<template>
  <AddServerModal ref="addServerModal" :instance="instance" @submit="addServer" />
  <EditServerModal ref="editServerModal" :instance="instance" @submit="editServer" />
  <EditWorldModal ref="editWorldModal" :instance="instance" @submit="editWorld" />
  <ConfirmModalWrapper
    ref="removeServerModal"
    :title="`Are you sure you want to remove this server?`"
    :description="`'${serverToRemove?.name}'${serverToRemove?.address === serverToRemove?.name ? ' ' : ` (${serverToRemove?.address})`} will be removed from your list, including in-game, and there will be no way to recover it.`"
    :markdown="false"
    @proceed="removeServer"
  />
  <ConfirmModalWrapper
    ref="deleteWorldModal"
    :title="`Are you sure you want to permanently delete this world?`"
    :description="`'${worldToRemove?.name}' will be **permanently deleted**, and there will be no way to recover it.`"
    @proceed="deleteWorld"
  />
  <div v-if="worlds.length > 0" class="flex flex-col gap-4">
    <div class="flex flex-wrap gap-2 items-center">
      <div class="iconified-input flex-grow">
        <SearchIcon />
        <input
          v-model="searchFilter"
          type="text"
          :placeholder="`Search worlds...`"
          class="text-input search-input"
          autocomplete="off"
        />
        <Button class="r-btn" @click="() => (searchFilter = '')">
          <XIcon />
        </Button>
      </div>
      <ButtonStyled>
        <button :disabled="refreshing" @click="refreshWorlds">
          <template v-if="refreshing">
            <SpinnerIcon class="animate-spin" />
            Refreshing...
          </template>
          <template v-else>
            <UpdatedIcon />
            Refresh
          </template>
        </button>
      </ButtonStyled>
      <ButtonStyled>
        <button @click="addServerModal.show()">
          <PlusIcon />
          Add a server
        </button>
      </ButtonStyled>
    </div>
    <FilterBar v-model="filters" :options="filterOptions" />
    <div
      class="flex flex-col w-full supports-[grid-template-columns:subgrid]:grid supports-[grid-template-columns:subgrid]:grid-cols-[auto_minmax(0,3fr)_minmax(0,4fr)_auto] gap-2"
    >
      <WorldItem
        v-for="world in worlds.filter((x) => {
          const availableFilter = filters.includes('available')
          const typeFilter = filters.includes('server') || filters.includes('singleplayer')

          return (
            (!typeFilter || filters.includes(x.type)) &&
            (!availableFilter || x.type !== 'server' || serverStatus[x.address]) &&
            (!searchFilter || x.name.toLowerCase().includes(searchFilter.toLowerCase()))
          )
        })"
        :key="world.name"
        :world="world"
        :supports-quick-play="supportsQuickPlay"
        :playing-instance="playing"
        :playing-world="worldsMatch(world, worldPlaying)"
        :starting-instance="startingInstance"
        :refreshing="
          world.type === 'server' ? refreshingServers.includes(world.address) : undefined
        "
        :server-status="world.type === 'server' ? serverStatus[world.address] : undefined"
        :rendered-motd="world.type === 'server' ? renderedMotds[world.address] : undefined"
        :game-mode="world.type === 'singleplayer' ? gameModes[world.game_mode] : undefined"
        @play="() => joinWorld(world)"
        @stop="() => emit('stop')"
        @refresh="() => refreshServer((world as ServerWorld).address)"
        @edit="
          () => (world.type === 'server' ? editServerModal.show(world) : editWorldModal.show(world))
        "
        @delete="() => promptToRemoveWorld(world)"
      />
    </div>
  </div>
  <div v-else class="w-full max-w-[48rem] mx-auto flex flex-col mt-6">
    <RadialHeader class="">
      <div class="flex items-center gap-6 w-[32rem] mx-auto">
        <img src="@/assets/sad-modrinth-bot.webp" alt aria-hidden="true" class="h-24" />
        <span class="text-contrast font-bold text-xl"> You don't have any worlds yet. </span>
      </div>
    </RadialHeader>
    <div class="flex gap-2 mt-4 mx-auto">
      <ButtonStyled>
        <button @click="addServerModal.show()">
          <PlusIcon aria-hidden="true" />
          Add a server
        </button>
      </ButtonStyled>
      <ButtonStyled>
        <button :disabled="refreshing" @click="refreshWorlds">
          <template v-if="refreshing">
            <SpinnerIcon aria-hidden="true" class="animate-spin" />
            Refreshing...
          </template>
          <template v-else>
            <UpdatedIcon aria-hidden="true" />
            Refresh
          </template>
        </button>
      </ButtonStyled>
    </div>
  </div>
</template>
<script setup lang="ts">
import { watch, ref, computed } from 'vue'
import type { GameInstance } from '@/helpers/types'
import dayjs from 'dayjs'
import {
  Avatar,
  Button,
  ButtonStyled,
  RadialHeader,
  FilterBar,
  type FilterBarOption,
  OverflowMenu,
} from '@modrinth/ui'
import {
  StopCircleIcon,
  ClipboardCopyIcon,
  MoreVerticalIcon,
  PlusIcon,
  PlayIcon,
  UserIcon,
  NoSignalIcon,
  SpinnerIcon,
  SignalIcon,
  UpdatedIcon,
  SearchIcon,
  XIcon,
  SkullIcon,
  PickaxeIcon,
  BlocksIcon,
  CompassIcon,
  EyeIcon,
  UnknownIcon,
  EditIcon,
  FolderOpenIcon,
  TrashIcon,
} from '@modrinth/assets'
import {
  delete_world,
  get_profile_protocol_version,
  get_profile_worlds,
  get_server_status,
  remove_server_from_profile,
  start_join_server,
  start_join_singleplayer_world,
} from '@/helpers/worlds.ts'
import type { ServerStatus, World, ServerWorld, SingleplayerWorld } from '@/helpers/worlds.ts'
import { formatNumber } from '@modrinth/utils'
import { autoToHTML } from '@geometrically/minecraft-motd-parser'
import { defineMessage, defineMessages, useVIntl } from '@vintl/vintl'
import AddServerModal from '@/components/ui/modal/AddServerModal.vue'
import EditServerModal from '@/components/ui/modal/EditServerModal.vue'
import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import { handleError } from '@/store/notifications'
import EditWorldModal from '@/components/ui/modal/EditSingleplayerWorldModal.vue'
import { get_game_versions } from '@/helpers/tags'
import WorldItem from '@/components/ui/world/WorldItem.vue'

const { formatMessage } = useVIntl()

const addServerModal = ref()
const editServerModal = ref()
const editWorldModal = ref()
const removeServerModal = ref()
const deleteWorldModal = ref()

const startingInstance = ref(false)
const serverToRemove = ref<ServerWorld>()
const worldToRemove = ref<SingleplayerWorld>()
const worldPlaying = ref<World>()

const emit = defineEmits<{
  (event: 'play', world: World): void
  (event: 'stop'): void
}>()

const props = defineProps<{
  instance: GameInstance
  offline: boolean
  playing: boolean
}>()

const refreshing = ref(false)
const filters = ref<string[]>([])
const searchFilter = ref('')

const protocolVersion = ref<number | null>(null)
protocolVersion.value = await get_profile_protocol_version(props.instance.path)

const worlds = ref<World[]>([])
const serverStatus = ref<Record<string, ServerStatus>>({})
const renderedMotds = ref<Record<string, string>>({})
const refreshingServers = ref<string[]>([])
const hadNoWorlds = ref(true)

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
  hardcore: {
    id: 'instance.worlds.hardcore',
    defaultMessage: 'Hardcore mode',
  },
  cantConnect: {
    id: 'instance.worlds.cant_connect',
    defaultMessage: "Can't connect to server",
  },
  aMinecraftServer: {
    id: 'instance.worlds.a_minecraft_server',
    defaultMessage: 'A Minecraft Server',
  },
})

const gameModes = ref({
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
})

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

refreshWorlds()

function onRefreshError(err: unknown, addr: string | null = null) {
  console.error(`Refreshing addr: ${addr}`, err)
  refreshing.value = false
  if (addr) {
    refreshingServers.value = refreshingServers.value.filter((s) => s !== addr)
  }
}

function sortWorlds() {
  worlds.value.sort((a, b) => dayjs(b.last_played).diff(dayjs(a.last_played)))
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

async function refreshWorlds() {
  refreshing.value = true

  worlds.value = (await get_profile_worlds(props.instance.path).catch(onRefreshError)) ?? []
  sortWorlds()

  const servers = worlds.value.filter((w) => w.type === 'server')
  refreshingServers.value = servers.map((server) => server.address)

  await Promise.all(refreshingServers.value.map(refreshServerPromise))
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
  sortWorlds()
  await refreshServer(server.address)
}

async function editServer(server: ServerWorld) {
  const index = worlds.value.findIndex((w) => w.type === 'server' && w.index === server.index)
  if (index !== -1) {
    worlds.value[index] = server
    sortWorlds()
    await refreshServer(server.address)
  } else {
    handleError(`Error refreshing server, refreshing all worlds`)
    await refreshWorlds()
  }
}
async function editWorld(path: string, name: string, removeIcon: boolean) {
  const world = worlds.value.find((world) => world.type === 'singleplayer' && world.path === path)
  if (world) {
    world.name = name
    if (removeIcon) {
      world.icon = undefined
    }
    sortWorlds()
  } else {
    handleError(`Error finding world in list, refreshing all worlds`)
    await refreshWorlds()
  }
}

async function promptToRemoveWorld(world: World) {
  if (world.type === 'server') {
    serverToRemove.value = world
    removeServerModal.value.show()
  } else {
    worldToRemove.value = world
    deleteWorldModal.value.show()
  }
}

async function removeServer() {
  if (!serverToRemove.value) {
    handleError(`Error removing server, no server marked for removal.`)
    return
  }
  await remove_server_from_profile(props.instance.path, serverToRemove.value.index).catch(
    handleError,
  )
  worlds.value = worlds.value.filter(
    (s) => s.type !== 'server' || s.index !== serverToRemove.value?.index,
  )
  serverToRemove.value = undefined
}

async function deleteWorld() {
  if (!worldToRemove.value) {
    handleError(`Error deleting world, no world marked for removal.`)
    return
  }
  await delete_world(props.instance.path, worldToRemove.value.path).catch(handleError)
  worlds.value = worlds.value.filter(
    (s) => s.type !== 'singleplayer' || s.path !== worldToRemove.value?.path,
  )
  worldToRemove.value = undefined
}

function getPingLevel(ping: number) {
  if (ping < 150) {
    return 5
  } else if (ping < 300) {
    return 4
  } else if (ping < 600) {
    return 3
  } else if (ping < 1000) {
    return 2
  } else {
    return 1
  }
}

function handleJoinError(err: unknown) {
  handleError(err)
  startingInstance.value = false
  worldPlaying.value = null
}

async function joinWorld(world: World) {
  startingInstance.value = true
  worldPlaying.value = world
  if (world.type === 'server') {
    await start_join_server(props.instance.path, world.address).catch(handleJoinError)
  } else if (world.type === 'singleplayer') {
    await start_join_singleplayer_world(props.instance.path, world.path).catch(handleJoinError)
  }
  emit('play', world)
  startingInstance.value = false
}

watch(
  () => props.playing,
  (playing) => {
    if (!playing) {
      worldPlaying.value = undefined
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

async function copyToClipboard(text: string) {
  await navigator.clipboard.writeText(text)
}

const gameVersions = ref(await get_game_versions().catch(() => []))

const supportsQuickPlay = computed(() => {
  if (!gameVersions.value.length) {
    return false
  }

  const versionIndex = gameVersions.value.findIndex(
    (v) => v.version === props.instance.game_version,
  )
  const targetIndex = gameVersions.value.findIndex((v) => v.version === '23w14a')

  console.log(versionIndex, targetIndex)

  return versionIndex !== -1 && targetIndex !== -1 && versionIndex <= targetIndex
})
</script>
