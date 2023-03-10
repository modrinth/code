<script setup>
import { ChevronLeftIcon, ChevronRightIcon } from 'omorphia'
import Instance from '@/components/ui/Instance.vue'
import News from '@/components/ui/News.vue'

const props = defineProps({
  instances: Array,
  news: Array,
  label: String,
})

const shouldRenderNormalInstances =
  props.instances && props.instances?.length !== 0 && props.instances?.some((i) => !i.trending)
const shouldRenderNews = props.news && props.news?.length !== 0
const shouldRenderTrending =
  props.instances && props.instances?.length !== 0 && props.instances?.every((i) => i.trending)

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
    <section class="mods" v-if="shouldRenderNormalInstances">
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
    <section class="trending" v-else-if="shouldRenderTrending">
      <Instance
        v-for="instance in props.instances"
        :key="instance.id"
        display="card"
        :instance="instance"
      />
    </section>
  </div>
</template>

<style lang="scss" scoped>
.row {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  padding: 1rem;

  &:nth-child(odd) {
    background: var(--color-raised-bg);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: inherit;
    width: 100%;
    margin-bottom: 1rem;
    gap: 1rem;

    p {
      font-size: 1rem;
      white-space: nowrap;
    }

    hr {
      background: var(--color-base);
      height: 1px;
      width: 100%;
      border: none;
    }

    .pagination {
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
    gap: 1rem;
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

  .trending {
    display: flex;
    flex-direction: column;
    justify-content: flex-start;
    align-items: flex-start;
    flex-wrap: wrap;
    gap: 1rem;
    height: 160px;
    margin-right: auto;
    margin-top: 0.8rem;
  }
}

.dark-mode {
  .row {
    &:nth-child(even) {
      background-color: rgba(22, 24, 28, 0.3);
    }
  }
}
</style>
