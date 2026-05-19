<template>
	<BaseEvent>
		{{ label }}
	</BaseEvent>
</template>

<script setup lang="ts">
import BaseEvent from './BaseEvent.vue'

defineProps<{
	label: string
}>()
</script>
