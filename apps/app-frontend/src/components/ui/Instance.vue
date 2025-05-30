<script setup>
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import {
  DownloadIcon,
  GameIcon,
  PlayIcon,
  SpinnerIcon,
  StopCircleIcon,
  TimerIcon,
} from '@modrinth/assets'
import { Avatar, ButtonStyled, useRelativeTime } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { finish_install, kill, run } from '@/helpers/profile'
import { get_by_profile_path } from '@/helpers/process'
import { process_listener } from '@/helpers/events'
import { handleError } from '@/store/state.js'
import { showProfileInFolder } from '@/helpers/utils.js'
import { handleSevereError } from '@/store/error.js'
import { trackEvent } from '@/helpers/analytics'
import dayjs from 'dayjs'
import { formatCategory } from '@modrinth/utils'

const formatRelativeTime = useRelativeTime()

const props = defineProps({
  instance: {
    type: Object,
    default() {
      return {}
    },
  },
  compact: {
    type: Boolean,
    default: false,
  },
  first: {
    type: Boolean,
    default: false,
  },
})

const playing = ref(false)
const loading = ref(false)
const modLoading = computed(
  () =>
    loading.value ||
    currentEvent.value === 'installing' ||
    (currentEvent.value === 'launched' && !playing.value),
)
const installing = computed(() => props.instance.install_stage.includes('installing'))
const installed = computed(() => props.instance.install_stage === 'installed')

const router = useRouter()

const seeInstance = async () => {
  await router.push(`/instance/${encodeURIComponent(props.instance.path)}`)
}

const checkProcess = async () => {
  const runningProcesses = await get_by_profile_path(props.instance.path).catch(handleError)

  playing.value = runningProcesses.length > 0
}

const play = async (e, context) => {
  e?.stopPropagation()
  loading.value = true
  await run(props.instance.path)
    .catch((err) => handleSevereError(err, { profilePath: props.instance.path }))
    .finally(() => {
      trackEvent('InstancePlay', {
        loader: props.instance.loader,
        game_version: props.instance.game_version,
        source: context,
      })
    })
  loading.value = false
}

const stop = async (e, context) => {
  e?.stopPropagation()
  playing.value = false

  await kill(props.instance.path).catch(handleError)

  trackEvent('InstanceStop', {
    loader: props.instance.loader,
    game_version: props.instance.game_version,
    source: context,
  })
}

const repair = async (e) => {
  e?.stopPropagation()

  await finish_install(props.instance)
}

const openFolder = async () => {
  await showProfileInFolder(props.instance.path)
}

const addContent = async () => {
  await router.push({
    path: `/browse/${props.instance.loader === 'vanilla' ? 'datapack' : 'mod'}`,
    query: { i: props.instance.path },
  })
}

defineExpose({
  play,
  stop,
  seeInstance,
  openFolder,
  addContent,
  instance: props.instance,
})

const currentEvent = ref(null)

const unlisten = await process_listener((e) => {
  if (e.profile_path_id === props.instance.path) {
    currentEvent.value = e.event
    if (e.event === 'finished') {
      playing.value = false
    }
  }
})

onMounted(() => checkProcess())
onUnmounted(() => unlisten())
</script>

<template>
  <template v-if="compact">
    <div
      class="card-shadow grid grid-cols-[auto_1fr_auto] bg-bg-raised rounded-xl p-3 pl-4 gap-2 cursor-pointer hover:brightness-90 transition-all"
      @click="seeInstance"
      @mouseenter="checkProcess"
    >
      <Avatar
        size="48px"
        :src="instance.icon_path ? convertFileSrc(instance.icon_path) : null"
        :tint-by="instance.path"
        alt="Mod card"
      />
      <div class="h-full flex items-center font-bold text-contrast leading-normal">
        <span class="line-clamp-2">{{ instance.name }}</span>
      </div>
      <div class="flex items-center">
        <ButtonStyled v-if="playing" color="red" circular @mousehover="checkProcess">
          <button v-tooltip="'Stop'" @click="(e) => stop(e, 'InstanceCard')">
            <StopCircleIcon />
          </button>
        </ButtonStyled>
        <ButtonStyled v-else-if="modLoading" color="standard" circular>
          <button v-tooltip="'Instance is loading...'" disabled>
            <SpinnerIcon class="animate-spin" />
          </button>
        </ButtonStyled>
        <ButtonStyled v-else :color="first ? 'brand' : 'standard'" circular>
          <button
            v-tooltip="'Play'"
            @click="(e) => play(e, 'InstanceCard')"
            @mousehover="checkProcess"
          >
            <!-- Translate for optical centering -->
            <PlayIcon class="translate-x-[1px]" />
          </button>
        </ButtonStyled>
      </div>
      <div class="flex items-center col-span-3 gap-1 text-secondary font-semibold">
        <TimerIcon />
        <span class="text-sm">
          Played {{ formatRelativeTime(dayjs(instance.last_played).toISOString()) }}
        </span>
      </div>
    </div>
  </template>
  <div v-else>
    <div
      class="button-base bg-bg-raised p-4 rounded-xl flex gap-3 group"
      @click="seeInstance"
      @mouseenter="checkProcess"
    >
      <div class="relative flex items-center justify-center">
        <Avatar
          size="48px"
          :src="instance.icon_path ? convertFileSrc(instance.icon_path) : null"
          :tint-by="instance.path"
          alt="Mod card"
          :class="`transition-all ${modLoading || installing ? `brightness-[0.25] scale-[0.85]` : `group-hover:brightness-75`}`"
        />
        <div class="absolute inset-0 flex items-center justify-center">
          <ButtonStyled v-if="playing" size="large" color="red" circular>
            <button
              v-tooltip="'Stop'"
              :class="{ 'scale-100 opacity-100': playing }"
              class="transition-all scale-75 origin-bottom opacity-0 card-shadow"
              @click="(e) => stop(e, 'InstanceCard')"
              @mousehover="checkProcess"
            >
              <StopCircleIcon />
            </button>
          </ButtonStyled>
          <SpinnerIcon
            v-else-if="modLoading || installing"
            v-tooltip="modLoading ? 'Instance is loading...' : 'Installing...'"
            class="animate-spin w-8 h-8"
            tabindex="-1"
          />
          <ButtonStyled v-else-if="!installed" size="large" color="brand" circular>
            <button
              v-tooltip="'Repair'"
              class="transition-all scale-75 group-hover:scale-100 group-focus-within:scale-100 origin-bottom opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 card-shadow"
              @click="(e) => repair(e)"
            >
              <DownloadIcon />
            </button>
          </ButtonStyled>
          <ButtonStyled v-else size="large" color="brand" circular>
            <button
              v-tooltip="'Play'"
              class="transition-all scale-75 group-hover:scale-100 group-focus-within:scale-100 origin-bottom opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 card-shadow"
              @click="(e) => play(e, 'InstanceCard')"
              @mousehover="checkProcess"
            >
              <PlayIcon class="translate-x-[2px]" />
            </button>
          </ButtonStyled>
        </div>
      </div>
      <div class="flex flex-col gap-1">
        <p class="m-0 text-md font-bold text-contrast leading-tight line-clamp-1">
          {{ instance.name }}
        </p>
        <div class="flex items-center col-span-3 gap-1 text-secondary font-semibold mt-auto">
          <GameIcon class="shrink-0" />
          <span class="text-sm">
            {{ formatCategory(instance.loader) }} {{ instance.game_version }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
