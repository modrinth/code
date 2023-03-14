<template>
  <div class="categories">
    <slot />
    <span
      v-for="category in categories"
      :key="category.name"
      v-html="category.icon + formatCategory(category.name)"
    />
  </div>
</template>
<script setup>
import { formatCategory } from '@/components/utils'
</script>
<script>
export default {
  name: 'Categories',
  props: {
    categories: {
      type: Array,
      default() {
        return []
      },
    },
    type: {
      type: String,
      required: true,
    },
  },
  methods: {
    formatCategory,
  },
}
</script>

<style lang="scss" scoped>
.categories {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  gap: var(--gap-sm);

  :deep(span) {
    display: flex;
    flex-direction: row;
    color: var(--color-icon);
    align-items: center;

    svg {
      width: 1rem;
      margin-right: 0.2rem;
    }
  }
}
</style>
