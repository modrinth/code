<template>
  <div>{{ server }}</div>
  <div>hai!</div>
  <RouterLink :to="{ name: 'servers.manage.options', params: { server } }">Options</RouterLink>
</template>

<script setup lang="ts">
const props = defineProps<{
  server: string
}>()

const server = props.server
</script>
