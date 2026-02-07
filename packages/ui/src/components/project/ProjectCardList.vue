<script setup lang="ts">
withDefaults(
	defineProps<{
		layout?: 'list' | 'grid'
	}>(),
	{
		layout: 'list',
	},
)
</script>
<template>
	<div
		class="gap-3"
		:class="{
			'flex flex-col': layout === 'list',
			'grid grid-project-list': layout === 'grid',
		}"
		role="list"
	>
		<slot />
	</div>
</template>
<style scoped lang="scss">
.grid-project-list {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
}
</style>
