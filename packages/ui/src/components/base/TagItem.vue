<template>
  <button
    v-if="action"
    class="bg-[--_bg-color,var(--color-button-bg)] px-2 py-1 leading-none rounded-full font-semibold text-sm inline-flex items-center gap-1 text-[--_color,var(--color-secondary)] [&>svg]:shrink-0 [&>svg]:h-4 [&>svg]:w-4 border-none transition-transform active:scale-[0.95] cursor-pointer hover:underline"
    @click="action"
  >
    <slot />
  </button>
  <div
    v-else
    class="bg-[--_bg-color,var(--color-button-bg)] px-2 py-1 leading-none rounded-full font-semibold text-sm inline-flex items-center gap-1 text-[--_color,var(--color-secondary)] [&>svg]:shrink-0 [&>svg]:h-4 [&>svg]:w-4"
  >
    <slot />
  </div>
</template>
<script setup lang="ts">
defineProps<{
  action?: (event: MouseEvent) => void
}>()
</script>
