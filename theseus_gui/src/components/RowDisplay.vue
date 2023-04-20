<script setup>
import { ChevronLeftIcon, ChevronRightIcon } from 'omorphia'
import Instance from '@/components/ui/Instance.vue'
import News from '@/components/ui/News.vue'
import { onMounted, onUnmounted, ref } from 'vue'

const props = defineProps({
  instances: {
    type: Array,
    default() {
      return []
    },
  },
  news: {
    type: Array,
    default() {
      return []
    },
  },
  label: {
    type: String,
    default: '',
  },
  canPaginate: Boolean,
  load: {
    type: Function,
    default: () => {},
  },
})
const allowPagination = ref(false)
const modsRow = ref(null)
const newsRow = ref(null)
// Remove after state is populated with real data
const shouldRenderNormalInstances = props.instances && props.instances?.length !== 0
const shouldRenderNews = props.news && props.news?.length !== 0
const handlePaginationDisplay = () => {
  let parentsRow
  if (shouldRenderNormalInstances) parentsRow = modsRow.value
  if (shouldRenderNews) parentsRow = newsRow.value
  if (!parentsRow) return
  const children = parentsRow.children
  const lastChild = children[children.length - 1]
  const childBox = lastChild?.getBoundingClientRect()
  if (childBox?.x + childBox?.width > window.innerWidth && props.canPaginate)
    allowPagination.value = true
  else allowPagination.value = false
}
onMounted(() => {
  if (props.canPaginate) window.addEventListener('resize', handlePaginationDisplay)
  // Check if pagination should be rendered on mount
  handlePaginationDisplay()
})
onUnmounted(() => {
  if (props.canPaginate) window.removeEventListener('resize', handlePaginationDisplay)
})
const handleLeftPage = () => {
  if (shouldRenderNormalInstances) modsRow.value.scrollLeft -= 170
  else if (shouldRenderNews) newsRow.value.scrollLeft -= 170
}
const handleRightPage = () => {
  if (shouldRenderNormalInstances) modsRow.value.scrollLeft += 170
  else if (shouldRenderNews) newsRow.value.scrollLeft += 170
}
</script>
<template>
  <div v-if="props.instances.length > 0" class="row">
    <div class="header">
      <p>{{ props.label }}</p>
      <hr aria-hidden="true" />
      <div v-if="allowPagination" class="pagination">
        <ChevronLeftIcon role="button" @click="handleLeftPage" />
        <ChevronRightIcon role="button" @click="handleRightPage" />
      </div>
    </div>
    <section v-if="shouldRenderNormalInstances" ref="modsRow" class="instances">
      <Instance
        v-for="instance in props.instances"
        :key="instance?.project_id || instance?.id"
        display="card"
        :instance="instance"
        :load="props.load"
        class="row-instance"
      />
    </section>
    <section v-else-if="shouldRenderNews" ref="newsRow" class="news">
      <News v-for="actualNews in props.news" :key="actualNews.id" :news="actualNews" />
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

  &:nth-child(even) {
    background: var(--color-bg);
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
      color: var(--color-contrast);
    }

    hr {
      background-color: var(--color-gray);
      height: 1px;
      width: 100%;
      border: none;
    }

    .pagination {
      display: inherit;
      align-items: inherit;

      svg {
        background: var(--color-raised-bg);
        border-radius: var(--radius-lg);
        width: 1.3rem;
        height: 1.2rem;
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

  .instances {
    display: flex;
    flex-direction: row;
    width: 100%;
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
    &:nth-child(odd) {
      background-color: rgb(30, 31, 34);
    }
  }
}

.row-instance {
  min-width: 12rem;
  max-width: 12rem;
}
</style>
