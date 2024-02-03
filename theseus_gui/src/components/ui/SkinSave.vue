<script setup>
//import { ref } from 'vue'
import { Card, Avatar } from 'omorphia'

const props = defineProps({
  data: {
    type: Object,
    default() {
      return {}
    },
  },
})

</script>

<template>
  <div class="instance">
    <Card class="instance-card-item button-base" @click="$emit('setSkin', props.data)" >
      <Avatar
        size="lg"
        :src="props.data.model"
        alt="Mod card"
        class="mod-image"
      />
      <div class="project-info">
        <p class="title">{{ props.data.name }}</p>
        <p class="description">
          {{ props.data.arms }}, {{ props.data.cape }}
        </p>
      </div>
    </Card>
  </div>
</template>

<style lang="scss">
.loading-indicator {
  width: 2.5rem !important;
  height: 2.5rem !important;

  svg {
    width: 2.5rem !important;
    height: 2.5rem !important;
  }
}
</style>

<style lang="scss" scoped>
.instance {
  position: relative;

  &:hover {
    .cta {
      opacity: 1;
      bottom: calc(var(--gap-md) + 4.25rem);
    }
  }
}

.cta {
  position: absolute;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  z-index: 1;
  width: 3rem;
  height: 3rem;
  right: calc(var(--gap-md) * 2);
  bottom: 3.25rem;
  opacity: 0;
  transition: 0.2s ease-in-out bottom, 0.2s ease-in-out opacity, 0.1s ease-in-out filter !important;
  cursor: pointer;
  box-shadow: var(--shadow-floating);

  svg {
    color: var(--color-accent-contrast);
    width: 1.5rem !important;
    height: 1.5rem !important;
  }

  &.install {
    background: var(--color-brand);
    display: flex;
  }

  &.stop {
    background: var(--color-red);
    display: flex;
  }

  &.loading-cta {
    background: hsl(220, 11%, 10%) !important;
    display: flex;
    justify-content: center;
    align-items: center;
  }
}

.instance-card-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: var(--gap-md);
  transition: 0.1s ease-in-out all !important; /* overrides Omorphia defaults */
  margin-bottom: 0;

  .mod-image {
    --size: 100%;

    width: 100% !important;
    height: auto !important;
    max-width: unset !important;
    max-height: unset !important;
    aspect-ratio: 1 / 1 !important;
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
      white-space: nowrap;
      text-overflow: ellipsis;
    }
  }
}
</style>
