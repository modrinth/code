<script setup lang="ts">
import { ChevronRightIcon } from '@modrinth/assets'
import { Avatar } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import type { GameInstance } from '@/helpers/types'

defineProps<{
  instance: GameInstance
}>()
</script>
<template>
  <span class="flex items-center gap-2 text-lg font-semibold text-primary">
    <Avatar
      :src="instance.icon_path ? convertFileSrc(instance.icon_path) : undefined"
      size="24px"
      :tint-by="instance.path"
    />
    {{ instance.name }} <ChevronRightIcon />
  </span>
</template>
