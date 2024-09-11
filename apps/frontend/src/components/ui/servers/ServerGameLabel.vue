<template>
  <div v-if="game" class="flex flex-row items-center gap-2">
    <img src="~/assets/images/games/minecraft.png" :alt="`${game} Logo`" class="size-5" />
    <span class="text-sm font-semibold">
      {{ game[0].toUpperCase() + game.slice(1) }} {{ mcVersion }}
    </span>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  game: string;
  mcVersion: string;
}>();
</script>
