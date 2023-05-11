<script setup>
import { ChevronLeftIcon, ChevronRightIcon } from 'omorphia'
import Instance from '@/components/ui/Instance.vue'
import { onMounted, onUnmounted, ref } from 'vue'

const props = defineProps({
  instances: {
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
})

const allowPagination = ref(false)
const modsRow = ref(null)

const handlePaginationDisplay = () => {
  let parentsRow = modsRow.value

  // This is wrapped in a setTimeout because the HtmlCollection seems to struggle
  // with getting populated sometimes. It's a flaky error, but providing a bit of
  // wait-time for the below expressions has not failed thus-far.
  setTimeout(() => {
    const children = parentsRow.children
    const lastChild = children[children.length - 1]
    const childBox = lastChild?.getBoundingClientRect()

    if (childBox?.x + childBox?.width > window.innerWidth && props.canPaginate)
      allowPagination.value = true
    else allowPagination.value = false
  }, 300)
}

onMounted(() => {
  if (props.canPaginate) window.addEventListener('resize', handlePaginationDisplay)

  handlePaginationDisplay()
})
onUnmounted(() => {
  if (props.canPaginate) window.removeEventListener('resize', handlePaginationDisplay)
})

const handleLeftPage = () => {
  modsRow.value.scrollLeft -= 170
}
const handleRightPage = () => {
  modsRow.value.scrollLeft += 170
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
    <section ref="modsRow" class="instances">
      <Instance
        v-for="instance in props.instances"
        :key="instance?.project_id || instance?.id"
        display="card"
        :instance="instance"
        class="row-instance"
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
  min-width: 10.5rem;
  max-width: 10.5rem;
}
</style>
