<script setup lang="ts">
import { SpinnerIcon, XCircleIcon } from '@modrinth/assets'

withDefaults(
  defineProps<{
    error?: boolean
  }>(),
  {
    error: false,
  },
)
</script>
<template>
  <div class="flex items-center gap-2 font-semibold" :class="error ? 'text-red' : 'animate-pulse'">
    <XCircleIcon v-if="error" class="w-6 h-6" />
    <SpinnerIcon v-else class="w-6 h-6 animate-spin" />
    <slot v-if="error" name="error" />
    <slot v-else />
  </div>
</template>
<style scoped>
.animate-pulse {
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

@keyframes pulse {
  0%,
  100% {
    scale: 1;
  }
  50% {
    scale: 0.95;
  }
}
</style>
