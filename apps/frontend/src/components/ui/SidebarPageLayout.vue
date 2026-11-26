<script setup lang="ts">
defineProps<{
	contentFirst?: boolean
	wideSidebar?: boolean
}>()
</script>

<template>
	<div
		class="grid gap-6"
		:class="wideSidebar ? 'lg:grid-cols-[300px_minmax(0,1fr)]' : 'lg:grid-cols-[1fr_3.25fr]'"
	>
		<div
			v-for="slotName in contentFirst ? ['default', 'sidebar'] : ['sidebar', 'default']"
			:key="slotName"
			class="min-w-0 lg:row-start-1"
			:class="
				slotName === 'sidebar'
					? 'sidebar lg:sticky lg:top-6 lg:col-start-1 lg:self-start'
					: 'lg:col-start-2'
			"
		>
			<div v-if="slotName === 'sidebar'" class="sidebar-scroll">
				<slot name="sidebar" />
			</div>
			<slot v-else />
		</div>
	</div>
</template>

<style scoped>
.sidebar :deep(nav li:not(:first-child) > div) {
	margin-top: 16px;
}

@media (min-width: 1024px) {
	.sidebar-scroll {
		max-height: calc(100dvh - 2rem);
		margin: -0.5rem;
		padding: 0.5rem;
		overflow-y: auto;
	}
}
</style>
