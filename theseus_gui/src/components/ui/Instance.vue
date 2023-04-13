<script setup>
import { shallowRef } from 'vue'
import { useRouter, RouterLink } from 'vue-router'
import { ofetch } from 'ofetch'
import { Card, SaveIcon } from 'omorphia'
import { PlayIcon } from '@/assets/icons'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { install as pack_install } from '@/helpers/pack'
import { run } from '@/helpers/profile'

const props = defineProps({
  instance: {
    type: Object,
    default() {
      return {}
    },
  },
})

const router = useRouter()

const install = async () => {
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
    const id = await pack_install(versions.value[0].id)
    await router.push({ path: `/instance/${encodeURIComponent(id)}` })
  }
  // TODO: Add condition for installing a mod
}

const play = () => {
  run(props.instance.metadata?.linked_project_id)
}
</script>

<template>
  <div>
    <RouterLink
      :to="
        props.instance.metadata
          ? `/instance/${encodeURIComponent(props.instance.path)}`
          : `/project/${encodeURIComponent(props.instance.project_id)}`
      "
    >
      <Card class="instance-card-item">
        <img
          :src="
            props.instance.metadata
              ? convertFileSrc(props.instance.metadata?.icon)
              : props.instance.icon_url
          "
          alt="Trending mod card"
        />
        <div class="project-info">
          <p class="title">{{ props.instance.metadata?.name || props.instance.title }}</p>
          <p class="description">
            {{ props.instance.metadata?.loader }}
            {{ props.instance.metadata?.game_version || props.instance.latest_version }}
          </p>
        </div>
        <div v-if="props.instance.metadata" class="cta" @click="play"><PlayIcon /></div>
        <div v-else class="cta" @click="install"><SaveIcon /></div>
      </Card>
    </RouterLink>
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

  .cta {
    position: absolute;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-brand);
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
