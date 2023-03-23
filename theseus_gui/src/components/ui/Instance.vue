<script setup>
import { RouterLink } from 'vue-router'
import { storeToRefs } from 'pinia'
import { Card, PlusIcon, ProjectCard } from 'omorphia'
import { useInstances } from '@/store/instancesStore'

const props = defineProps({
  display: String,
  instance: Object,
})

const instanceStore = useInstances()
const { getCategoriesByInstanceId } = storeToRefs(instanceStore)
</script>

<template>
  <div>
    <RouterLink v-if="display === 'list'" class="instance-list-item" :to="`${props.instance.id}`">{{
      props.instance.name
    }}</RouterLink>
    <Card class="instance-card-item" v-else-if="display === 'card'">
      <img :src="props.instance.img" alt="Trending mod card" />
      <div class="project-info">
        <p class="title">{{ props.instance.name }}</p>
        <p class="description">{{ props.instance.version }}</p>
      </div>
      <div class="cta"><PlusIcon /></div>
    </Card>
    <ProjectCard
      v-else-if="display === 'project'"
      class="instance-project-item"
      :id="props.instance?.slug"
      :type="props.instance?.project_type"
      :name="props.instance?.title"
      :description="props.instance?.description"
      :iconUrl="props.instance?.icon_url"
      :downloads="props.instance?.downloads?.toString()"
      :follows="props.instance?.follows"
      :createdAt="props.instance?.date_created"
      :updatedAt="props.instance?.date_modified"
      :categories="getCategoriesByInstanceId(props.instance?.project_id)"
      :projectTypeDisplay="props.instance?.project_type"
      projectTypeUrl="mod"
      :serverSide="props.instance?.server_side"
      :clientSide="props.instance?.client_side"
      :showUpdatedDate="false"
      :color="props.instance?.color"
    >
    </ProjectCard>
  </div>
</template>

<style lang="scss" scoped>
.instance-list-item {
  display: inline-block;
  margin: 0.25rem auto;
  cursor: pointer;
  transition: all ease-out 0.1s;
  font-size: 0.8rem;
  color: var(--color-primary);

  &:hover {
    text-decoration: none;
    filter: brightness(150%);
  }
}

.instance-card-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  width: 180px;
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
      color: #fff;
    }

    &:hover {
      filter: brightness(0.75);
      box-shadow: var(--shadow-floating);
    }
  }

  img {
    width: 160px;
    border-radius: var(--radius-sm);
    filter: none !important;
  }

  .project-info {
    margin-top: 1rem;
    width: 100%;

    .title {
      color: var(--color-contrast);
      max-width: 6rem;
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
      display: -webkit-box;
      -webkit-line-clamp: 2;
      -webkit-box-orient: vertical;
      overflow: hidden;
      font-weight: 500;
      font-size: 0.775rem;
      line-height: 125%;
      margin: 0.25rem 0 0;
    }
  }
}

.dark-mode {
  .cta > svg {
    color: #000;
  }
}

.instance-project-item {
  width: 100%;
  height: auto;
  margin: 2rem auto;
  cursor: pointer;
  transition: all ease-in-out 0.1s;

  &:hover {
    filter: brightness(0.85);
  }
}
</style>
