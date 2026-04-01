<script setup lang="ts">
withDefaults(
	defineProps<{
		layout?: 'list' | 'grid' | 'gallery'
	}>(),
	{
		layout: 'list',
	},
)
</script>
<template>
	<div class="gap-3 flex flex-col" role="list">
		<slot />
	</div>
</template>
