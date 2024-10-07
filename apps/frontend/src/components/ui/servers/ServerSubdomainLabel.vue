<template>
  <div v-if="subdomain" class="flex flex-row items-center gap-4">
    <div class="experimental-styles-within h-6 w-0.5 bg-button-border"></div>
    <div class="flex flex-row items-center gap-2">
      <LinkIcon />
      <span class="text-sm font-semibold"> {{ subdomain }}.modrinth.gg </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { LinkIcon } from "@modrinth/assets";
defineProps<{
  subdomain: string;
}>();
</script>
