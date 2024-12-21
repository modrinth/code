<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import { formatCategory } from '@modrinth/utils'
import { GameIcon, LeftArrowIcon } from '@modrinth/assets'
import { Avatar, ButtonStyled } from '@modrinth/ui'

type Instance = {
  game_version: string
  loader: string
  path: string
  install_stage: string
  icon_path?: string
  name: string
}

defineProps<{
  instance: Instance
}>()
</script>

<template>
  <div class="flex justify-between items-center border-0 border-b border-solid border-divider pb-4">
    <router-link
      :to="`/instance/${encodeURIComponent(instance.path)}`"
      tabindex="-1"
      class="flex flex-col gap-4 text-primary"
    >
      <span class="flex items-center gap-2">
        <Avatar
          :src="instance.icon_path ? convertFileSrc(instance.icon_path) : undefined"
          :alt="instance.name"
          size="48px"
        />
        <span class="flex flex-col gap-2">
          <span class="font-extrabold bold text-contrast">
            {{ instance.name }}
          </span>
          <span class="text-secondary flex items-center gap-2 font-semibold">
            <GameIcon class="h-5 w-5 text-secondary" />
            {{ formatCategory(instance.loader) }} {{ instance.game_version }}
          </span>
        </span>
      </span>
    </router-link>
    <ButtonStyled>
      <router-link :to="`/instance/${encodeURIComponent(instance.path)}`">
        <LeftArrowIcon /> Back to instance
      </router-link>
    </ButtonStyled>
  </div>
</template>

<style scoped lang="scss"></style>
