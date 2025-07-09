<template>
  <div class="flex items-center gap-3">
    <slot></slot>
    <div class="flex flex-col">
      <span class="font-bold">{{ value }}</span>
      <span class="text-secondary">{{ label }}</span>
    </div>
  </div>
</template>

<script setup>
defineProps({
  label: {
    type: String,
    required: true,
  },
  value: {
    type: String,
    required: true,
  },
})
</script>
<style scoped>
:slotted(*) {
  @apply h-6 w-6 text-secondary;
}
</style>
