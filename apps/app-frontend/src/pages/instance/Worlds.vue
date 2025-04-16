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
        <button @click="addServerModal?.show()">
          <PlusIcon />
          Add a server
        </button>
      </ButtonStyled>
    </div>
    <FilterBar v-model="filters" :options="filterOptions" />
    <div class="flex flex-col w-full gap-2">
      <WorldItem
        v-for="world in filteredWorlds"
        :key="`world-${world.type}-${world.type == 'singleplayer' ? world.path : world.address}`"
        :world="world"
        :highlighted="highlightedWorld === getWorldIdentifier(world)"
        :supports-quick-play="supportsQuickPlay"
        :current-protocol="protocolVersion"
        :playing-instance="playing"
        :playing-world="worldsMatch(world, worldPlaying)"
        :starting-instance="startingInstance"
        :refreshing="
          world.type === 'server' ? refreshingServers.includes(world.address) : undefined
        "
        :server-status="world.type === 'server' ? serverStatus[world.address] : undefined"
        :rendered-motd="world.type === 'server' ? renderedMotds[world.address] : undefined"
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
import { ref, computed, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import type { GameInstance } from '@/helpers/types'
import { Button, ButtonStyled, RadialHeader, FilterBar } from '@modrinth/ui'
import { PlusIcon, SpinnerIcon, UpdatedIcon, SearchIcon, XIcon } from '@modrinth/assets'
import type { World, ServerWorld, SingleplayerWorld } from '@/helpers/worlds.ts'
import { getWorldIdentifier } from '@/helpers/worlds.ts'
import AddServerModal from '@/components/ui/world/modal/AddServerModal.vue'
import EditServerModal from '@/components/ui/world/modal/EditServerModal.vue'
import EditWorldModal from '@/components/ui/world/modal/EditSingleplayerWorldModal.vue'
import WorldItem from '@/components/ui/world/WorldItem.vue'

import { GAME_MODES, useWorlds } from '@/composables/worlds.ts'
import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import { handleError } from '@/store/notifications'

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
  offline: boolean
  playing: boolean
}>()

const instance = computed(() => props.instance)
const playing = computed(() => props.playing)

function play(world: World) {
  emit('play', world)
}

const {
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
} = await useWorlds(instance, playing, play)

const filteredWorlds = computed(() =>
  worlds.value.filter((x) => {
    const availableFilter = filters.value.includes('available')
    const typeFilter = filters.value.includes('server') || filters.value.includes('singleplayer')

    return (
      (!typeFilter || filters.value.includes(x.type)) &&
      (!availableFilter || x.type !== 'server' || serverStatus.value[x.address]) &&
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
  unlistenWorldsListener()
})
</script>
