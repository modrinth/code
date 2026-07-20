<script setup lang="ts">
withDefaults(
	defineProps<{
		label?: string
	}>(),
	{
		label: undefined,
	},
)
</script>

<template>
	<div
		role="group"
		:aria-label="label"
		class="inline-flex [&>[data-button]:focus-visible]:z-10 [&>[data-button]:not(:first-child)]:rounded-l-none [&>[data-button]:not(:last-child)]:rounded-r-none"
	>
		<slot />
	</div>
</template>
