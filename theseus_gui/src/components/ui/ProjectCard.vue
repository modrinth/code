<script setup>
import { Card, Avatar } from "omorphia";

defineProps({
  project: {
    type: Object,
    default() {
      return {};
    },
  },
});


</script>

<template>
  <Card class="project-card btn">
    <Avatar class="icon" :src="project.icon_url"/>
    <div class="text">
      <p class="title">{{ project.title }}</p>
      <p class="subtitle">
        by {{ project.author }}
      </p>
      <p class="description">
        {{ project.description }}
      </p>
    </div>
  </Card>
</template>

<style scoped lang="scss">
.project-card {
  display: flex;
  flex-direction: row;
  padding: 0;
  aspect-ratio: 21/9;
  margin: 0;

  .icon {
    --size: 100% !important;
    aspect-ratio: 1/1;
    height: 100% !important;
    width: unset !important;
    border-radius: var(--radius-lg) 0 0 var(--radius-lg) !important;
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    padding: var(--gap-lg);

    .title {
      font-weight: bolder;
      white-space: nowrap;
      max-width: 10rem;
      overflow: hidden;
      text-overflow: ellipsis;
      margin: 0;
    }

    .subtitle {
      margin: 0;
    }

    .description {
      color: var(--color-base);
      margin: 0;
      max-height: 5rem;
      overflow: hidden;
      text-overflow: ellipsis;
      display: -webkit-box;
      -webkit-line-clamp: 3;
      -webkit-box-orient: vertical;
    }
  }
}
</style>
