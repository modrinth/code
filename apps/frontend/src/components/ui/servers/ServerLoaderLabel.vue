<template>
  <div
    v-if="loader"
    v-tooltip="'Change server loader'"
    class="flex min-w-0 flex-row items-center gap-4"
  >
    <div class="experimental-styles-within h-6 w-0.5 bg-button-border"></div>
    <div class="flex flex-row items-center gap-2">
      <UiServersIconsLoaderIcon :loader="loader" class="flex shrink-0 [&&]:size-5" />
      <NuxtLink
        :to="serverId ? `/servers/manage/${serverId}/options/loader` : ''"
        class="min-w-0 truncate text-sm font-semibold"
        :class="serverId ? 'hover:underline' : ''"
      >
        {{ loader }} <span v-if="loaderVersion">{{ loaderVersion }}</span>
      </NuxtLink>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  loader: string;
  loaderVersion: string;
}>();

const route = useNativeRoute();
const serverId = route.params.id as string;
</script>
