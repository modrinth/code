<script setup>
import { RouterLink } from 'vue-router'
import { Card } from 'omorphia'

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
    <div class="instance-gallery-item" v-else-if="display === 'gallery'">
      <p>{{ props.instance.version }}</p>
      <p>{{ props.instance.name }}</p>
    </div>
    <Card class="instance-card-item" v-else-if="display === 'card'">
      <img src="https://cdn.modrinth.com/data/AANobbMI/icon.png" alt="Trending mod card" />
      <div class="project-info">
        <p class="title">{{ props.instance.name }}</p>
        <p class="description">{{ props.instance.description }}</p>
      </div>
    </Card>
  </div>
</template>

<style lang="scss" scoped>
.instance-list-item {
  display: inline-block;
  margin: 0.2rem auto;
  cursor: pointer;
  transition: all ease-out 0.1s;
  font-size: 0.8rem;
  color: var(--color-primary);

  &:hover {
    font-weight: bold;
    text-decoration: none;
    filter: brightness(150%);
  }
}

.instance-gallery-item {
  width: 110px;
  height: 110px;
  background: url('https://avatars1.githubusercontent.com/u/6166773?v=4');
  background-position: center;
  background-size: cover;
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  padding: 0.3rem;
  cursor: pointer;
  transition: all ease-in-out 0.2s;
  border-radius: var(--radius-md);

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

/* from Knossos */
.instance-card-item {
  position: relative;
  display: flex;

  cursor: pointer;
  padding: 1rem;
  gap: 1rem;
  border-radius: 1rem;
  border: 1px solid var(--color-raised-bg);
  transition: background 0.1s ease-in-out, transform 0.02s ease-in-out;

  &:hover {
    box-shadow: 0 0 4px 4px rgba(0, 0, 0, 0.5);
  }

  img {
    height: 3rem;
  }

  .project-info {
    box-sizing: border-box;
  }

  .title {
    color: var(--color-primary);
    max-width: 13.75rem;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    margin: 0;
    font-weight: 600;
    font-size: 1.25rem;
    line-height: 110%;
    display: block;
  }

  .description {
    width: 13.75rem;

    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;

    font-weight: 500;
    font-size: 0.875rem;
    line-height: 125%;
    margin: 0.25rem 0 0;
  }
}
</style>
