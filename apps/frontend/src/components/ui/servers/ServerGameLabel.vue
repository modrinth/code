<template>
  <div v-if="game" class="flex flex-row items-center gap-2">
    <img src="~/assets/images/games/minecraft.png" :alt="`${game} Logo`" class="size-5" />
    <span class="text-sm font-semibold">
      {{ game[0].toUpperCase() + game.slice(1) }} {{ mcVersion }}
    </span>
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "GameInfo",
  props: {
    game: {
      type: String,
      required: true,
    },
    mcVersion: {
      type: String,
      required: true,
    },
  },
});
</script>
