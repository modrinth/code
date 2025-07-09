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
import { formatCategory } from '@modrinth/utils'

defineProps({
  categories: {
    type: Array,
    default() {
      return []
    },
  },
})
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
    align-items: center;

    &:not(.version-badge) {
      color: var(--color-gray);
    }

    svg {
      width: 1rem;
      margin-right: 0.2rem;
    }
  }
}
</style>
