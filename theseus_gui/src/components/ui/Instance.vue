<script setup>
import { shallowRef, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ofetch } from 'ofetch'
import { Card, SaveIcon, XIcon } from 'omorphia'
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
  load: {
    type: Function,
    default: () => {},
  },
})

const confirmModal = ref(null)
const stopBtn = ref(null)
const playBtn = ref(null)

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
  props.load()
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
  uuid.value = await run(props.instance.path)

  stopBtn.value.style.opacity = 1
  stopBtn.value.style.bottom = '4.5rem'
  stopBtn.value.style.display = 'flex'
  playBtn.value.style.display = 'none'
}

const stop = async (e) => {
  e.stopPropagation()
  await kill_by_uuid(uuid.value)

  stopBtn.value.style.opacity = 0
  stopBtn.value.style.bottom = '0'
  stopBtn.value.style.display = 'none'
  playBtn.value.style.display = 'flex'
}
</script>

<template>
  <div>
    <Card class="instance-card-item" @click="seeInstance">
      <img
        :src="
          props.instance.metadata
            ? convertFileSrc(props.instance.metadata?.icon)
            : props.instance.icon_url
        "
        alt="Mod card"
      />
      <div class="project-info">
        <p class="title">{{ props.instance.metadata?.name || props.instance.title }}</p>
        <p class="description">
          {{ props.instance.metadata?.loader }}
          {{ props.instance.metadata?.game_version || props.instance.latest_version }}
        </p>
      </div>
      <div v-if="props.instance.metadata" ref="playBtn" class="install cta" @click="play">
        <PlayIcon />
      </div>
      <div v-else class="install cta" @click="install"><SaveIcon /></div>
      <div ref="stopBtn" class="stop cta" @click="stop"><XIcon /></div>
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

  &:hover {
    filter: brightness(0.85);
    .install {
      opacity: 1;
      bottom: 4.5rem;
    }
  }

  .install {
    background: var(--color-brand);
    z-index: 55;
    display: flex;
  }

  .stop {
    background: var(--color-red);
    z-index: 77;
    display: none;
  }

  .cta {
    position: absolute;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-lg);
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
      filter: brightness(0.75);
      box-shadow: var(--shadow-floating);
    }
  }
  img {
    width: 100%;
    border-radius: var(--radius-sm);
    filter: none !important;
    aspect-ratio: 1;
  }
  .project-info {
    margin-top: 1rem;
    width: 100%;
    .title {
      color: var(--color-contrast);
      //max-width: 10rem;
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
