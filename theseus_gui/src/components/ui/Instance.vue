<script setup>
import { shallowRef, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ofetch } from 'ofetch'
import { Card, SaveIcon, XIcon, Avatar } from 'omorphia'
import { PlayIcon } from '@/assets/icons'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import InstallConfirmModal from '@/components/ui/InstallConfirmModal.vue'
import { install as pack_install } from '@/helpers/pack'
import { run, list } from '@/helpers/profile'
import { kill_by_uuid } from '@/helpers/process'

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

const router = useRouter()

const seeInstance = async () => {
  const instancePath = props.instance.metadata
    ? `/instance/${encodeURIComponent(props.instance.path)}`
    : `/project/${encodeURIComponent(props.instance.project_id)}`

  await router.push(instancePath)
}

const install = async (e) => {
  e.stopPropagation()
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
      router.go()
    } else confirmModal.value.show(versions.value[0].id)
  }
  // TODO: Add condition for installing a mod
}

const play = async (e) => {
  e.stopPropagation()
  playing.value = true
  uuid.value = await run(props.instance.path)
}

const stop = async (e) => {
  e.stopPropagation()
  playing.value = false
  await kill_by_uuid(uuid.value)
}
</script>

<template>
  <div>
    <Card class="instance-card-item" @click="seeInstance">
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
      <div v-if="props.instance.metadata && !playing" class="install cta button-base" @click="play">
        <PlayIcon />
      </div>
      <div v-else-if="playing === true" class="stop cta button-base" @click="stop"><XIcon /></div>
      <div v-else class="install cta buttonbase" @click="install"><SaveIcon /></div>
    </Card>
    <InstallConfirmModal ref="confirmModal" />
  </div>
</template>

<style lang="scss" scoped>
.instance-card-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: 0.75rem;
  transition: 0.1s ease-in-out all;

  &:hover {
    filter: brightness(0.85);

    .cta {
      opacity: 1;
      bottom: 4.5rem;
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

  .cta {
    position: absolute;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-lg);
    z-index: 41;
    width: 3rem;
    height: 3rem;
    right: 1rem;
    bottom: 3.5rem;
    opacity: 0;
    transition: 0.3s ease-in-out bottom, 0.1s ease-in-out opacity;
    cursor: pointer;

    svg {
      color: var(--color-accent-contrast);
      width: 1.5rem;
      height: 1.5rem;
    }

    &:hover {
      filter: none !important; /* overrides button-base class */
      box-shadow: var(--shadow-floating);
    }
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
