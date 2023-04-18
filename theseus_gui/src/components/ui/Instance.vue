<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { Card, XIcon } from 'omorphia'
import { PlayIcon } from '@/assets/icons'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { run } from '@/helpers/profile'
import { kill_by_uuid } from '@/helpers/process'

const props = defineProps({
  instance: {
    type: Object,
    default() {
      return {}
    },
  },
})

const stopBtn = ref(null)
const playBtn = ref(null)

const uuid = ref(null)

const router = useRouter()

const seeInstance = async () => {
  const instancePath = `/instance/${encodeURIComponent(props.instance.path)}`
  await router.push(instancePath)
}

const play = async (e) => {
  console.log('playing')
  e.stopPropagation()
  uuid.value = await run(props.instance.path)
  console.log('uuid', uuid.value)

  stopBtn.value.style.opacity = 1
  stopBtn.value.style.bottom = '4.5rem'
  stopBtn.value.style.display = 'flex'
  playBtn.value.style.display = 'none'
}

const stop = async (e) => {
  console.log('stopping')
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
      <img :src="convertFileSrc(props.instance.metadata.icon)" alt="Trending mod card" />
      <div class="project-info">
        <p class="title">{{ props.instance.metadata.name }}</p>
        <p class="description">
          {{ props.instance.metadata.loader }} {{ props.instance.metadata.game_version }}
        </p>
      </div>
      <div ref="playBtn" class="install cta" @click="play"><PlayIcon /></div>
      <div ref="stopBtn" class="stop cta" @click="stop"><XIcon /></div>
    </Card>
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
