<template>
  <div v-if="loaderData?.icon" v-html="loaderData.icon" />
  <LoaderIcon v-else />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { LoaderIcon } from '@modrinth/assets'
import type { Loaders } from '@modrinth/utils'

export interface LoaderTag {
  name: string
  icon: string
  supported_project_types: string[]
}

const props = defineProps<{
  loader: Loaders
  tags: {
    loaders: LoaderTag[]
  }
}>()

const loaderData = computed(() => {
  return props.tags.loaders.find((tag) => tag.name.toLowerCase() === props.loader.toLowerCase())
})
</script>
