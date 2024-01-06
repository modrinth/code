<template>
  <div
    v-if="project.body"
    class="markdown-body card"
    v-html="renderHighlightedString(project.body || '')"
  />
</template>

<script>
import { renderHighlightedString } from '~/helpers/highlight.js'

export default defineNuxtComponent({
  props: {
    project: {
      type: Object,
      default() {
        return {}
      },
    },
  },
  methods: { renderHighlightedString },
})
</script>
