<template>
	<section
		class="project-review relative flex overflow-hidden bg-bg"
		:aria-label="formatMessage(messages.title)"
		@dblclick="onDividerDoubleClick"
	>
		<div class="min-h-0 min-w-0 flex-1 overflow-x-auto">
			<ProjectReviewColumns
				class="project-review-dock h-full"
				:style="{
					minWidth: `${workspacePanelSizes.center.minimum + (leftVisible ? workspacePanelSizes.left.minimum : 0) + (rightVisible ? workspacePanelSizes.right.minimum : 0)}px`,
				}"
			/>
		</div>
	</section>
</template>

<script setup lang="ts">
import 'dockview-vue/dist/styles/dockview.css'

import { useVIntl } from '@modrinth/ui'
import { useEventListener } from '@vueuse/core'
import { onScopeDispose, watch } from 'vue'

import { useModerationKeybinds } from '~/composables/moderation'

import { projectReviewMessages as messages } from '../messages'
import { injectReviewContext } from '../review-panel/context'
import { reviewShortcutBlocked, useReviewShortcut } from '../shortcuts'
import ProjectReviewColumns from './columns.vue'
import { provideProjectReviewContext } from './context'
import { workspacePanelSizes } from './layout-storage'
import type { ProjectReviewSlots, ProjectReviewTab } from './types'
import { useProjectReviewLayout } from './use-layout'

const props = defineProps<{ tabs: readonly ProjectReviewTab[]; resetKey: string }>()
const slots = defineSlots<ProjectReviewSlots>()
const { formatMessage } = useVIntl()
const layout = useProjectReviewLayout(
	(tab) => formatMessage(messages[tab]),
	() => props.tabs,
)
const { leftVisible, rightVisible, onDividerDoubleClick } = layout
const { heldTab, shortcutTab } = injectReviewContext()
let heldTabKey: string | undefined

function releaseHeldTab() {
	heldTab.value = undefined
	heldTabKey = undefined
}

function clearShortcutTab() {
	releaseHeldTab()
	shortcutTab.value = undefined
}

watch(
	() => props.resetKey,
	(resetKey, previousResetKey) => {
		clearShortcutTab()
		if (resetKey && previousResetKey) layout.resetActiveTabs()
	},
)

provideProjectReviewContext({ ...layout, slots })

useReviewShortcut('toggle-left', () => layout.toggleSidebar('left'))
useReviewShortcut('toggle-right', () => layout.toggleSidebar('right'))
useReviewShortcut('reveal-right', () => layout.revealSlot('right'))
useReviewShortcut('toggle-bottom', layout.toggleToolsPanel)

const keybinds = useModerationKeybinds()
useEventListener('keydown', (event) => {
	if (reviewShortcutBlocked(event)) return
	keybinds.value.handle(event, {
		scope: 'project-review',
		openTab: (tab) => {
			if (!props.tabs.includes(tab)) return
			layout.openTab(tab)
			heldTab.value = tab
			shortcutTab.value = tab
			heldTabKey = event.code || event.key.toLowerCase()
		},
	})
})
useEventListener('keyup', (event) => {
	if ((event.code || event.key.toLowerCase()) === heldTabKey) releaseHeldTab()
})
useEventListener('blur', clearShortcutTab)
onScopeDispose(clearShortcutTab)
</script>

<style scoped>
.project-review {
	height: 100dvh;
	width: 100%;
}

.project-review :deep(.project-review-dock) {
	--dv-background-color: var(--color-bg);
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
	transition: opacity 0s 0.5s;
}

.project-review :deep(.dv-sash:not(.dv-disabled):active::after) {
	opacity: 1;
	transition: none;
}
</style>
