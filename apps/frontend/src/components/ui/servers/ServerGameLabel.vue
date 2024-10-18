<template>
  <div
    v-if="game"
    v-tooltip="'Change server version'"
    class="flex min-w-0 flex-row items-center gap-2"
  >
    <GameIcon aria-hidden="true" class="size-5 shrink-0 text-secondary" />
    <NuxtLink
      :to="serverId ? `/servers/manage/${serverId}/options/loader` : ''"
      class="min-w-0 truncate text-sm font-semibold"
      :class="serverId ? 'hover:underline' : ''"
    >
      {{ game[0].toUpperCase() + game.slice(1) }} {{ mcVersion }}
    </NuxtLink>
  </div>
</template>

<script setup lang="ts">
import { GameIcon } from "@modrinth/assets";

defineProps<{
  game: string;
  mcVersion: string;
}>();

const route = useNativeRoute();
const serverId = route.params.id as string;
</script>
