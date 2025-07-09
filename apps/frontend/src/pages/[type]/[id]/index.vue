<template>
  <section class="normal-page__content">
    <div v-if="project.body" class="card">
      <ProjectPageDescription :description="project.body" />
    </div>
  </section>
</template>

<script setup>
import { ProjectPageDescription } from "@modrinth/ui";

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
