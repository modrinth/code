<template>
  <router-link v-if="to.path || to.query || to.startsWith('/')" :to="to" v-bind="$attrs">
    <slot />
  </router-link>
  <a v-else-if="to.startsWith('http')" :href="to" v-bind="$attrs">
    <slot />
  </a>
  <span v-else v-bind="$attrs">
    <slot />
  </span>
</template>

<script setup lang="ts">
defineProps<{
  to: any
}>()

defineOptions({
  inheritAttrs: false,
})
</script>
