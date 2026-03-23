<template>{{ instance.name }} overview</template>
<script setup lang="ts">
import type ContextMenu from '@/components/ui/ContextMenu.vue'
import type { GameInstance } from '@/helpers/types'

defineProps<{
	instance: GameInstance
	options: InstanceType<typeof ContextMenu>
	offline: boolean
	playing: boolean
	installed: boolean
}>()
</script>
