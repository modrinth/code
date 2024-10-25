<template>
  <div class="flex flex-col gap-4">
    <div
      v-for="n in count"
      :key="n"
      class="relative h-[128px] w-full animate-pulse rounded-3xl bg-bg-raised p-4"
    />
  </div>
</template>

<script setup lang="ts">
defineProps({
  count: {
    type: Number,
    default: 3,
  },
});
</script>
