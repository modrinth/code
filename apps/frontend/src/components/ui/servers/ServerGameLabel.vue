<template>
  <div
    v-if="game"
    v-tooltip="'Change server version'"
    class="min-w-0 flex-none flex-row items-center gap-2 first:!flex"
  >
    <GameIcon aria-hidden="true" class="size-5 shrink-0" />
    <NuxtLink
      v-if="isLink"
      :to="serverId ? `/servers/manage/${serverId}/options/loader` : ''"
      class="min-w-0 truncate text-sm font-semibold"
      :class="serverId ? 'hover:underline' : ''"
    >
      {{ game[0].toUpperCase() + game.slice(1) }} {{ mcVersion }}
    </NuxtLink>
    <div v-else class="min-w-0 truncate text-sm font-semibold">
      {{ game[0].toUpperCase() + game.slice(1) }} {{ mcVersion }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { GameIcon } from "@modrinth/assets";

defineProps<{
  game: string;
  mcVersion: string;
  isLink?: boolean;
}>();

const route = useNativeRoute();
const serverId = route.params.id as string;
</script>
