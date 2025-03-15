<template>
  <div v-for="world in worlds" :key="world.name">
    <div class="flex items-center gap-2 p-3 bg-bg-raised rounded-xl mb-2">
      <Avatar :src="world.icon" size="48px" />
      <div class="flex flex-col justify-between">
        <div class="flex items-center gap-2">
          <div class="text-lg text-contrast font-bold">{{ world.name }}</div>
          <div v-if="world.type === 'singleplayer'" class="text-sm text-secondary flex items-center gap-1 font-semibold">
            <UserIcon class="h-4 w-4 text-secondary" stroke-width="3px" />
            Singleplayer
          </div>
          <div v-else-if="world.type === 'server'" class="text-sm text-secondary flex items-center gap-1 font-semibold">
            <template v-if="world.online">
              <SignalIcon  style="--_signal-4: var(--color-green)" stroke-width="3px" /> {{ world.online_players }} online
            </template>
            <template v-else>
              <NoSignalIcon stroke-width="3px" /> Offline
            </template>
          </div>
        </div>
        <div class="text-sm text-secondary">Played {{ dayjs(world.last_played).fromNow() }}</div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue'
import type { GameInstance } from '@/helpers/types'
import dayjs from 'dayjs'
import { Avatar } from '@modrinth/ui'
import { UserIcon, NoSignalIcon, SignalIcon } from '@modrinth/assets'
import { get_profile_worlds } from '@/helpers/worlds.ts'
import type { World } from '@/helpers/worlds.ts'
import { handleError } from '@/store/notifications'

const props = defineProps< {
  instance: GameInstance,
  offline: boolean,
}>()

const worlds = ref<World[]>([])


worlds.value = await get_profile_worlds(props.instance.path).catch(handleError) ?? [];
console.log(worlds.value)

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
