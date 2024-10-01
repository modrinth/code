<template>
  <div v-if="game" class="flex flex-row items-center gap-2">
    <GameIcon aria-hidden="true" class="size-5 text-secondary" />
    <span class="text-sm font-semibold">
      {{ game[0].toUpperCase() + game.slice(1) }} {{ mcVersion }}
    </span>
  </div>
</template>

<script setup lang="ts">
import { GameIcon } from "@modrinth/assets";

defineProps<{
  game: string;
  mcVersion: string;
}>();
</script>
