<template>
	<div class="relative overflow-hidden">
		<div
			class="collapsible-region-content"
			:class="{ open: !collapsed || disabled }"
			:style="{ '--collapsed-height': collapsedHeight }"
		>
			<div :class="{ 'pointer-events-none select-none': collapsed }">
				<slot />
			</div>
		</div>

		<div
			v-if="!disabled"
			class="pointer-events-none absolute inset-0 bg-gradient-to-b from-transparent transition-opacity duration-250"
			:class="[gradientTo, { 'opacity-0': !collapsed }]"
		/>

		<div v-if="!collapsed && !disabled" class="top-4 right-4 z-20 group absolute">
			<Button v-tooltip="collapseText" type="quiet" circular icon-only @click="collapsed = true">
				<DropdownIcon class="rotate-180" />
			</Button>
		</div>

		<div v-if="!disabled" class="absolute bottom-4 left-1/2 z-20 -translate-x-1/2">
			<Button type="quiet" class="text-xs" @click="collapsed = !collapsed">
				<DropdownIcon class="transition-transform" :class="{ 'rotate-180': !collapsed }" />
				{{ collapsed ? expandText : collapseText }}
			</Button>
		</div>
	</div>
</template>

<script setup lang="ts">
import { DropdownIcon } from '@modrinth/assets'

import { Button } from '#ui/components/base/buttons'

withDefaults(
	defineProps<{
		expandText?: string
		collapseText?: string
		collapsedHeight?: string
		gradientTo?: string
		disabled?: boolean
	}>(),
	{
		expandText: 'Expand',
		collapseText: 'Collapse',
		collapsedHeight: '8rem',
		gradientTo: 'to-surface-2',
		disabled: false,
	},
)

const collapsed = defineModel<boolean>('collapsed', { default: true })
</script>

<style scoped>
.collapsible-region-content {
	display: grid;
	grid-template-rows: minmax(var(--collapsed-height), 0fr);
	transition: grid-template-rows 500ms var(--ease-out-expo);

	& > div {
		grid-row: 1 / span 2;
	}
}

@media (prefers-reduced-motion) {
	.collapsible-region-content {
		transition: none !important;
	}
}

.collapsible-region-content.open {
	grid-template-rows: minmax(var(--collapsed-height), 1fr);
}

.collapsible-region-content > div {
	overflow: hidden;
}
</style>
