<template>{{ instance.name }} overview</template>
<script setup lang="ts">
import type { GameInstance } from '@/helpers/types'
import type ContextMenu from '@/components/ui/ContextMenu.vue'
import type { Version } from '@modrinth/utils'

defineProps<{
  instance: GameInstance
  options: InstanceType<typeof ContextMenu>
  offline: boolean
  playing: boolean
  versions: Version[]
  installed: boolean
}>()
</script>
