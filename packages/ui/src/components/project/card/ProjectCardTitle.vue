<script setup lang="ts">
defineProps<{
	title: string
	compact?: boolean
}>()
</script>

<template>
	<!-- the weird padding and negative margin are to include the potential hover underline in bounding box which affects rendering on firefox -->
	<span
		class="project-card-title line-clamp-1 pb-[2px] mb-[-2px] break-all font-semibold text-contrast m-0 leading-none smart-clickable:underline-on-hover"
		:class="compact ? 'text-lg' : 'text-xl'"
	>
		{{ title }}
	</span>
</template>
<style scoped>
@container (width < 550px) {
	.project-card-title {
		@apply text-base;
	}
}
</style>
