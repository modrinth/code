<template>
	<BaseEvent>
		<span class="text-secondary">Unknown event</span>
		<span class="font-mono text-secondary"> {{ rawAction }}</span>
	</BaseEvent>
</template>

<script setup lang="ts">
import BaseEvent from './BaseEvent.vue'

defineProps<{
	rawAction: string
}>()
</script>
