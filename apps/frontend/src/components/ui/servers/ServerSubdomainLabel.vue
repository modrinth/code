<template>
  <div v-if="subdomain" class="flex flex-row items-center gap-4">
    <div class="experimental-styles-within h-6 w-0.5 bg-button-border"></div>
    <div class="flex flex-row items-center gap-2">
      <LinkIcon />
      <NuxtLink
        :to="serverId ? `/servers/manage/${serverId}/options/network` : ''"
        class="text-sm font-semibold"
        :class="serverId ? 'hover:underline' : ''"
      >
        {{ subdomain }}.modrinth.gg
      </NuxtLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import { LinkIcon } from "@modrinth/assets";
defineProps<{
  subdomain: string;
}>();

const route = useNativeRoute();
const serverId = computed(() => route.params.id as string);
</script>
