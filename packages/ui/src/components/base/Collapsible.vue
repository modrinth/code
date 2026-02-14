<template>
	<div
		class="accordion-content"
		:class="[
			baseClass ?? ``,
			{ open: !collapsed, 'overflow-visible': overflowVisible && !collapsed },
		]"
	>
		<div v-bind="$attrs" :inert="collapsed">
			<slot />
		</div>
	</div>
</template>

<script setup lang="ts">
defineProps<{
	baseClass?: string
	collapsed: boolean
	overflowVisible?: boolean
}>()

defineOptions({
	inheritAttrs: false,
})
</script>
<style scoped>
.accordion-content {
	display: grid;
	grid-template-rows: 0fr;
	transition: grid-template-rows 0.3s ease-in-out;
}

@media (prefers-reduced-motion) {
	.accordion-content {
		transition: none !important;
	}
}

.accordion-content.open {
	grid-template-rows: 1fr;
}

.accordion-content > div {
	overflow: hidden;
}

.accordion-content.overflow-visible > div {
	overflow: visible;
}
</style>
