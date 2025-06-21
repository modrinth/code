<template>
  <div
    v-if="game"
    :v-tooltip="linkComponent ? 'Change server version' : undefined"
    class="min-w-0 flex-none flex-row items-center gap-2 first:!flex"
  >
    <GameIcon aria-hidden="true" class="size-5 shrink-0" />

    <template v-if="linkComponent"
      ><component
        :is="linkComponent"
        :to="serverId ? `/servers/manage/${serverId}/options/loader` : undefined"
        :class="`flex min-w-0 items-center truncate text-sm font-semibold ${
          serverId ? 'hover:underline' : ''
        }`"
      >
        <div class="flex flex-row items-center gap-1">
          {{ game[0].toUpperCase() + game.slice(1) }}
          <span v-if="mcVersion">{{ mcVersion }}</span>
          <span v-else class="inline-block h-3 w-12 animate-pulse rounded bg-button-border"></span>
        </div> </component
    ></template>
  </div>
</template>

<script setup lang="ts">
import { GameIcon } from '@modrinth/assets'
import type { Component } from 'vue'

defineProps<{
  game: string
  mcVersion: string
  serverId: string
  linkComponent?: Component
}>()
</script>
