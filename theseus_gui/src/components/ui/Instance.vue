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

const playing = ref(false)
const uuid = ref(null)

const router = useRouter()

const seeInstance = async () =>
  await router.push(`/instance/${encodeURIComponent(props.instance.path)}`)

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
      <img :src="convertFileSrc(props.instance.metadata.icon)" alt="Trending mod card" />
      <div class="project-info">
        <p class="title">{{ props.instance.metadata.name }}</p>
        <p class="description">
          {{ props.instance.metadata.loader }} {{ props.instance.metadata.game_version }}
        </p>
      </div>
      <div v-if="playing === true" class="stop cta" @click="stop"><XIcon /></div>
      <div v-else class="install cta" @click="play"><PlayIcon /></div>
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
  z-index: 40;

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
