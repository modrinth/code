<script setup>
import { ref, computed } from 'vue'
import { ChevronLeftIcon, ChevronRightIcon } from 'omorphia'
import Instance from './ui/Instance.vue'
import News from './ui/News.vue'

const props = defineProps({
  instances: Array,
  news: Array,
  label: String,
})

const shouldRenderInstances = props.instances && props.instances?.length !== 0
const shouldRenderNews = props.news && props.news?.length !== 0

const handleLeftPage = () => {
  console.log('page left')
}

const handleRightPage = () => {
  console.log('page right')
}
</script>

<template>
  <div class="row">
    <div class="header">
      <p>{{ props.label }}</p>
      <hr aria-hidden="true" />
      <div class="pagination">
        <ChevronLeftIcon @click="handleLeftPage" />
        <ChevronRightIcon @click="handleRightPage" />
      </div>
    </div>
    <section class="mods" v-if="shouldRenderInstances">
      <Instance
        v-for="instance in props.instances"
        :key="instance.id"
        display="gallery"
        :instance="instance"
      />
    </section>
    <section class="news" v-else-if="shouldRenderNews">
      <News v-for="news in props.news" :key="news.id" :news="news" />
    </section>
  </div>
</template>

<style lang="scss" scoped>
.row {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  margin-top: 2rem;
  padding: 1rem 0;

  &:nth-child(odd) {
    background: rgba(59, 59, 59, 0.5);
  }

  .header {
    display: flex;
    justify-content: space-evenly;
    align-items: inherit;
    width: 95%;

    p {
      font-size: 1rem;
    }

    hr {
      background: #fff;
      height: 1px;
      width: 60%;
    }

    .pagination {
      width: 20%;
      display: inherit;
      align-items: inherit;

      svg {
        cursor: pointer;
        margin-right: 0.5rem;
        transition: all ease-in-out 0.1s;

        &:hover {
          filter: brightness(150%);
        }
      }
    }
  }

  section {
    display: flex;
    align-items: inherit;
    transition: all ease-in-out 0.4s;
  }

  .mods {
    width: 100%;
    margin: auto;
    transition: all ease-in-out 0.4s;
  }

  .news {
    margin: auto;
    width: 100%;
  }
}
</style>
