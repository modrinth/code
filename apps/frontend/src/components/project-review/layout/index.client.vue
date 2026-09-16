<template>
	<section
		class="project-review relative flex overflow-hidden bg-surface-2"
		:aria-label="formatMessage(messages.title)"
	>
		<div class="min-h-0 min-w-0 flex-1 overflow-x-auto">
			<ProjectReviewColumns
				class="project-review-dock h-full"
				:style="{
					minWidth: `${320 + (leftVisible ? 180 : 0) + (rightVisible ? 200 : 0)}px`,
				}"
			/>
		</div>
	</section>
</template>

<script setup lang="ts">
import 'dockview-vue/dist/styles/dockview.css'

import { useVIntl } from '@modrinth/ui'

import { projectReviewMessages as messages } from '../messages'
import ProjectReviewColumns from './columns.vue'
import { provideProjectReviewContext } from './context'
import type { ProjectReviewSlots } from './types'
import { useProjectReviewLayout } from './use-layout'

const slots = defineSlots<ProjectReviewSlots>()
const { formatMessage } = useVIntl()
const layout = useProjectReviewLayout((tab) => formatMessage(messages[tab]))
const { leftVisible, rightVisible } = layout

provideProjectReviewContext({ ...layout, slots })
</script>

<style scoped>
.project-review {
	height: 100dvh;
	width: 100%;
}

.project-review :deep(.project-review-dock) {
	--dv-background-color: var(--surface-2);
	--dv-separator-border: var(--surface-4);
	--dv-sash-color: transparent;
	--dv-active-sash-color: transparent;
}

.project-review :deep(.dv-sash:not(.dv-disabled)::after) {
	content: '';
	position: absolute;
	pointer-events: none;
	background-color: var(--color-brand);
	opacity: 0;
}

.project-review :deep(.dv-horizontal > .dv-sash-container > .dv-sash::after) {
	top: 0;
	left: 50%;
	width: 1px;
	height: 100%;
}

.project-review :deep(.dv-vertical > .dv-sash-container > .dv-sash::after) {
	top: 50%;
	left: 0;
	width: 100%;
	height: 1px;
}

.project-review :deep(.dv-sash:not(.dv-disabled):hover::after) {
	opacity: 1;
	transition: opacity 0s 1s;
}

.project-review :deep(.dv-sash:not(.dv-disabled):active::after) {
	opacity: 1;
	transition: none;
}
</style>
