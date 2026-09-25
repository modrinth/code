<template>
	<div class="project-review-tabs h-full">
		<DockviewVue
			class="project-review-dock h-full"
			:theme="theme"
			dnd-strategy="pointer"
			disable-tabs-overflow-list
			disable-floating-groups
			default-tab-component="ProjectReviewTab"
			prefix-header-actions-component="ProjectReviewLeftControl"
			right-header-actions-component="ProjectReviewRightControl"
			@ready="onTabsReady"
		/>
	</div>
</template>

<script setup lang="ts">
import { type DockviewTheme, DockviewVue } from 'dockview-vue'

import { injectProjectReviewContext } from '../context'
import ProjectReviewLeftControl from '../controls/left-control.vue'
import ProjectReviewRightControl from '../controls/right-control.vue'
import ProjectReviewPanel from '../panel.vue'
import ProjectReviewTab from './tab.vue'

defineOptions({
	components: {
		ProjectReviewLeftControl,
		ProjectReviewPanel,
		ProjectReviewRightControl,
		ProjectReviewTab,
	},
	inheritAttrs: false,
})

const { onTabsReady } = injectProjectReviewContext()
const theme: DockviewTheme = {
	name: 'modrinth',
	className: 'project-review-dock',
	dndTabIndicator: 'line',
	tabAnimation: 'default',
}
</script>

<style scoped>
.project-review-tabs :deep(.project-review-dock) {
	--dv-activegroup-visiblepanel-tab-color: var(--color-green);
	--dv-inactivegroup-visiblepanel-tab-color: var(--color-green);
	--dv-activegroup-hiddenpanel-tab-color: var(--color-text-default);
	--dv-inactivegroup-hiddenpanel-tab-color: var(--color-text-default);
	--dv-icon-hover-background-color: var(--surface-3);
	--dv-tabs-container-scrollbar-color: var(--surface-5);
	--dv-tab-divider-color: transparent;
	--dv-tabs-and-actions-container-height: 32px;
	--dv-tabs-and-actions-container-font-size: 14px;
	--dv-drag-over-background-color: color-mix(in srgb, var(--color-text-default) 5%, transparent);
	--dv-drag-over-border-color: color-mix(in srgb, var(--color-text-default) 10%, transparent);
	--dv-drag-over-border: 1px solid var(--dv-drag-over-border-color);
}

.project-review-tabs :deep(.dv-tabs-and-actions-container) {
	height: auto;
	min-height: var(--dv-tabs-and-actions-container-height);
	align-items: flex-start;
	padding: 10px 6px 0 10px;
	gap: 4px;
}

.project-review-tabs :deep(.dv-tabs-container) {
	@apply rounded-xl border border-solid border-surface-5 p-0.5 shadow-sm;

	flex-wrap: wrap;
	height: auto;
	overflow: visible;
	align-content: flex-start;
	gap: 4px;
}

.project-review-tabs :deep(.dv-pre-actions-container),
.project-review-tabs :deep(.dv-left-actions-container),
.project-review-tabs :deep(.dv-right-actions-container),
.project-review-tabs :deep(.dv-void-container) {
	height: calc(var(--dv-tabs-and-actions-container-height) + 6px);
}

.project-review-tabs :deep(.dv-tab) {
	height: var(--dv-tabs-and-actions-container-height);
	padding: 0;
	border-radius: 10px;
}

.project-review-tabs :deep(.dv-tab:focus-visible::after),
.project-review-tabs :deep(.dv-tab:has(:focus-visible)::after) {
	border-radius: 10px;
	box-shadow: 0 0 0 4px var(--color-brand-shadow);
}

.project-review-tabs
	:deep(
		.dv-tab > .dv-drop-target-dropzone > .dv-drop-target-selection.dv-drop-target-selection-line
	) {
	position: absolute;
	width: 2px !important;
	background-color: var(--color-text-default);
	border: none;
	transition: none;
}

.project-review-tabs :deep(.dv-tab > .dv-drop-target-dropzone > .dv-drop-target-left) {
	left: -1px !important;
	right: auto !important;
}

.project-review-tabs :deep(.dv-tab > .dv-drop-target-dropzone > .dv-drop-target-right) {
	left: auto !important;
	right: -1px !important;
}
</style>
