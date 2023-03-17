<script setup>
import { Card, ChevronRightIcon } from 'omorphia'

const props = defineProps({
  news: Object,
})
</script>

<template>
  <Card class="news-cta">
    <img :src="props.news.img" alt="News Image" />
    <div class="body">
      <div class="headline">
        <h2>{{ props.news.headline }}</h2>
        <p>{{ props.news.blurb }}</p>
      </div>
      <div class="underline">
        <p>{{ props.news.source }}</p>
        <a href="#"><ChevronRightIcon /></a>
      </div>
    </div>
  </Card>
</template>

<style lang="scss" scoped>
.news-cta {
  display: flex;
  justify-content: center;
  padding: 0;
  background: var(--color-raised-bg);
  min-width: 24.125rem; /* from wireframe */
  min-height: 8.5rem; /* from wireframe */
  box-shadow: var(--shadow-raised-lg);
  cursor: pointer;
  transition: all ease-in-out 0.1s;

  &:hover {
    box-shadow: var(--shadow-floating);
    filter: brightness(0.85);
  }

  img {
    display: flex;
    width: 8.4375rem; /* from wireframe */
    height: 8.5rem; /* from wireframe */
    border-radius: 0.9rem 0 0 0.9rem;
  }

  .body {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 8.5rem; /* from wireframe */
    padding: 0.45rem;

    .headline {
      display: inherit;
      flex-direction: inherit;
      margin: 0.4rem 0;
      width: 100%;

      h2 {
        font-size: 1rem;
        text-transform: uppercase;
      }

      p {
        font-size: 0.7rem;
      }
    }

    .underline {
      display: flex;
      justify-content: space-between;
      align-items: center;
      width: 100%;
      margin-top: auto;

      p {
        font-size: 0.7rem;
      }

      a {
        transition: all ease-in-out 0.2s;
        width: 1.5rem;
        color: var(--color-primary);
        font-size: 1.3rem;

        &:hover {
          transform: translate(1px);
          filter: brightness(150%);
        }
      }
    }
  }
}
</style>
