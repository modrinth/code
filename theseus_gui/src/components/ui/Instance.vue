<script setup>
import { shallowRef, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ofetch } from 'ofetch'
import { Card, SaveIcon, XIcon, Avatar, AnimatedLogo } from 'omorphia'
import { PlayIcon } from '@/assets/icons'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import InstallConfirmModal from '@/components/ui/InstallConfirmModal.vue'
import { install as pack_install } from '@/helpers/pack'
import { run, list } from '@/helpers/profile'
import {
  kill_by_uuid,
  get_all_running_profile_paths,
  get_uuids_by_profile_path,
} from '@/helpers/process'

const props = defineProps({
  instance: {
    type: Object,
    default() {
      return {}
    },
  },
})

const confirmModal = ref(null)
const playing = ref(false)

const uuid = ref(null)
const modLoading = ref(false)

const router = useRouter()

const seeInstance = async () => {
  const instancePath = props.instance.metadata
    ? `/instance/${encodeURIComponent(props.instance.path)}`
    : `/project/${encodeURIComponent(props.instance.project_id)}`

  await router.push(instancePath)
}

const checkProcess = async () => {
  const runningPaths = await get_all_running_profile_paths()

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
  const [data, versions] = await Promise.all([
    ofetch(
      `https://api.modrinth.com/v2/project/${
        props.instance.metadata
          ? props.instance.metadata?.linked_project_id
          : props.instance.project_id
      }`
    ).then(shallowRef),
    ofetch(
      `https://api.modrinth.com/v2/project/${
        props.instance.metadata
          ? props.instance.metadata?.linked_project_id
          : props.instance.project_id
      }/version`
    ).then(shallowRef),
  ])

  if (data.value.project_type === 'modpack') {
    const packs = Object.values(await list())

    if (
      packs.length === 0 ||
      !packs.map((value) => value.metadata).find((pack) => pack.linked_project_id === data.value.id)
    ) {
      await pack_install(versions.value[0].id)
    } else confirmModal.value.show(versions.value[0].id)
  }

  modLoading.value = false
  // TODO: Add condition for installing a mod
}

const play = async (e) => {
  e.stopPropagation()
  modLoading.value = true
  uuid.value = await run(props.instance.path)
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
      const uuids = await get_uuids_by_profile_path(props.instance.path)
      uuids.forEach(async (u) => await kill_by_uuid(u))
    } else await kill_by_uuid(uuid.value) // If we still have the uuid, just kill it
  } catch (err) {
    // Theseus currently throws:
    //  "Error launching Minecraft: Minecraft exited with non-zero code 1" error
    // For now, we will catch and just warn
    console.warn(err)
  }

  uuid.value = null
}
</script>

<template>
  <div class="instance">
    <Card class="instance-card-item button-base" @click="seeInstance" @mouseenter="checkProcess">
      <Avatar
        size="lg"
        :src="
          props.instance.metadata
            ? convertFileSrc(props.instance.metadata?.icon)
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
    <div
      v-if="props.instance.metadata && playing === false && modLoading === false"
      class="install cta button-base"
      @click="play"
    >
      <PlayIcon />
    </div>
    <div v-else-if="modLoading === true && playing === false" class="cta loading">
      <AnimatedLogo class="loading" />
    </div>
    <div
      v-else-if="playing === true"
      class="stop cta button-base"
      @click="stop"
      @mousehover="checkProcess"
    >
      <XIcon />
    </div>
    <div v-else class="install cta buttonbase" @click="install"><SaveIcon /></div>
    <InstallConfirmModal ref="confirmModal" />
  </div>
</template>

<style lang="scss">
.instance {
  position: relative;

  &:hover {
    .cta {
      opacity: 1;
      bottom: 4.5rem;
    }
  }
}

.install {
  background: var(--color-brand);
  display: flex;
}

.stop {
  background: var(--color-red);
  display: flex;
}

.cta.loading {
  background: hsl(220, 11%, 10%) !important;
  display: flex;
  justify-content: center;
  align-items: center;

  .loading {
    width: 2.5rem !important;
    height: 2.5rem !important;
  }

  svg {
    width: 2.5rem !important;
    height: 2.5rem !important;
  }
}

.cta {
  position: absolute;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-lg);
  z-index: 41;
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
}

.instance-card-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: 0.75rem;
  transition: 0.1s ease-in-out all;
  background: hsl(220, 11%, 17%) !important;

  &:hover {
    filter: brightness(1) !important;
    background: hsl(220, 11%, 11%) !important;
  }

  .mod-image {
    border-radius: 1.5rem !important;
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
