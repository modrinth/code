<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
	defineProps<{
		progress: number
		max?: number
	}>(),
	{
		max: 1,
	},
)

const percent = computed(() => props.progress / props.max)
</script>
<template>
	<span class="relative flex items-center justify-center">
		<svg
			width="24"
			height="24"
			xmlns="http://www.w3.org/2000/svg"
			fill="none"
			viewBox="0 0 24 24"
			class="absolute"
		>
			<circle opacity="0.25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
		</svg>
		<svg
			:style="{ '--_progress': `${percent * 100}%` }"
			width="24"
			height="24"
			xmlns="http://www.w3.org/2000/svg"
			fill="none"
			viewBox="0 0 24 24"
			class="absolute progress-circle"
		>
			<circle opacity="0.75" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
		</svg>
	</span>
</template>

<style scoped lang="scss">
@property --_progress {
	syntax: '<percentage>';
	inherits: false;
	initial-value: 0%;
}

.progress-circle {
	transition: --_progress 0.125s ease-in-out;
	mask-image: conic-gradient(
		black 0%,
		black var(--_progress),
		transparent calc(var(--_progress) + 1%),
		transparent 100%
	);
}
</style>
