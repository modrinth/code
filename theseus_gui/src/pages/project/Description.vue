<template>
  <Card>
    <div class="markdown-body" v-html="renderHighlightedString(project?.body ?? '')" />
  </Card>
</template>

<script setup>
import { Card, renderHighlightedString } from 'omorphia'

defineProps({
  project: {
    type: Object,
    default: () => {},
  },
})
</script>

<script>
export default {
  name: 'Description',
}
</script>

<style scoped lang="scss">
.markdown-body {
  :deep(hr),
  :deep(h1),
  :deep(h2) {
    max-width: max(60rem, 90%);
  }

  :deep(ul),
  :deep(ol) {
    margin-left: 2rem;
  }
}
</style>
