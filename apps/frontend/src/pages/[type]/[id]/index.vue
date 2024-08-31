<template>
  <section class="normal-page__content">
    <div
      v-if="project.body"
      class="markdown-body card"
      v-html="renderHighlightedString(project.body || '')"
    />
  </section>
</template>

<script setup>
import { renderHighlightedString } from "~/helpers/highlight.js";

defineProps({
  project: {
    type: Object,
    default() {
      return {};
    },
  },
  versions: {
    type: Array,
    default() {
      return {};
    },
  },
  members: {
    type: Array,
    default() {
      return {};
    },
  },
  organization: {
    type: Object,
    default() {
      return {};
    },
  },
});
</script>
