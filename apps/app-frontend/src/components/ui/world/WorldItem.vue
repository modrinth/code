<script setup lang="ts">
import dayjs from 'dayjs'
import type { ServerStatus, ServerWorld, World } from '@/helpers/worlds.ts'
import { formatNumber } from '@modrinth/utils'
import {
  ClipboardCopyIcon,
  EditIcon,
  FolderOpenIcon,
  MoreVerticalIcon,
  NoSignalIcon,
  PlayIcon,
  SignalIcon,
  SkullIcon,
  SpinnerIcon,
  StopCircleIcon,
  TrashIcon,
  UpdatedIcon,
  UserIcon,
} from '@modrinth/assets'
import { Avatar, ButtonStyled, commonMessages, OverflowMenu } from '@modrinth/ui'
import type { MessageDescriptor } from '@vintl/vintl'
import { defineMessages, useVIntl } from '@vintl/vintl'
import type { Component } from 'vue'
import { computed } from 'vue'
import { copyToClipboard } from '@/helpers/utils'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
  (e: 'play' | 'stop' | 'refresh' | 'edit' | 'delete'): void
}>()

const props = withDefaults(
  defineProps<{
    world: World
    playingInstance?: boolean
    playingWorld?: boolean
    startingInstance?: boolean
    supportsQuickPlay?: boolean

    // Server only
    refreshing?: boolean
    serverStatus?: ServerStatus
    renderedMotd?: string

    // Singleplayer only
    gameMode?: {
      icon: Component
      message: MessageDescriptor
    }
  }>(),
  {
    playingInstance: false,
    playingWorld: false,
    startingInstance: false,
    supportsQuickPlay: false,

    refreshing: false,
    serverStatus: undefined,
    renderedMotd: undefined,

    gameMode: undefined,
  },
)

const playingOtherWorld = computed(() => props.playingInstance && !props.playingWorld)

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

const messages = defineMessages({
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
  noQuickPlay: {
    id: 'instance.worlds.no_quick_play',
    defaultMessage: 'You can only jump straight into worlds on Minecraft 1.20+',
  },
  gameAlreadyOpen: {
    id: 'instance.worlds.game_already_open',
    defaultMessage: 'Instance is already open',
  },
  copyAddress: {
    id: 'instance.worlds.copy_address',
    defaultMessage: 'Copy address',
  },
})
</script>
<template>
  <div
    class="grid grid-cols-[auto_minmax(0,3fr)_minmax(0,4fr)_auto] items-center gap-2 p-3 bg-bg-raised rounded-xl supports-[grid-template-columns:subgrid]:col-span-full supports-[grid-template-columns:subgrid]:!grid-cols-subgrid"
  >
    <Avatar
      :src="
        world.type === 'server' && serverStatus ? serverStatus.favicon ?? world.icon : world.icon
      "
      size="48px"
    />
    <div class="flex flex-col justify-between h-full">
      <div class="flex items-center gap-2">
        <div class="text-lg text-contrast font-bold truncate">{{ world.name }}</div>
        <div
          v-if="world.type === 'singleplayer'"
          class="text-sm text-secondary flex items-center gap-1 font-semibold"
        >
          <UserIcon aria-hidden="true" class="h-4 w-4 text-secondary shrink-0" stroke-width="3px" />
          {{ formatMessage(commonMessages.singleplayerLabel) }}
        </div>
        <div
          v-else-if="world.type === 'server'"
          class="text-sm text-secondary flex items-center gap-1 font-semibold flex-nowrap whitespace-nowrap"
        >
          <template v-if="refreshing">
            <SpinnerIcon aria-hidden="true" class="animate-spin shrink-0" />
            Loading...
          </template>
          <template v-else-if="serverStatus">
            <SignalIcon
              v-tooltip="serverStatus ? `${serverStatus.ping}ms` : null"
              aria-hidden="true"
              :style="`--_signal-${getPingLevel(serverStatus.ping || 0)}: var(--color-green)`"
              stroke-width="3px"
              class="shrink-0"
            />
            {{ formatNumber(serverStatus.players?.online, false) }} online
          </template>
          <template v-else>
            <NoSignalIcon aria-hidden="true" stroke-width="3px" class="shrink-0" /> Offline
          </template>
        </div>
      </div>
      <div
        v-tooltip="
          world.last_played ? dayjs(world.last_played).format('MMMM D, YYYY [at] h:mm A') : null
        "
        class="text-sm text-secondary w-fit"
        :class="{ 'cursor-help': world.last_played }"
      >
        <template v-if="world.last_played">
          {{
            formatMessage(commonMessages.playedLabel, { time: dayjs(world.last_played).fromNow() })
          }}
        </template>
        <template v-else> Not played yet </template>
      </div>
    </div>
    <div
      class="font-semibold flex items-center gap-1 justify-center text-center"
      :class="world.type === 'singleplayer' && world.hardcore ? `text-red` : 'text-secondary'"
    >
      <template v-if="world.type === 'server'">
        <template v-if="refreshing">
          <SpinnerIcon aria-hidden="true" class="animate-spin" />
          {{ formatMessage(commonMessages.loadingLabel) }}
        </template>
        <div
          v-else-if="renderedMotd"
          class="font-normal font-minecraft line-clamp-2 text-secondary leading-5"
          v-html="renderedMotd"
        />
        <div v-else-if="!serverStatus" class="font-normal font-minecraft text-red leading-5">
          {{ formatMessage(messages.cantConnect) }}
        </div>
        <div v-else class="font-normal font-minecraft text-secondary leading-5">
          {{ formatMessage(messages.aMinecraftServer) }}
        </div>
      </template>
      <template v-else-if="world.type === 'singleplayer'">
        <template v-if="world.hardcore">
          <SkullIcon aria-hidden="true" class="h-4 w-4 shrink-0" />
          {{ formatMessage(messages.hardcore) }}
        </template>
        <template v-else>
          <component :is="gameMode.icon" aria-hidden="true" class="h-4 w-4 shrink-0" />
          {{ formatMessage(gameMode.message) }}
        </template>
      </template>
    </div>
    <div class="flex gap-1 justify-end">
      <template v-if="world.type === 'singleplayer' || serverStatus">
        <ButtonStyled v-if="playingWorld && !startingInstance" color="red">
          <button @click="emit('stop')">
            <StopCircleIcon aria-hidden="true" />
            {{ formatMessage(commonMessages.stopButton) }}
          </button>
        </ButtonStyled>
        <ButtonStyled v-else>
          <button
            v-tooltip="
              !supportsQuickPlay
                ? formatMessage(messages.noQuickPlay)
                : playingOtherWorld
                  ? formatMessage(messages.gameAlreadyOpen)
                  : null
            "
            :disabled="!supportsQuickPlay || playingOtherWorld || startingInstance"
            @click="emit('play', world)"
          >
            <SpinnerIcon v-if="startingInstance && playingWorld" class="animate-spin" />
            <PlayIcon v-else aria-hidden="true" />
            {{ formatMessage(commonMessages.playButton) }}
          </button>
        </ButtonStyled>
      </template>
      <ButtonStyled circular type="transparent">
        <OverflowMenu
          :options="[
            {
              id: 'refresh',
              shown: world.type === 'server',
              action: () => emit('refresh', world as ServerWorld),
            },
            {
              id: 'copy-address',
              shown: world.type === 'server',
              action: () => copyToClipboard((world as ServerWorld).address),
            },
            {
              id: 'edit',
              action: () => emit('edit', world),
            },
            {
              id: 'open-folder',
              shown: world.type === 'singleplayer',
              action: () => {},
            },
            {
              divider: true,
            },
            {
              id: 'delete',
              color: 'red',
              hoverFilled: true,
              action: () => emit('delete', world),
            },
          ]"
        >
          <MoreVerticalIcon aria-hidden="true" />
          <template #edit>
            <EditIcon aria-hidden="true" /> {{ formatMessage(commonMessages.editButton) }}
          </template>
          <template #open-folder>
            <FolderOpenIcon aria-hidden="true" />
            {{ formatMessage(commonMessages.openFolderButton) }}
          </template>
          <template #copy-address>
            <ClipboardCopyIcon aria-hidden="true" /> {{ formatMessage(messages.copyAddress) }}
          </template>
          <template #refresh>
            <UpdatedIcon aria-hidden="true" /> {{ formatMessage(commonMessages.refreshButton) }}
          </template>
          <template #delete>
            <TrashIcon aria-hidden="true" />
            {{
              formatMessage(
                world.type === 'server' ? commonMessages.removeButton : commonMessages.deleteLabel,
              )
            }}
          </template>
        </OverflowMenu>
      </ButtonStyled>
    </div>
  </div>
</template>
