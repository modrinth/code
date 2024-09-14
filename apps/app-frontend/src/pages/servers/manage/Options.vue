<template>
  <div>{{ server }}</div>
  <div>hai adsf s!</div>
  <RouterLink :to="`/servers/manage/${server}/options`">Options</RouterLink>
</template>

<script setup lang="ts">
const props = defineProps<{
  server: string
}>()

const server = props.server
</script>
