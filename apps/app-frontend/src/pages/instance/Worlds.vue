<template>
  <div v-if="worlds.length > 0" class="flex flex-col gap-2">
    <div>
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
    </div>
    <div v-for="world in worlds" :key="world.name">
      <div class="grid grid-cols-[auto_3fr_4fr_auto] items-center gap-2 p-3 bg-bg-raised rounded-xl">
        <Avatar :src="world.type === 'server' && serverStatus[world.address] ? serverStatus[world.address].favicon : world.icon" size="48px" />
        <div class="flex flex-col justify-between h-full">
          <div class="flex items-center gap-2">
            <div class="text-lg text-contrast font-bold">{{ world.name }}</div>
            <div
              v-if="world.type === 'singleplayer'"
              class="text-sm text-secondary flex items-center gap-1 font-semibold"
            >
              <UserIcon class="h-4 w-4 text-secondary shrink-0" stroke-width="3px" />
              Singleplayer
            </div>
            <div
              v-else-if="world.type === 'server'"
              class="text-sm text-secondary flex items-center gap-1 font-semibold"
            >
              <template v-if="serverStatus[world.address]">
                <SpinnerIcon v-if="refreshingServers.includes(world.address)" class="animate-spin shrink-0" />
                <SignalIcon
                  v-else
                  v-tooltip="
                    serverStatus[world.address] ? `${serverStatus[world.address].ping}ms` : null
                  "
                  :style="`--_signal-${getPingLevel(serverStatus[world.address].ping || 0)}: var(--color-green)`"
                  stroke-width="3px"
                  class="shrink-0"
                />
                {{ formatNumber(serverStatus[world.address].players?.online, false) }} online
              </template>
              <template v-else> <NoSignalIcon stroke-width="3px" class="shrink-0" /> Offline </template>
            </div>
          </div>
          <div class="text-sm text-secondary">
            <template v-if="world.last_played">
              Played {{ dayjs(world.last_played).fromNow() }}
            </template>
            <template v-else> Not played yet </template>
          </div>
        </div>
        <div>
          <template v-if="world.type==='server'">
            <div v-if="refreshingServers.includes(world.address)" class="flex items-center text-secondary font-semibold gap-1 px-12 py-3 w-fit rounded-xl">
              <SpinnerIcon class="animate-spin" /> Loading...
            </div>
            <div v-else-if="renderedMotds[world.address]" class="motd">
              <div v-html="renderedMotds[world.address]" />
            </div>
            <div v-else-if="!serverStatus[world.address]" class="motd text-red">
              Can't connect to server
            </div>
            <div v-else>
              No MOTD
            </div>
          </template>
          <div v-else-if="world.type === 'singleplayer'" class="font-medium">
            {{ capitalizeString(world.game_mode) }} mode
          </div>
        </div>
        <div>
          <ButtonStyled>
            <button @click="joinWorld(world)">
              <PlayIcon />
              Play
            </button>
          </ButtonStyled>
        </div>
      </div>
    </div>
  </div>
  <div v-else class="w-full max-w-[48rem] mx-auto flex flex-col mt-6">
    <RadialHeader class="">
      <div class="flex items-center gap-6 w-[32rem] mx-auto">
        <img src="@/assets/sad-modrinth-bot.webp" class="h-24" />
        <span class="text-contrast font-bold text-xl"
        >
          You don't have any worlds yet.
        </span
        >
      </div>
    </RadialHeader>
    <div class="flex gap-2 mt-4 mx-auto">
      <ButtonStyled>
        <button>
          <PlusIcon />
          Add a server
        </button>
      </ButtonStyled>
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
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue'
import type { GameInstance } from '@/helpers/types'
import dayjs from 'dayjs'
import { Avatar, ButtonStyled, RadialHeader } from '@modrinth/ui'
import { PlusIcon, PlayIcon, UserIcon, NoSignalIcon, SpinnerIcon, SignalIcon, UpdatedIcon } from '@modrinth/assets'
import { get_profile_worlds, get_server_status, start_join_server, start_join_singleplayer_world } from '@/helpers/worlds.ts'
import type { World } from '@/helpers/worlds.ts'
import { handleError } from '@/store/notifications'
import type { ServerStatus } from '@/helpers/types.d.ts'
import { formatNumber, capitalizeString } from '@modrinth/utils'
import { autoToHTML } from '@sfirew/minecraft-motd-parser'

const props = defineProps<{
  instance: GameInstance
  offline: boolean
}>()

const refreshing = ref(false)

const worlds = ref<World[]>([])
const serverStatus = ref<Record<string, ServerStatus>>({})
const renderedMotds = ref<Record<string, string>>({})
const refreshingServers = ref<string[]>([])
const hadNoWorlds = ref(true);

refreshWorlds()

function onError(err: any, addr: string | null = null) {
  handleError(err)
  refreshing.value = false
  if (addr) {
    refreshingServers.value = refreshingServers.value.filter((s) => s !== addr)
  }
}

async function refreshWorlds() {
  refreshing.value = true

  worlds.value = (await get_profile_worlds(props.instance.path).catch(onError)) ?? []
  worlds.value.sort((a, b) => dayjs(b.last_played).diff(dayjs(a.last_played)))

  const servers = worlds.value.filter((w) => w.type === 'server')
  refreshingServers.value = servers.map((server) => server.address)

  await Promise.all(
    servers.map((server) =>
        get_server_status(server.address)
          .then((status) => {
            serverStatus.value[server.address] = status
            if (status.description) {
              renderedMotds.value[server.address] = autoToHTML(status.description)
            }
            refreshingServers.value = refreshingServers.value.filter((s) => s !== server.address)
          })
          .catch((error) => onError(error, server.address)),
      ),
  )
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

async function joinWorld(world: World) {
  if (world.type === 'server') {
    await start_join_server(props.instance.path, world.address)
  } else if (world.type === 'singleplayer') {
    await start_join_singleplayer_world(props.instance.path, world.path)
  }
}

// const worlds = ref<World[]>([
//   {
//     name: 'Hello World',
//     last_played: '2025-03-12T12:00:00-08:00',
//     icon: 'https://cdn.modrinth.com/data/mOgUt4GM/1bfe2006b38340e9d064700e41adf84a8abb1bd4_96.webp',
//     pinned: false,
//
//     type: 'singleplayer',
//     path: props.instance.path,
//     game_mode: 'survival',
//     hardcore: false,
//   },
//   {
//     name: 'Hello Server',
//     last_played: '2025-03-12T12:00:00-08:00',
//     icon: 'https://cdn.modrinth.com/data/mOgUt4GM/1bfe2006b38340e9d064700e41adf84a8abb1bd4_96.webp',
//     pinned: false,
//
//     type: 'server',
//     address: '127.0.0.1',
//   }
// ])
//

// const worlds = ref<World[]>([
//   {
//     name: 'Hello World',
//     last_played: '2025-03-12T12:00:00-08:00',
//     icon: 'https://cdn.modrinth.com/data/mOgUt4GM/1bfe2006b38340e9d064700e41adf84a8abb1bd4_96.webp',
//     pinned: false,
//
//     path: props.instance.path,
//     game_mode: 'survival',
//     hardcore: false,
//     type: 'singleplayer',
//   },
//   {
//     name: 'Hello Server',
//     last_played: '2025-03-12T12:00:00-08:00',
//     icon: 'https://cdn.modrinth.com/data/1bokaNcj/354080f65407e49f486fcf9c4580e82c45ae63b8_96.webp',
//     pinned: false,
//
//     online: false,
//     address: '127.0.0.1',
//     order: 0,
//     online_players: 3,
//     ping: 240,
//     motd: `This is a server's MOTD`,
//     type: 'server',
//   }
// ]);
</script>
<style lang="scss" scoped>
.motd {
  font-family: 'Minecraft';
}
</style>
