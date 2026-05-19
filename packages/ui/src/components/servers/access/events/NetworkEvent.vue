<template>
	<BaseEvent>
		{{ kind === 'added' ? 'Added port allocation' : 'Removed port allocation' }}
		<span class="font-mono font-semibold text-contrast">{{ port }}</span>
	</BaseEvent>
</template>

<script setup lang="ts">
import BaseEvent from './BaseEvent.vue'

defineProps<{
	kind: 'added' | 'removed'
	port: number
}>()
</script>
