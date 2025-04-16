<template>
  <AutoLink
    :to="to"
    class="flex mb-3 leading-none items-center gap-1 text-primary text-lg font-bold hover:underline group w-fit"
  >
    <slot />
    <ChevronRightIcon
      class="h-5 w-5 stroke-[3px] group-hover:translate-x-1 transition-transform group-hover:text-brand"
    />
  </AutoLink>
</template>

<script setup lang="ts">
import AutoLink from './AutoLink.vue'
import { ChevronRightIcon } from '@modrinth/assets'

defineProps<{
  to: unknown
}>()
</script>
