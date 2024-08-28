<script setup>
import { onUnmounted, ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { StopCircleIcon, PlayIcon } from '@modrinth/assets'
import { Card, Avatar, AnimatedLogo } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { kill, run } from '@/helpers/profile'
import { get_by_profile_path } from '@/helpers/process'
import { process_listener } from '@/helpers/events'
import { handleError } from '@/store/state.js'
import { showProfileInFolder } from '@/helpers/utils.js'
import { handleSevereError } from '@/store/error.js'
import { trackEvent } from '@/helpers/analytics'

const props = defineProps({
  instance: {
    type: Object,
    default() {
      return {}
    },
  },
})

const playing = ref(false)

const modLoading = computed(() => props.instance.install_stage !== 'installed')

const router = useRouter()

const seeInstance = async () => {
  await router.push(`/instance/${encodeURIComponent(props.instance.path)}/`)
}

const checkProcess = async () => {
  const runningProcesses = await get_by_profile_path(props.instance.path).catch(handleError)

  playing.value = runningProcesses.length > 0
}

const play = async (e, context) => {
  e?.stopPropagation()
  modLoading.value = true
  await run(props.instance.path).catch((err) =>
    handleSevereError(err, { profilePath: props.instance.path }),
  )
  modLoading.value = false

  trackEvent('InstancePlay', {
    loader: props.instance.loader,
    game_version: props.instance.game_version,
    source: context,
  })
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

const unlisten = await process_listener((e) => {
  if (e.event === 'finished' && e.profile_path_id === props.instance.path) playing.value = false
})

onUnmounted(() => unlisten())
</script>

<template>
  <div class="instance">
    <Card class="instance-card-item button-base" @click="seeInstance" @mouseenter="checkProcess">
      <Avatar
        size="lg"
        :src="props.instance.icon_path ? convertFileSrc(props.instance.icon_path) : null"
        alt="Mod card"
        class="mod-image"
      />
      <div class="project-info">
        <p class="title">{{ props.instance.name }}</p>
        <p
          v-if="
            props.instance.install_stage === 'installing' ||
            props.instance.install_stage === 'not_installed' ||
            props.instance.install_stage === 'pack_installing'
          "
          class="description"
        >
          Installing...
        </p>
        <p v-else class="description">
          {{ props.instance.loader }}
          {{ props.instance.game_version }}
        </p>
      </div>
    </Card>
    <div
      v-if="playing === true"
      class="stop cta button-base"
      @click="(e) => stop(e, 'InstanceCard')"
      @mousehover="checkProcess"
    >
      <StopCircleIcon />
    </div>
    <div v-else-if="modLoading === true && playing === false" class="cta loading-cta">
      <AnimatedLogo class="loading-indicator" />
    </div>
    <div v-else class="install cta button-base" @click="(e) => play(e, 'InstanceCard')">
      <PlayIcon />
    </div>
  </div>
</template>

<style lang="scss">
.loading-indicator {
  width: 2.5rem !important;
  height: 2.5rem !important;

  svg {
    width: 2.5rem !important;
    height: 2.5rem !important;
  }
}
</style>

<style lang="scss" scoped>
.instance {
  position: relative;

  &:hover {
    .cta {
      opacity: 1;
      bottom: calc(var(--gap-md) + 4.25rem);
    }
  }
}

.cta {
  position: absolute;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  z-index: 1;
  width: 3rem;
  height: 3rem;
  right: calc(var(--gap-md) * 2);
  bottom: 3.25rem;
  opacity: 0;
  transition:
    0.2s ease-in-out bottom,
    0.2s ease-in-out opacity,
    0.1s ease-in-out filter !important;
  cursor: pointer;
  box-shadow: var(--shadow-floating);

  svg {
    color: var(--color-accent-contrast);
    width: 1.5rem !important;
    height: 1.5rem !important;
  }

  &.install {
    background: var(--color-brand);
    display: flex;
  }

  &.stop {
    background: var(--color-red);
    display: flex;
  }

  &.loading-cta {
    background: hsl(220, 11%, 10%) !important;
    display: flex;
    justify-content: center;
    align-items: center;
  }
}

.instance-card-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: var(--gap-md);
  transition: 0.1s ease-in-out all !important; /* overrides Omorphia defaults */
  margin-bottom: 0;

  .mod-image {
    --size: 100%;

    width: 100% !important;
    height: auto !important;
    max-width: unset !important;
    max-height: unset !important;
    aspect-ratio: 1 / 1 !important;
  }

  .project-info {
    margin-top: 1rem;
    width: 100%;

    .title {
      color: var(--color-contrast);
      overflow: hidden;
      white-space: nowrap;
      text-overflow: ellipsis;
      width: 100%;
      margin: 0;
      font-weight: 600;
      font-size: 1rem;
      line-height: 110%;
      display: inline-block;
    }

    .description {
      color: var(--color-base);
      display: -webkit-box;
      -webkit-line-clamp: 2;
      -webkit-box-orient: vertical;
      overflow: hidden;
      font-weight: 500;
      font-size: 0.775rem;
      line-height: 125%;
      margin: 0.25rem 0 0;
      text-transform: capitalize;
      white-space: nowrap;
      text-overflow: ellipsis;
    }
  }
}
</style>
