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
  align-items: center;
  justify-content: center;
  padding: 0;
  background: var(--color-bg);
  width: 350px;
  height: 100px;
  box-shadow: var(--shadow-raised-lg);
  cursor: pointer;

  &:hover {
    box-shadow: var(--shadow-floating);
  }

  img {
    display: flex;
    height: 100%;
    border-radius: 0.9rem 0 0 0.9rem;
  }
  .body {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    height: 100%;
    margin: 0.9rem;

    .headline {
      display: inherit;
      flex-direction: inherit;
      margin: 0.7rem 0;
      width: 100%;

      h2 {
        font-size: 0.8rem;
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
      align-self: flex-end;
      width: 100%;
      margin: 0px 0px 0.2rem 0.2rem;

      p {
        font-size: 0.7rem;
      }

      a {
        display: inline-block;
        transition: all ease-in-out 0.2s;
        width: 20px;
        font-size: 1.1rem;

        &:hover {
          transform: translate(2px);
        }
      }
    }
  }
}
</style>
