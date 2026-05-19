<template>
	<BaseEvent>
		Ran console command
		<span
			v-tooltip="command"
			class="inline-block max-w-full truncate align-bottom font-mono text-contrast"
		>
			{{ command }}
		</span>
	</BaseEvent>
</template>

<script setup lang="ts">
import BaseEvent from './BaseEvent.vue'

defineProps<{
	command: string
}>()
</script>
