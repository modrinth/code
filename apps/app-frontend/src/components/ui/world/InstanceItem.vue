<script setup lang="ts">
import dayjs from 'dayjs'
import {
  EyeIcon,
  FolderOpenIcon,
  MoreVerticalIcon,
  PlayIcon,
  SpinnerIcon,
  StopCircleIcon,
} from '@modrinth/assets'
import {
  Avatar,
  ButtonStyled,
  commonMessages,
  OverflowMenu,
  SmartClickable,
  useRelativeTime,
} from '@modrinth/ui'
import { useVIntl } from '@vintl/vintl'
import { computed, nextTick, ref, onMounted, onUnmounted } from 'vue'
import { showProfileInFolder } from '@/helpers/utils'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useRouter } from 'vue-router'
import type { GameInstance } from '@/helpers/types'
import { get_project } from '@/helpers/cache'
import { capitalizeString } from '@modrinth/utils'
import { kill, run } from '@/helpers/profile'
import { handleSevereError } from '@/store/error'
import { trackEvent } from '@/helpers/analytics'
import { get_by_profile_path } from '@/helpers/process'
import { handleError } from '@/store/notifications'
import { process_listener } from '@/helpers/events'

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()

const router = useRouter()

const emit = defineEmits<{
  (e: 'play' | 'stop'): void
}>()

const props = defineProps<{
  instance: GameInstance
}>()

const loadingModpack = ref(!!props.instance.linked_data)

const modpack = ref()

if (props.instance.linked_data) {
  nextTick().then(async () => {
    modpack.value = await get_project(props.instance.linked_data?.project_id, 'must_revalidate')
    loadingModpack.value = false
  })
}

const instanceIcon = computed(() => props.instance.icon_path)

const loader = computed(() => {
  if (props.instance.loader === 'vanilla') {
    return 'Minecraft'
  } else if (props.instance.loader === 'neoforge') {
    return 'NeoForge'
  } else {
    return capitalizeString(props.instance.loader)
  }
})

const loading = ref(false)
const playing = ref(false)

const play = async (event: MouseEvent) => {
  event?.stopPropagation()
  loading.value = true
  await run(props.instance.path)
    .catch((err) => handleSevereError(err, { profilePath: props.instance.path }))
    .finally(() => {
      trackEvent('InstancePlay', {
        loader: props.instance.loader,
        game_version: props.instance.game_version,
        source: 'InstanceItem',
      })
    })
  emit('play')
  loading.value = false
}

const stop = async (event: MouseEvent) => {
  event?.stopPropagation()
  loading.value = true
  await kill(props.instance.path).catch(handleError)
  trackEvent('InstanceStop', {
    loader: props.instance.loader,
    game_version: props.instance.game_version,
    source: 'InstanceItem',
  })
  emit('stop')
  loading.value = false
}

const unlistenProcesses = await process_listener(async () => {
  await checkProcess()
})

const checkProcess = async () => {
  const runningProcesses = await get_by_profile_path(props.instance.path).catch(handleError)

  playing.value = runningProcesses.length > 0
}

onMounted(() => {
  checkProcess()
})

onUnmounted(() => {
  unlistenProcesses()
})
</script>
<template>
  <SmartClickable>
    <template #clickable>
      <router-link
        class="no-click-animation"
        :to="`/instance/${encodeURIComponent(instance.path)}`"
      />
    </template>
    <div
      class="grid grid-cols-[auto_minmax(0,3fr)_minmax(0,4fr)_auto] items-center gap-2 p-3 bg-bg-raised rounded-xl smart-clickable:highlight-on-hover"
    >
      <Avatar
        :src="instanceIcon ? convertFileSrc(instanceIcon) : undefined"
        :tint-by="instance.path"
        size="48px"
      />
      <div class="flex flex-col col-span-2 justify-between h-full">
        <div class="flex items-center gap-2">
          <div class="text-lg text-contrast font-bold truncate smart-clickable:underline-on-hover">
            {{ instance.name }}
          </div>
        </div>
        <div class="flex items-center gap-2 text-sm text-secondary">
          <div
            v-tooltip="
              instance.last_played
                ? dayjs(instance.last_played).format('MMMM D, YYYY [at] h:mm A')
                : null
            "
            class="w-fit shrink-0"
            :class="{ 'cursor-help smart-clickable:allow-pointer-events': instance.last_played }"
          >
            <template v-if="instance.last_played">
              {{
                formatMessage(commonMessages.playedLabel, {
                  time: formatRelativeTime(instance.last_played.toISOString()),
                })
              }}
            </template>
            <template v-else> Not played yet </template>
          </div>
          •
          <span v-if="modpack" class="flex items-center gap-1 truncate text-secondary">
            <router-link
              class="inline-flex items-center gap-1 truncate hover:underline text-secondary smart-clickable:allow-pointer-events"
              :to="`/project/${modpack.id}`"
            >
              <Avatar :src="modpack.icon_url" size="16px" class="shrink-0" />
              <span class="truncate">{{ modpack.title }}</span>
            </router-link>
            ({{ loader }} {{ instance.game_version }})
          </span>
          <span v-else-if="loadingModpack" class="flex items-center gap-1 truncate text-secondary">
            <SpinnerIcon class="animate-spin shrink-0" />
            <span class="truncate">Loading modpack...</span>
          </span>
          <span v-else class="flex items-center gap-1 truncate text-secondary">
            {{ loader }}
            {{ instance.game_version }}
          </span>
        </div>
      </div>
      <div class="flex gap-1 justify-end smart-clickable:allow-pointer-events">
        <ButtonStyled v-if="playing && !loading" color="red">
          <button @click="stop">
            <StopCircleIcon aria-hidden="true" />
            {{ formatMessage(commonMessages.stopButton) }}
          </button>
        </ButtonStyled>
        <ButtonStyled v-else>
          <button
            v-tooltip="playing ? 'Instance is already open' : null"
            :disabled="playing || loading"
            @click="play"
          >
            <SpinnerIcon v-if="loading" class="animate-spin" />
            <PlayIcon v-else aria-hidden="true" />
            {{ formatMessage(commonMessages.playButton) }}
          </button>
        </ButtonStyled>
        <ButtonStyled circular type="transparent">
          <OverflowMenu
            :options="[
              {
                id: 'open-instance',
                shown: !!instance.path,
                action: () => router.push(encodeURI(`/instance/${instance.path}`)),
              },
              {
                id: 'open-folder',
                action: () => showProfileInFolder(instance.path),
              },
            ]"
          >
            <MoreVerticalIcon aria-hidden="true" />
            <template #open-instance>
              <EyeIcon aria-hidden="true" />
              View instance
            </template>
            <template #open-folder>
              <FolderOpenIcon aria-hidden="true" />
              {{ formatMessage(commonMessages.openFolderButton) }}
            </template>
          </OverflowMenu>
        </ButtonStyled>
      </div>
    </div>
  </SmartClickable>
</template>
