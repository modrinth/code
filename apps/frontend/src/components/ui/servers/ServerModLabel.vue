<template>
  <div>
    <div v-if="modsCount > 0" class="h-6 w-0.5 bg-button-border"></div>
    <div v-if="modsCount > 0" class="flex flex-row items-center gap-2">
      <BoxIcon />
      <span class="text-sm font-semibold">
        {{ modsCount }}
        <span v-if="modsCount === 1">Mod</span>
        <span v-else>Mods</span>
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { BoxIcon } from "@modrinth/assets";
import type { Mod } from "~/types/servers";

const props = defineProps<{
  mods: Mod[];
}>();

const modsCount = computed(() => props.mods?.length ?? 0);
</script>
