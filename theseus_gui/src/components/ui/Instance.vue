<script setup>
import { onUnmounted, ref, useSlots } from 'vue'
import { useRouter } from 'vue-router'
import { Card, DownloadIcon, XIcon, Avatar, AnimatedLogo, PlayIcon } from 'omorphia'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import InstallConfirmModal from '@/components/ui/InstallConfirmModal.vue'
import { install as pack_install } from '@/helpers/pack'
import { run, list } from '@/helpers/profile'
import {
  kill_by_uuid,
  get_all_running_profile_paths,
  get_uuids_by_profile_path,
} from '@/helpers/process'
import { process_listener } from '@/helpers/events'
import { useFetch } from '@/helpers/fetch.js'
import { handleError } from '@/store/state.js'

const props = defineProps({
  instance: {
    type: Object,
    default() {
      return {}
    },
  },
  small: {
    type: Boolean,
    default: false,
  },
})

const confirmModal = ref(null)
const playing = ref(false)

const uuid = ref(null)
const modLoading = ref(false)
const slots = useSlots()

const router = useRouter()

const seeInstance = async () => {
  const instancePath = props.instance.metadata
    ? `/instance/${encodeURIComponent(props.instance.path)}/`
    : `/project/${encodeURIComponent(props.instance.project_id)}/`

  await router.push(instancePath)
}

const checkProcess = async () => {
  const runningPaths = await get_all_running_profile_paths().catch(handleError)

  if (runningPaths.includes(props.instance.path)) {
    playing.value = true
    return
  }

  playing.value = false
  uuid.value = null
}

const install = async (e) => {
  e.stopPropagation()
  modLoading.value = true
  const versions = await useFetch(
    `https://api.modrinth.com/v2/project/${props.instance.project_id}/version`,
    'project versions'
  )

  if (props.instance.project_type === 'modpack') {
    const packs = Object.values(await list(true).catch(handleError))

    if (
      packs.length === 0 ||
      !packs
        .map((value) => value.metadata)
        .find((pack) => pack.linked_data?.project_id === props.instance.project_id)
    ) {
      try {
        modLoading.value = true
        await pack_install(versions[0].id, props.instance.title, props.instance.icon_url).catch(
          handleError
        )
        modLoading.value = false
      } catch (err) {
        console.error(err)
        modLoading.value = false
      }
    } else confirmModal.value.show(versions[0].id, props.instance.title, props.instance.icon_url)
  }

  modLoading.value = false
  // TODO: Add condition for installing a mod
}

const play = async (e) => {
  e.stopPropagation()
  modLoading.value = true
  uuid.value = await run(props.instance.path).catch(handleError)
  modLoading.value = false
  playing.value = true
}

const stop = async (e) => {
  e.stopPropagation()
  playing.value = false

  try {
    // If we lost the uuid for some reason, such as a user navigating
    // from-then-back to this page, we will get all uuids by the instance path.
    // For-each uuid, kill the process.
    if (!uuid.value) {
      const uuids = await get_uuids_by_profile_path(props.instance.path).catch(handleError)
      uuid.value = uuids[0]
      uuids.forEach(async (u) => await kill_by_uuid(u).catch(handleError))
    } else await kill_by_uuid(uuid.value).catch(handleError) // If we still have the uuid, just kill it
  } catch (err) {
    // Theseus currently throws:
    //  "Error launching Minecraft: Minecraft exited with non-zero code 1" error
    // For now, we will catch and just warn
    console.warn(err)
  }

  uuid.value = null
}

const unlisten = await process_listener((e) => {
  if (e.event === 'finished' && e.uuid === uuid.value) playing.value = false
})

onUnmounted(() => unlisten())
</script>

<template>
  <div class="instance">
    <Card v-if="props.small" class="instance-small-card" :class="{ 'button-base': !slots.content }">
      <div
        class="instance-small-card__description"
        :class="{ 'button-base': slots.content }"
        @click="seeInstance"
      >
        <Avatar
          :src="
            !props.instance.metadata.icon ||
            (props.instance.metadata.icon && props.instance.metadata.icon.startsWith('http'))
              ? props.instance.metadata.icon
              : convertFileSrc(instance.metadata?.icon)
          "
          :alt="props.instance.metadata.name"
          size="sm"
        />
        <div class="instance-small-card__info">
          <span class="title">{{ props.instance.metadata.name }}</span>
          {{
            props.instance.metadata.loader.charAt(0).toUpperCase() +
            props.instance.metadata.loader.slice(1)
          }}
          {{ props.instance.metadata.game_version }}
        </div>
      </div>
      <div v-if="slots.content" class="instance-small-card__content">
        <slot name="content" />
      </div>
    </Card>
    <Card
      v-else
      class="instance-card-item button-base"
      @click="seeInstance"
      @mouseenter="checkProcess"
    >
      <Avatar
        size="none"
        :src="
          props.instance.metadata
            ? !props.instance.metadata.icon ||
              (props.instance.metadata.icon && props.instance.metadata.icon.startsWith('http'))
              ? props.instance.metadata.icon
              : convertFileSrc(instance.metadata?.icon)
            : props.instance.icon_url
        "
        alt="Mod card"
        class="mod-image"
      />
      <div class="project-info">
        <p class="title">{{ props.instance.metadata?.name || props.instance.title }}</p>
        <p class="description">
          {{ props.instance.metadata?.loader }}
          {{ props.instance.metadata?.game_version || props.instance.latest_version }}
        </p>
      </div>
    </Card>
    <template v-if="!props.small">
      <div
        v-if="props.instance.metadata && playing === false && modLoading === false"
        class="install cta button-base"
        @click="play"
      >
        <PlayIcon />
      </div>
      <div v-else-if="modLoading === true && playing === false" class="cta loading-cta">
        <AnimatedLogo class="loading-indicator" />
      </div>
      <div
        v-else-if="playing === true"
        class="stop cta button-base"
        @click="stop"
        @mousehover="checkProcess"
      >
        <XIcon />
      </div>
      <div v-else class="install cta buttonbase" @click="install"><DownloadIcon /></div>
    </template>
    <InstallConfirmModal ref="confirmModal" />
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
.instance-small-card {
  background-color: var(--color-bg) !important;
  display: flex;
  flex-direction: column;
  min-height: min-content !important;
  gap: 0.5rem;
  align-items: flex-start;
  padding: 0;

  .instance-small-card__description {
    display: flex;
    flex-direction: row;
    justify-content: flex-start;
    gap: 1rem;
    flex-grow: 1;
    padding: var(--gap-xl);
    padding-bottom: 0;
    width: 100%;

    &:not(.button-base) {
      padding-bottom: var(--gap-xl);
    }
  }

  .instance-small-card__info {
    display: flex;
    flex-direction: column;
    justify-content: center;

    .title {
      color: var(--color-contrast);
      font-weight: bolder;
    }
  }

  .instance-small-card__content {
    padding: var(--gap-xl);
    padding-top: 0;
  }

  .cta {
    display: none;
  }
}

.instance {
  position: relative;

  &:hover {
    .cta {
      opacity: 1;
      bottom: 4.5rem;
    }

    .instance-card-item {
      background: hsl(220, 11%, 11%) !important;
    }
  }
}

.light-mode {
  .instance:hover {
    .instance-card-item {
      background: hsl(0, 0%, 91%) !important;
    }
  }
}

.light-mode {
  .instance-card-item {
    background: hsl(0, 0%, 100%) !important;

    &:hover {
      background: hsl(0, 0%, 91%) !important;
    }
  }
}

.cta {
  position: absolute;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-lg);
  z-index: 1;
  width: 3rem;
  height: 3rem;
  right: 1rem;
  bottom: 3.5rem;
  opacity: 0;
  transition: 0.3s ease-in-out bottom, 0.1s ease-in-out opacity !important;
  cursor: pointer;

  svg {
    color: var(--color-accent-contrast);
    width: 1.5rem !important;
    height: 1.5rem !important;
  }

  &:hover {
    filter: none !important; /* overrides button-base class */
    box-shadow: var(--shadow-floating);
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
  padding: 0.75rem !important; /* overrides card class */
  transition: 0.1s ease-in-out all !important; /* overrides Omorphia defaults */
  background: hsl(220, 11%, 17%) !important;

  &:hover {
    filter: brightness(1) !important;
    background: hsl(220, 11%, 11%) !important;
  }

  > .avatar {
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
    }
  }
}
</style>
