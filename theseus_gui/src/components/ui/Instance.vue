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
import { useNotifications } from '@/store/state'
import {
  kill_by_uuid,
  get_all_running_profile_paths,
  get_uuids_by_profile_path,
} from '@/helpers/process'
import { process_listener } from '@/helpers/events'

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

const notificationStore = useNotifications()

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
  try {
    const runningPaths = await get_all_running_profile_paths()

    if (runningPaths.includes(props.instance.path)) {
      playing.value = true
      return
    }

    playing.value = false
    uuid.value = null
  } catch (err) {
    notificationStore.addTauriErrorNotif(err)
  }
}

const install = async (e) => {
  e.stopPropagation()
  modLoading.value = true
  const [data, versions] = await Promise.all([
    ofetch(
      `https://api.modrinth.com/v2/project/${
        props.instance.metadata
          ? props.instance.metadata?.linked_data?.project_id
          : props.instance.project_id
      }`
    ).then(shallowRef),
    ofetch(
      `https://api.modrinth.com/v2/project/${
        props.instance.metadata
          ? props.instance.metadata?.linked_dadta?.project_id
          : props.instance.project_id
      }/version`
    ).then(shallowRef),
  ]).catch((err) => notificationStore.addApiErrorNotif(err))

  try {
    if (data.value.project_type === 'modpack') {
      const packs = Object.values(await list())

      if (
        packs.length === 0 ||
        !packs
          .map((value) => value.metadata)
          .find((pack) => pack.linked_data?.project_id === data.value.id)
      ) {
        await pack_install(versions.value[0].id)
      } else confirmModal.value.show(versions.value[0].id)
    }

    modLoading.value = false
    // TODO: Add condition for installing a mod
  } catch (err) {
    notificationStore.addTauriErrorNotif(err)
  }
}

const play = async (e) => {
  e.stopPropagation()
  modLoading.value = true

  try {
    uuid.value = await run(props.instance.path)
    playing.value = true
  } catch (err) {
    notificationStore.addTauriErrorNotif(err)
    playing.value = false
  } finally {
    modLoading.value = false
  }
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
      uuid.value = uuids[0]
      uuids.forEach(async (u) => await kill_by_uuid(u))
    } else await kill_by_uuid(uuid.value) // If we still have the uuid, just kill it
  } catch (err) {
    notificationStore.addTauriErrorNotif(err)
  }

  uuid.value = null
}

await process_listener((e) => {
  if (e.event === 'Finished' && e.uuid == uuid.value) playing.value = false
})
</script>

<template>
  <div class="instance">
    <Card v-if="props.small" class="instance-small-card button-base">
      <Avatar
        :src="convertFileSrc(props.instance.metadata.icon)"
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
    </Card>
    <Card
      v-else
      class="instance-card-item button-base"
      @click="seeInstance"
      @mouseenter="checkProcess"
    >
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
.instance-small-card {
  background-color: var(--color-bg) !important;
  padding: 1rem !important;
  display: flex;
  flex-direction: row;
  min-height: min-content !important;
  gap: 1rem;
  align-items: center;

  .instance-small-card__info {
    display: flex;
    flex-direction: column;
    justify-content: center;

    .title {
      color: var(--color-contrast);
      font-weight: bolder;
    }
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
  padding: 0.75rem !important; /* overrides card class */
  transition: 0.1s ease-in-out all !important; /* overrides Omorphia defaults */
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
