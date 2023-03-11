<script setup>
import { ChevronLeftIcon, ChevronRightIcon } from 'omorphia'
import Instance from '@/components/ui/Instance.vue'
import News from '@/components/ui/News.vue'

const props = defineProps({
  instances: Array,
  news: Array,
  label: String,
})

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
    </div>
    <section class="mods">
      <Instance
        v-for="instance in props.instances"
        :key="instance.id"
        display="gallery"
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

  &:nth-child(even) {
    background-color: var(--color-raised-bg);
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

  .mods {
    width: 100%;
    display: grid;
    grid-template-columns: repeat(auto-fill, 9rem);
    transition: all ease-in-out 0.4s;
    gap: 1rem;
  }
}
</style>
