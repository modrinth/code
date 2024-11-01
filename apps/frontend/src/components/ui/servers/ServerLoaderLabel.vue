<template>
  <div
    v-if="loader"
    v-tooltip="'Change server loader'"
    class="flex min-w-0 flex-row items-center gap-4 truncate"
  >
    <div v-if="!noSeparator" class="experimental-styles-within h-6 w-0.5 bg-button-border"></div>
    <div class="flex flex-row items-center gap-2">
      <UiServersIconsLoaderIcon :loader="loader" class="flex shrink-0 [&&]:size-5" />
      <NuxtLink
        v-if="isLink"
        :to="serverId ? `/servers/manage/${serverId}/options/loader` : ''"
        class="min-w-0 text-sm font-semibold"
        :class="serverId ? 'hover:underline' : ''"
      >
        {{ loader }} <span v-if="loaderVersion">{{ loaderVersion }}</span>
      </NuxtLink>
      <div v-else class="min-w-0 text-sm font-semibold">
        {{ loader }} <span v-if="loaderVersion">{{ loaderVersion }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  noSeparator?: boolean;
  loader: "Fabric" | "Quilt" | "Forge" | "NeoForge" | "Paper" | "Spigot" | "Bukkit" | "Vanilla";
  loaderVersion: string;
  isLink?: boolean;
}>();

const route = useNativeRoute();
const serverId = route.params.id as string;
</script>
