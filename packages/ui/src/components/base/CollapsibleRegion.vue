<template>
	<div class="relative overflow-hidden">
		<div
			class="collapsible-region-content"
			:class="{ open: !collapsed }"
			:style="{ '--collapsed-height': collapsedHeight }"
		>
			<div :class="{ 'pointer-events-none select-none pb-16': collapsed }">
				<slot />
			</div>
		</div>

		<div
			v-if="collapsed"
			class="pointer-events-none absolute inset-0 bg-gradient-to-b from-transparent"
			:class="gradientTo"
		/>

		<div class="absolute bottom-4 left-1/2 z-20 -translate-x-1/2">
			<ButtonStyled circular type="transparent">
				<button class="flex items-center gap-1 text-xs" @click="collapsed = !collapsed">
					<ExpandIcon v-if="collapsed" />
					<CollapseIcon v-else />
					{{ collapsed ? expandText : collapseText }}
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import { CollapseIcon, ExpandIcon } from '@modrinth/assets'

import ButtonStyled from './ButtonStyled.vue'

withDefaults(
	defineProps<{
		expandText?: string
		collapseText?: string
		collapsedHeight?: string
		gradientTo?: string
	}>(),
	{
		expandText: 'Expand',
		collapseText: 'Collapse',
		collapsedHeight: '8rem',
		gradientTo: 'to-surface-2',
	},
)

const collapsed = defineModel<boolean>('collapsed', { default: true })
</script>

<style scoped>
.collapsible-region-content {
	display: grid;
	grid-template-rows: 0fr;
	transition: grid-template-rows 0.3s linear;
}

@media (prefers-reduced-motion) {
	.collapsible-region-content {
		transition: none !important;
	}
}

.collapsible-region-content.open {
	grid-template-rows: 1fr;
}

.collapsible-region-content > div {
	overflow: hidden;
	min-height: var(--collapsed-height);
	transition: min-height 0.3s linear;
}

.collapsible-region-content.open > div {
	min-height: 0;
}
</style>
