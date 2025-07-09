<template>
  <nav class="breadcrumbs">
    <template v-for="(link, index) in linkStack" :key="index">
      <RouterLink
        :to="link.href"
        class="breadcrumb goto-link"
        :class="{ trim: link.allowTrimming ? link.allowTrimming : false }"
      >
        {{ link.label }}
      </RouterLink>
      <ChevronRightIcon />
    </template>
    <span class="breadcrumb">{{ currentTitle }}</span>
  </nav>
</template>

<script setup>
import { ChevronRightIcon } from '@modrinth/assets'

defineProps({
  linkStack: {
    type: Array,
    default: () => [],
  },
  currentTitle: {
    type: String,
    required: true,
  },
})
</script>

<style lang="scss" scoped>
.breadcrumbs {
  display: flex;
  margin-bottom: var(--gap-lg);
  align-items: center;
  flex-wrap: wrap;

  svg {
    width: 1.25rem;
    height: 1.25rem;
  }

  a.breadcrumb {
    padding-block: var(--gap-xs);
    &.trim {
      text-overflow: ellipsis;
      white-space: nowrap;
    }
  }
}
</style>
