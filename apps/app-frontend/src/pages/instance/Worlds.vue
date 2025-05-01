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
        <Button v-if="searchFilter" class="r-btn" @click="() => (searchFilter = '')">
          <XIcon />
        </Button>
      </div>
      <ButtonStyled>
        <button :disabled="refreshingAll" @click="refreshAllWorlds">
          <template v-if="refreshingAll">
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
        <button @click="addServerModal?.show()">
          <PlusIcon />
          Add a server
        </button>
      </ButtonStyled>
    </div>
    <FilterBar v-model="filters" :options="filterOptions" show-all-options />
    <div class="flex flex-col w-full gap-2">
      <WorldItem
        v-for="world in filteredWorlds"
        :key="`world-${world.type}-${world.type == 'singleplayer' ? world.path : `${world.address}-${world.index}`}`"
        :world="world"
        :highlighted="highlightedWorld === getWorldIdentifier(world)"
        :supports-quick-play="supportsQuickPlay"
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
            world.type === 'server' ? editServerModal?.show(world) : editWorldModal?.show(world)
        "
        @delete="() => promptToRemoveWorld(world)"
      />
    </div>
  </div>
  <div v-else class="w-full max-w-[48rem] mx-auto flex flex-col mt-6">
    <RadialHeader class="">
      <div class="flex items-center gap-6 w-[32rem] mx-auto">
        <img src="@/assets/sad-modrinth-bot.webp" alt="" aria-hidden="true" class="h-24" />
        <span class="text-contrast font-bold text-xl"> You don't have any worlds yet. </span>
      </div>
    </RadialHeader>
    <div class="flex gap-2 mt-4 mx-auto">
      <ButtonStyled>
        <button @click="addServerModal?.show()">
          <PlusIcon aria-hidden="true" />
          Add a server
        </button>
      </ButtonStyled>
      <ButtonStyled>
        <button :disabled="refreshingAll" @click="refreshAllWorlds">
          <template v-if="refreshingAll">
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
import { ref, computed, onUnmounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import type { GameInstance } from '@/helpers/types'
import {
  Button,
  ButtonStyled,
  RadialHeader,
  FilterBar,
  type FilterBarOption,
  type GameVersion,
  GAME_MODES,
} from '@modrinth/ui'
import { PlusIcon, SpinnerIcon, UpdatedIcon, SearchIcon, XIcon } from '@modrinth/assets'
import {
  type SingleplayerWorld,
  type World,
  type ServerWorld,
  type ServerData,
  type ProfileEvent,
  get_profile_protocol_version,
  remove_server_from_profile,
  delete_world,
  start_join_server,
  start_join_singleplayer_world,
  getWorldIdentifier,
  refreshServerData,
  refreshWorld,
  sortWorlds,
  refreshServers,
  hasQuickPlaySupport,
  refreshWorlds,
  handleDefaultProfileUpdateEvent,
} from '@/helpers/worlds.ts'
import AddServerModal from '@/components/ui/world/modal/AddServerModal.vue'
import EditServerModal from '@/components/ui/world/modal/EditServerModal.vue'
import EditWorldModal from '@/components/ui/world/modal/EditSingleplayerWorldModal.vue'
import WorldItem from '@/components/ui/world/WorldItem.vue'

import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import { handleError } from '@/store/notifications'
import type ContextMenu from '@/components/ui/ContextMenu.vue'
import type { Version } from '@modrinth/utils'
import { profile_listener } from '@/helpers/events'
import { get_game_versions } from '@/helpers/tags'
import { defineMessages } from '@vintl/vintl'

const route = useRoute()

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
  versions: Version[]
  installed: boolean
}>()

const instance = computed(() => props.instance)
const playing = computed(() => props.playing)

function play(world: World) {
  emit('play', world)
}

const filters = ref<string[]>([])
const searchFilter = ref('')

const refreshingAll = ref(false)
const hadNoWorlds = ref(true)
const startingInstance = ref(false)
const worldPlaying = ref<World>()

const worlds = ref<World[]>([])
const serverData = ref<Record<string, ServerData>>({})

const protocolVersion = ref<number | null>(await get_profile_protocol_version(instance.value.path))

const unlistenProfile = await profile_listener(async (e: ProfileEvent) => {
  if (e.profile_path_id !== instance.value.path) return

  console.info(`Handling profile event '${e.event}' for profile: ${e.profile_path_id}`)

  if (e.event === 'servers_updated') {
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

  refreshingAll.value = true

  worlds.value = await refreshWorlds(instance.value.path).finally(
    () => (refreshingAll.value = false),
  )
  await refreshServers(worlds.value, serverData.value, protocolVersion.value)

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
    handleError(`Error refreshing server, refreshing all worlds`)
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
    handleError(`Error finding world in list, refreshing all worlds`)
    await refreshAllWorlds()
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
const supportsQuickPlay = computed(() =>
  hasQuickPlaySupport(gameVersions.value, instance.value.game_version),
)

const filterOptions = computed(() => {
  const options: FilterBarOption[] = []

  const hasServer = worlds.value.some((x) => x.type === 'server')

  if (worlds.value.some((x) => x.type === 'singleplayer') && hasServer) {
    options.push({
      id: 'singleplayer',
      message: messages.singleplayer,
    })
    options.push({
      id: 'server',
      message: messages.server,
    })
  }

  if (hasServer) {
    // add available filter if there's any offline ("unavailable") servers AND there's any singleplayer worlds or available servers
    if (
      worlds.value.some(
        (x) =>
          x.type === 'server' &&
          !serverData.value[x.address]?.status &&
          !serverData.value[x.address]?.refreshing,
      ) &&
      worlds.value.some(
        (x) =>
          x.type === 'singleplayer' ||
          (x.type === 'server' &&
            serverData.value[x.address]?.status &&
            !serverData.value[x.address]?.refreshing),
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

const filteredWorlds = computed(() =>
  worlds.value.filter((x) => {
    const availableFilter = filters.value.includes('available')
    const typeFilter = filters.value.includes('server') || filters.value.includes('singleplayer')

    return (
      (!typeFilter || filters.value.includes(x.type)) &&
      (!availableFilter || x.type !== 'server' || serverData.value[x.address]?.status) &&
      (!searchFilter.value || x.name.toLowerCase().includes(searchFilter.value.toLowerCase()))
    )
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
    handleError(`Error removing server, no server marked for removal.`)
    return
  }
  await removeServer(serverToRemove.value)
  serverToRemove.value = undefined
}

async function proceedDeleteWorld() {
  if (!worldToDelete.value) {
    handleError(`Error deleting world, no world marked for removal.`)
    return
  }
  await deleteWorld(worldToDelete.value)
  worldToDelete.value = undefined
}

onUnmounted(() => {
  unlistenProfile()
})

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
</script>
