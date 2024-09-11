<template>
  <div v-if="loader" class="flex flex-row items-center gap-4">
    <div class="experimental-styles-within h-6 w-0.5 bg-button-border"></div>
    <div class="flex flex-row items-center gap-2">
      <UiServersLoaderIcon :loader="loader" />
      <span class="text-sm font-semibold capitalize">
        {{ loader }} <span v-if="loaderVersion">{{ loaderVersion }}</span>
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  loader: string;
  loaderVersion: string;
}>();
</script>
