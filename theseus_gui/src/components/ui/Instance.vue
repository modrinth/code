<script setup>
import { RouterLink } from 'vue-router'
import { Avatar, Card } from 'omorphia'
import { PlayIcon } from '@/assets/icons'
import { convertFileSrc } from '@tauri-apps/api/tauri'

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
</script>

<template>
  <div>
    <RouterLink :to="`/instance/${encodeURIComponent(props.instance.path)}`">
      <Card v-if="props.small" class="instance-small-card button-base">
        <Avatar
          :src="convertFileSrc(props.instance.metadata.icon)"
          :alt="props.instance.metadata.name"
          size="sm"
        />
        <div class="instance-small-card__info">
          <span class="title">{{ props.instance.metadata.name }}</span>
          {{ props.instance.metadata.game_version }} {{ props.instance.metadata.loader }}
        </div>
      </Card>
      <Card v-else class="instance-card-item">
        <img :src="convertFileSrc(props.instance.metadata.icon)" alt="Trending mod card" />
        <div class="project-info">
          <p class="title">{{ props.instance.metadata.name }}</p>
          <p class="description">
            {{ props.instance.metadata.loader }} {{ props.instance.metadata.game_version }}
          </p>
        </div>
        <div class="cta"><PlayIcon /></div>
      </Card>
    </RouterLink>
  </div>
</template>

<style lang="scss" scoped>
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
