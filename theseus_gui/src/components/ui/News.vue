<script setup>
import { Card, ChevronRightIcon } from 'omorphia'

const props = defineProps({
  news: Object,
})
</script>

<template>
  <Card class="news-cta">
    <img src="https://avatars1.githubusercontent.com/u/6166773?v=4" alt="News Image" />
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
  background: var(--color-bg);
  min-width: 386px; /* from wireframe */
  min-height: 136px; /* from wireframe */
  box-shadow: var(--shadow-raised-lg);
  cursor: pointer;

  &:hover {
    box-shadow: var(--shadow-floating);
  }

  img {
    display: flex;
    width: 135px; /* from wireframe */
    height: 136px; /* from wireframe */
    border-radius: 0.9rem 0 0 0.9rem;
  }
  .body {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 136px; /* from wireframe */
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
        width: 24px;
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
