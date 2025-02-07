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
      class="flex min-w-0 items-center truncate text-sm font-semibold"
      :class="serverId ? 'hover:underline' : ''"
    >
      <div class="flex flex-row items-center gap-1">
        {{ game[0].toUpperCase() + game.slice(1) }}
        <span v-if="mcVersion">{{ mcVersion }}</span>
        <span v-else class="inline-block h-3 w-12 animate-pulse rounded bg-button-border"></span>
      </div>
    </NuxtLink>
    <div v-else class="flex min-w-0 flex-row items-center gap-1 truncate text-sm font-semibold">
      {{ game[0].toUpperCase() + game.slice(1) }}
      <span v-if="mcVersion">{{ mcVersion }}</span>
      <span v-else class="inline-block h-3 w-16 animate-pulse rounded bg-button-border"></span>
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
