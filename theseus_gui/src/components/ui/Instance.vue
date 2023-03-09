<script setup>
import { Card } from 'omorphia'

const props = defineProps({
  display: String,
  instance: Object,
})

const handleInstanceClick = (id) => {
  console.log(id)
}
</script>

<template>
  <div>
    <div
      @click="handleInstanceClick(props.instance.id)"
      class="instance-list-item"
      v-if="display === 'list'"
    >
      <p>{{ props.instance.name }}</p>
    </div>
    <div class="instance-gallery-item" v-else-if="display === 'gallery'">
      <p>{{ props.instance.version }}</p>
      <p>{{ props.instance.name }}</p>
    </div>
    <Card class="instance-card-item" v-else-if="display === 'card'">
      <img src="https://cdn.modrinth.com/data/AANobbMI/icon.png" alt="Trending mod card" />
      <p>{{ props.instance.name }}</p>
    </Card>
  </div>
</template>

<style lang="scss" scoped>
.instance-list-item {
  display: inline-block;
  margin: 0.2rem auto;
  cursor: pointer;
  transition: all ease-out 0.1s;

  p {
    font-size: 0.8rem;
  }

  &:hover {
    font-weight: bold;
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

.instance-card-item {
  display: flex;
  justify-content: space-between;
  min-width: 250px;
  padding: 0;
  flex: 1 1 120px;
  cursor: pointer;
  transition: all ease-in-out 0.1s;

  &:hover {
    box-shadow: var(--shadow-raised-lg);
  }

  p {
    font-size: 0.9rem;
    display: inline-block;
    padding: 1rem;
  }

  img {
    width: 50px;
    height: 100%;
    border-radius: var(--radius-md);
  }
}
</style>
