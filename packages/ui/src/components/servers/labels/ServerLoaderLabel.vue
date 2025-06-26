<template>
  <div v-tooltip="'Change server loader'" class="flex min-w-0 flex-row items-center gap-4 truncate">
    <div v-if="!noSeparator" class="experimental-styles-within h-6 w-0.5 bg-button-border"></div>
    <div class="flex flex-row items-center gap-2">
      <LoaderIcon v-if="loader" :loader="loader" class="flex shrink-0 [&&]:size-5" />
      <div v-else class="size-5 shrink-0 animate-pulse rounded-full bg-button-border"></div>

      <template v-if="isLink && linkComponent">
        <component
          :is="linkComponent"
          :to="serverId ? `/servers/manage/${serverId}/options/loader` : ''"
          class="flex min-w-0 items-center text-sm font-semibold"
          :class="serverId ? 'hover:underline' : ''"
        >
          <span v-if="loader">
            {{ loader }}
            <span v-if="loaderVersion">{{ loaderVersion }}</span>
          </span>
          <span v-else class="flex gap-2">
            <span class="inline-block h-4 w-12 animate-pulse rounded bg-button-border"></span>
            <span class="inline-block h-4 w-12 animate-pulse rounded bg-button-border"></span>
          </span>
        </component>
      </template>

      <div v-else class="min-w-0 text-sm font-semibold">
        <span v-if="loader">
          {{ loader }}
          <span v-if="loaderVersion">{{ loaderVersion }}</span>
        </span>
        <span v-else class="flex gap-2">
          <span class="inline-block h-4 w-12 animate-pulse rounded bg-button-border"></span>
          <span class="inline-block h-4 w-12 animate-pulse rounded bg-button-border"></span>
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import { LoaderIcon, type LoaderTag } from '@modrinth/ui'

defineProps<{
  noSeparator?: boolean
  loader?: 'Fabric' | 'Quilt' | 'Forge' | 'NeoForge' | 'Paper' | 'Spigot' | 'Bukkit' | 'Vanilla'
  loaderVersion?: string
  isLink?: boolean
  linkComponent?: Component
  serverId: string
  tags: {
    loaders: LoaderTag[]
  }
}>()
</script>
