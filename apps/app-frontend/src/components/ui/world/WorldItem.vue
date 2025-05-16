<script setup lang="ts">
import dayjs from 'dayjs'
import type { ServerStatus, ServerWorld, World } from '@/helpers/worlds.ts'
import {
  set_world_display_status,
  getWorldIdentifier,
  showWorldInFolder,
} from '@/helpers/worlds.ts'
import { formatNumber } from '@modrinth/utils'
import {
  useRelativeTime,
  Avatar,
  ButtonStyled,
  commonMessages,
  OverflowMenu,
  SmartClickable,
} from '@modrinth/ui'
import {
  IssuesIcon,
  EyeIcon,
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
  XIcon,
} from '@modrinth/assets'
import type { MessageDescriptor } from '@vintl/vintl'
import { defineMessages, useVIntl } from '@vintl/vintl'
import type { Component } from 'vue'
import { computed } from 'vue'
import { copyToClipboard } from '@/helpers/utils'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useRouter } from 'vue-router'
import { Tooltip } from 'floating-vue'

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()

const router = useRouter()

const emit = defineEmits<{
  (e: 'play' | 'play-instance' | 'update' | 'stop' | 'refresh' | 'edit' | 'delete'): void
}>()

const props = withDefaults(
  defineProps<{
    world: World
    playingInstance?: boolean
    playingWorld?: boolean
    startingInstance?: boolean
    supportsQuickPlay?: boolean
    currentProtocol?: number | null
    highlighted?: boolean

    // Server only
    refreshing?: boolean
    serverStatus?: ServerStatus
    renderedMotd?: string

    // Singleplayer only
    gameMode?: {
      icon: Component
      message: MessageDescriptor
    }

    // Instance
    instancePath?: string
    instanceName?: string
    instanceIcon?: string
  }>(),
  {
    playingInstance: false,
    playingWorld: false,
    startingInstance: false,
    supportsQuickPlay: false,
    currentProtocol: null,

    refreshing: false,
    serverStatus: undefined,
    renderedMotd: undefined,

    gameMode: undefined,

    instancePath: undefined,
    instanceName: undefined,
    instanceIcon: undefined,
  },
)

const playingOtherWorld = computed(() => props.playingInstance && !props.playingWorld)
const hasPlayersTooltip = computed(
  () => !!props.serverStatus?.players?.sample && props.serverStatus.players?.sample?.length > 0,
)
const serverIncompatible = computed(
  () =>
    !!props.serverStatus &&
    !!props.serverStatus.version?.protocol &&
    !!props.currentProtocol &&
    props.serverStatus.version.protocol !== props.currentProtocol,
)

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

const locked = computed(() => props.world.type === 'singleplayer' && props.world.locked)

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
  viewInstance: {
    id: 'instance.worlds.view_instance',
    defaultMessage: 'View instance',
  },
  playAnyway: {
    id: 'instance.worlds.play_anyway',
    defaultMessage: 'Play anyway',
  },
  playInstance: {
    id: 'instance.worlds.play_instance',
    defaultMessage: 'Play instance',
  },
  worldInUse: {
    id: 'instance.worlds.world_in_use',
    defaultMessage: 'World is in use',
  },
  dontShowOnHome: {
    id: 'instance.worlds.dont_show_on_home',
    defaultMessage: `Don't show on Home`,
  },
})
</script>
<template>
  <SmartClickable>
    <template v-if="instancePath" #clickable>
      <router-link
        class="no-click-animation"
        :to="`/instance/${encodeURIComponent(instancePath)}/worlds?highlight=${encodeURIComponent(getWorldIdentifier(world))}`"
      />
    </template>
    <div
      class="grid grid-cols-[auto_minmax(0,3fr)_minmax(0,4fr)_auto] items-center gap-2 p-3 bg-bg-raised smart-clickable:highlight-on-hover rounded-xl"
      :class="{
        'world-item-highlighted': highlighted,
      }"
    >
      <Avatar
        :src="
          world.type === 'server' && serverStatus ? serverStatus.favicon ?? world.icon : world.icon
        "
        size="48px"
      />
      <div class="flex flex-col justify-between h-full">
        <div class="flex items-center gap-2">
          <div class="text-lg text-contrast font-bold truncate smart-clickable:underline-on-hover">
            {{ world.name }}
          </div>
          <div
            v-if="world.type === 'singleplayer'"
            class="text-sm text-secondary flex items-center gap-1 font-semibold"
          >
            <UserIcon
              aria-hidden="true"
              class="h-4 w-4 text-secondary shrink-0"
              stroke-width="3px"
            />
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
              <template v-if="serverIncompatible">
                <IssuesIcon class="shrink-0 text-orange" aria-hidden="true" />
                <span class="text-orange">
                  Incompatible version {{ serverStatus.version?.name }}
                </span>
              </template>
              <template v-else>
                <SignalIcon
                  v-tooltip="serverStatus ? `${serverStatus.ping}ms` : null"
                  aria-hidden="true"
                  :style="`--_signal-${getPingLevel(serverStatus.ping || 0)}: var(--color-green)`"
                  stroke-width="3px"
                  class="shrink-0"
                  :class="{
                    'smart-clickable:allow-pointer-events': serverStatus,
                  }"
                />
                <Tooltip :disabled="!hasPlayersTooltip">
                  <span :class="{ 'cursor-help': hasPlayersTooltip }">
                    {{ formatNumber(serverStatus.players?.online, false) }} online
                  </span>
                  <template #popper>
                    <div class="flex flex-col gap-1">
                      <span v-for="player in serverStatus.players?.sample" :key="player.name">
                        {{ player.name }}
                      </span>
                    </div>
                  </template>
                </Tooltip>
              </template>
            </template>
            <template v-else>
              <NoSignalIcon aria-hidden="true" stroke-width="3px" class="shrink-0" /> Offline
            </template>
          </div>
        </div>
        <div class="flex items-center gap-2 text-sm text-secondary">
          <div
            v-tooltip="
              world.last_played ? dayjs(world.last_played).format('MMMM D, YYYY [at] h:mm A') : null
            "
            class="w-fit shrink-0"
            :class="{ 'cursor-help smart-clickable:allow-pointer-events': world.last_played }"
          >
            <template v-if="world.last_played">
              {{
                formatMessage(commonMessages.playedLabel, {
                  time: formatRelativeTime(dayjs(world.last_played).toISOString()),
                })
              }}
            </template>
            <template v-else> Not played yet </template>
          </div>
          <template v-if="instancePath">
            •
            <router-link
              class="flex items-center gap-1 truncate hover:underline text-secondary smart-clickable:allow-pointer-events"
              :to="`/instance/${instancePath}`"
            >
              <Avatar
                :src="instanceIcon ? convertFileSrc(instanceIcon) : undefined"
                size="16px"
                :tint-by="instancePath"
                class="shrink-0"
              />
              <span class="truncate">{{ instanceName }}</span>
            </router-link>
          </template>
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
            class="motd-renderer font-normal font-minecraft line-clamp-2 text-secondary leading-5"
            v-html="renderedMotd"
          />
          <div v-else-if="!serverStatus" class="font-normal font-minecraft text-red leading-5">
            {{ formatMessage(messages.cantConnect) }}
          </div>
          <div v-else class="font-normal font-minecraft text-secondary leading-5">
            {{ formatMessage(messages.aMinecraftServer) }}
          </div>
        </template>
        <template v-else-if="world.type === 'singleplayer' && gameMode">
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
      <div class="flex gap-1 justify-end smart-clickable:allow-pointer-events">
        <template v-if="world.type === 'singleplayer' || serverStatus">
          <ButtonStyled
            v-if="(playingWorld || (locked && playingInstance)) && !startingInstance"
            color="red"
          >
            <button @click="emit('stop')">
              <StopCircleIcon aria-hidden="true" />
              {{ formatMessage(commonMessages.stopButton) }}
            </button>
          </ButtonStyled>
          <ButtonStyled v-else>
            <button
              v-tooltip="
                serverIncompatible
                  ? 'Server is incompatible'
                  : !supportsQuickPlay
                    ? formatMessage(messages.noQuickPlay)
                    : playingOtherWorld || locked
                      ? formatMessage(messages.gameAlreadyOpen)
                      : null
              "
              :disabled="!supportsQuickPlay || playingOtherWorld || startingInstance"
              @click="emit('play')"
            >
              <SpinnerIcon v-if="startingInstance && playingWorld" class="animate-spin" />
              <PlayIcon v-else aria-hidden="true" />
              {{ formatMessage(commonMessages.playButton) }}
            </button>
          </ButtonStyled>
        </template>
        <ButtonStyled v-else>
          <button class="invisible">
            <PlayIcon aria-hidden="true" />
            {{ formatMessage(commonMessages.playButton) }}
          </button>
        </ButtonStyled>
        <ButtonStyled circular type="transparent">
          <OverflowMenu
            :options="[
              {
                id: 'play-instance',
                shown: !!instancePath,
                disabled: playingInstance,
                action: () => emit('play-instance'),
              },
              {
                id: 'play-anyway',
                shown: serverIncompatible && !playingInstance && supportsQuickPlay,
                action: () => emit('play'),
              },
              {
                id: 'open-instance',
                shown: !!instancePath,
                action: () => router.push(encodeURI(`/instance/${instancePath}`)),
              },
              {
                id: 'refresh',
                shown: world.type === 'server',
                action: () => emit('refresh'),
              },
              {
                id: 'copy-address',
                shown: world.type === 'server',
                action: () => copyToClipboard((world as ServerWorld).address),
              },
              {
                id: 'edit',
                action: () => emit('edit'),
                shown: !instancePath,
                disabled: locked,
                tooltip: locked ? formatMessage(messages.worldInUse) : undefined,
              },
              {
                id: 'open-folder',
                shown: world.type === 'singleplayer',
                action: () =>
                  world.type === 'singleplayer' ? showWorldInFolder(instancePath, world.path) : {},
              },
              {
                divider: true,
                shown: !!instancePath,
              },
              {
                id: 'dont-show-on-home',
                shown: !!instancePath,
                action: () => {
                  set_world_display_status(
                    instancePath,
                    world.type,
                    getWorldIdentifier(world),
                    'hidden',
                  ).then(() => {
                    emit('update')
                  })
                },
              },
              {
                divider: true,
                shown: !instancePath,
              },
              {
                id: 'delete',
                color: 'red',
                hoverFilled: true,
                action: () => emit('delete'),
                shown: !instancePath,
                disabled: locked,
                tooltip: locked ? formatMessage(messages.worldInUse) : undefined,
              },
            ]"
          >
            <MoreVerticalIcon aria-hidden="true" />
            <template #play-instance>
              <PlayIcon aria-hidden="true" />
              {{ formatMessage(messages.playInstance) }}
            </template>
            <template #play-anyway>
              <PlayIcon aria-hidden="true" />
              {{ formatMessage(messages.playAnyway) }}
            </template>
            <template #open-instance>
              <EyeIcon aria-hidden="true" />
              {{ formatMessage(messages.viewInstance) }}
            </template>
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
            <template #dont-show-on-home>
              <XIcon aria-hidden="true" />
              {{ formatMessage(messages.dontShowOnHome) }}
            </template>
            <template #delete>
              <TrashIcon aria-hidden="true" />
              {{
                formatMessage(
                  world.type === 'server'
                    ? commonMessages.removeButton
                    : commonMessages.deleteLabel,
                )
              }}
            </template>
          </OverflowMenu>
        </ButtonStyled>
      </div>
    </div>
  </SmartClickable>
</template>
<style scoped lang="scss">
.world-item-highlighted {
  position: relative;
  animation: fade-highlight 4s ease-out;
  filter: brightness(1);

  &::before {
    @apply rounded-xl inset-0 absolute;

    animation: fade-opacity 4s ease-out;

    content: '';
    box-shadow: 0 0 8px 2px var(--color-brand);
    border: 1.5px solid var(--color-brand);
    opacity: 0;
  }
}

@keyframes fade-highlight {
  0% {
    filter: brightness(1.25);
  }
  75% {
    filter: brightness(1.25);
  }
  100% {
    filter: brightness(1);
  }
}

@keyframes fade-opacity {
  0% {
    opacity: 0.5;
  }
  75% {
    opacity: 0.5;
  }
  100% {
    opacity: 0;
  }
}

.light-mode .motd-renderer {
  filter: brightness(0.75);
}
</style>
