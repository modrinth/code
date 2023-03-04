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
  background: rgba(229, 229, 229, 0.1);
  width: 384px;
  height: 100px;
  margin: 10px 15px;

  img {
    display: flex;
    width: 20%;
    height: 100%;
    background: red;
    border-radius: 15px 0 0 15px;
  }
  .body {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: center;
    width: 80%;
    height: 100%;
    margin: 15px;

    .headline {
      display: inherit;
      flex-direction: inherit;
      margin: 10px 0;

      h2 {
        font-size: 14px;
        text-transform: uppercase;
      }

      p {
        font-size: 12px;
      }
    }

    .underline {
      display: flex;
      justify-content: space-between;
      align-items: center;
      align-self: flex-end;
      width: 100%;
      margin: 0px 0px 5px 5px;

      p {
        font-size: 12px;
      }
    }
  }
}
</style>
