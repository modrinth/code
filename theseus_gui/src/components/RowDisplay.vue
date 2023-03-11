<script setup>
import { ChevronLeftIcon, ChevronRightIcon } from 'omorphia'
import Instance from '@/components/ui/Instance.vue'
import News from '@/components/ui/News.vue'
import { onMounted, onUnmounted, ref } from 'vue'

const props = defineProps({
  instances: Array,
  news: Array,
  label: String,
  canPaginate: Boolean,
})

const allowPagination = ref(false)
const modsRow = ref(null)
const newsRow = ref(null)
const trendingRow = ref(null)

const shouldRenderNormalInstances =
  props.instances && props.instances?.length !== 0 && props.instances?.some((i) => !i.trending)
const shouldRenderNews = props.news && props.news?.length !== 0
const shouldRenderTrending =
  props.instances && props.instances?.length !== 0 && props.instances?.every((i) => i.trending)

const handlePaginationDisplay = () => {
  let parentsRow
  if (shouldRenderNormalInstances) parentsRow = modsRow.value
  if (shouldRenderNews) parentsRow = newsRow.value
  else if (shouldRenderTrending) parentsRow = trendingRow.value

  if (!parentsRow) return

  const children = parentsRow.children
  const lastChild = children[children.length - 1]
  const childBox = lastChild.getBoundingClientRect()

  if (childBox.x + childBox.width > window.innerWidth) allowPagination.value = true
  else allowPagination.value = false
}

onMounted(() => {
  if (props.canPaginate) window.addEventListener('resize', handlePaginationDisplay)
})

onUnmounted(() => {
  if (props.canPaginate) window.removeEventListener('resize', handlePaginationDisplay)
})

const handleLeftPage = () => {
  if (shouldRenderNormalInstances) modsRow.value.scrollLeft -= 100
  else if (shouldRenderNews) newsRow.value.scrollLeft -= 100
  else if (shouldRenderTrending) trendingRow.value.scrollLeft -= 100
}

const handleRightPage = () => {
  if (shouldRenderNormalInstances) modsRow.value.scrollLeft += 100
  else if (shouldRenderNews) newsRow.value.scrollLeft += 100
  else if (shouldRenderTrending) trendingRow.value.scrollLeft += 100
}
</script>

<template>
  <div class="row">
    <div class="header">
      <p>{{ props.label }}</p>
      <hr aria-hidden="true" />
      <div v-if="allowPagination" class="pagination">
        <ChevronLeftIcon @click="handleLeftPage" />
        <ChevronRightIcon @click="handleRightPage" />
      </div>
    </div>
    <section ref="modsRow" class="mods" v-if="shouldRenderNormalInstances">
      <Instance
        v-for="instance in props.instances"
        :key="instance.id"
        display="gallery"
        :instance="instance"
      />
    </section>
    <section ref="newsRow" class="news" v-else-if="shouldRenderNews">
      <News v-for="news in props.news" :key="news.id" :news="news" />
    </section>
    <section ref="trendingRow" class="trending" v-else-if="shouldRenderTrending">
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
    background: rgba(22, 24, 28, 0.7);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: inherit;
    width: 100%;
    margin-bottom: 1rem;
    gap: 1rem;

    p {
      color: var(--color-contrast);
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
    scroll-behavior: smooth;
    overflow-x: scroll;
    overflow-y: hidden;

    &::-webkit-scrollbar {
      width: 0px;
      background: transparent;
    }
  }

  .news {
    margin: auto;
    width: 100%;
    scroll-behavior: smooth;
    overflow-x: scroll;
    overflow-y: hidden;

    &::-webkit-scrollbar {
      width: 0px;
      background: transparent;
    }
  }

  .trending {
    display: flex;
    flex-direction: column;
    justify-content: flex-start;
    align-items: flex-start;
    flex-wrap: wrap;
    gap: 1rem;
    margin-right: auto;
    margin-top: 0.8rem;
    scroll-behavior: smooth;

    overflow-x: scroll;
    overflow-y: hidden;

    &::-webkit-scrollbar {
      width: 0px;
      background: transparent;
    }
  }
}

.dark-mode {
  .row {
    &:nth-child(even) {
      background-color: var(--color-raised-bg);
    }
  }
}
</style>
