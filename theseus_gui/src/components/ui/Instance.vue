<script setup>
import { RouterLink } from 'vue-router'
import { Card, PlusIcon } from 'omorphia'

const props = defineProps({
  display: String,
  instance: Object,
})
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

.instance-gallery-item {
  width: 110px;
  height: 110px;
  background-position: center;
  background-size: cover;
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  margin-left: 0.3rem;
  padding: 0.3rem;
  cursor: pointer;
  transition: all ease-in-out 0.2s;
  border-radius: var(--radius-md);

  &:hover {
    .cta {
      opacity: 1;
      top: 0.3rem;
    }
  }

  .cta {
    position: absolute;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-brand);
    border-radius: var(--radius-lg);
    width: 2rem;
    height: 2rem;
    right: 0.5rem;
    top: 0.8rem;
    opacity: 0;
    transition: 0.2s ease-in-out top, 0.1s ease-in-out opacity;
    cursor: pointer;
    z-index: 15;

    svg {
      color: #fff;
    }

    &:hover {
      filter: brightness(0.75);
      box-shadow: var(--shadow-floating);
    }
  }

  &:before {
    content: '';
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
    background-image: linear-gradient(to bottom right, rgba(50, 50, 50, 0.9), rgba(0, 0, 0, 0.9));
    opacity: 0.5;
    z-index: 10;
    border-radius: inherit;
  }

  &:hover {
    box-shadow: 0 0 4px 4px rgba(0, 0, 0, 0.5);
  }

  p {
    font-weight: bold;
    font-size: 0.75rem;
    color: #fff;
    z-index: 11;
    margin-bottom: 0.2rem;

    &:nth-child(1) {
      font-weight: normal;
      color: #ddd;
      font-size: 0.6rem;
    }
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
</style>
