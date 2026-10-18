<template>
	<section class="project-review relative flex overflow-hidden bg-surface-1" aria-label="Project Review">
		<WorkbenchLayout class="min-w-0 flex-1" :workbench="workbench">
			<template v-for="panel in panels" :key="panel.id" #[`panel-${panel.id}`]>
				<ProjectReviewPanel :slot-name="panel.id as ProjectReviewSlot" />
			</template>
			<template #tab="{ panel }">
				<ProjectReviewTabLabel :tab="panel.id as ProjectReviewTab" />
			</template>
			<template v-if="slots.footer" #center-footer>
				<slot name="footer" />
			</template>
		</WorkbenchLayout>
	</section>
</template>

<script setup lang="ts">
import {
	createWorkbench,
	WORKBENCH_MODES,
	type WorkbenchPanel,
	WorkbenchLayout,
} from '@modrinth/ui'
import { computed, onScopeDispose, watch, watchEffect } from 'vue'

import { useModerationKeybinds } from '~/composables/moderation'

import { injectReviewContext } from '../review-panel/context'
import {
	injectReviewShortcuts,
	reviewShortcutBlocked,
	useReviewEventListener,
	useReviewShortcut,
} from '../shortcuts'
import { provideProjectReviewContext } from './context'
import { reviewPanelLabels } from './labels'
import ProjectReviewPanel from './panel.vue'
import ProjectReviewTabLabel from './tab.vue'
import type { ProjectReviewSlot, ProjectReviewSlots, ProjectReviewTab } from './types'

const POPOUT_URL = '/project-review-popout.html'

/** Panels that clip their own content rather than scrolling as a whole. */
const UNSCROLLED_SLOTS: ProjectReviewSlot[] = ['right', 'disclosures', 'permissions', 'gallery', 'slicer']

const props = defineProps<{
	tabs: readonly ProjectReviewTab[]
	resetKey: string
	ready: boolean
}>()
const slots = defineSlots<ProjectReviewSlots>()
const shortcuts = injectReviewShortcuts()
const review = injectReviewContext()
const { heldTab, shortcutTab } = review
let heldTabKey: string | undefined

function panelFor(slot: ProjectReviewSlot, region: WorkbenchPanel['region']): WorkbenchPanel {
	return {
		id: slot,
		title: reviewPanelLabels[slot],
		region,
		padded: true,
		scroll: !UNSCROLLED_SLOTS.includes(slot),
		minimumHeight: region === 'center' ? 120 : undefined,
	}
}

const panels = computed<WorkbenchPanel[]>(() => [
	panelFor('left', 'left'),
	panelFor('right', 'right'),
	...props.tabs.map((tab) => panelFor(tab, 'center')),
])

const workbench = createWorkbench({
	panels,
	storageKey: 'moderation.project-review',
	mode: {
		...WORKBENCH_MODES.review,
		popouts: {
			url: POPOUT_URL,
			onWindow: (child) => {
				review.close()
				const unregister = shortcuts.registerWindow(child)
				return () => {
					review.close()
					unregister()
				}
			},
		},
	},
	variant: 'rounded',
	sizes: {
		left: { default: 400, minimum: 200 },
		right: { default: 400, minimum: 200 },
	},
	centerMinimum: 20,
	ready: () => props.ready,
})

const leftVisible = workbench.visible.left
const rightVisible = workbench.visible.right

function openTab(tab: ProjectReviewTab) {
	if (props.tabs.includes(tab)) workbench.revealPanel(tab)
}

provideProjectReviewContext({
	slots,
	leftVisible,
	rightVisible,
	visibleRightWidth: workbench.regionExtents.right,
	toggleSidebar: (side) => workbench.toggleRegion(side),
	revealSlot: (slot) => {
		if (slot === 'left' || slot === 'right') workbench.setRegionVisible(slot, true)
		else openTab(slot)
	},
	openTab,
	resetActiveTabs: workbench.resetActiveTabs,
	resetPanelLayout: workbench.resetLayout,
	selectedTabs: workbench.selectedPanels,
	hasPopouts: workbench.hasPopouts,
	detachedTabs: workbench.detachedPanels,
	poppingOut: workbench.poppingOut,
	popoutTab: (tab) => void workbench.popoutPanel(tab),
	dockTab: workbench.dockPanel,
	closeExternalPanels: workbench.closePopouts,
})

watchEffect(() => {
	document.body.style.setProperty(
		'--project-review-right-panel-width',
		`${workbench.regionExtents.right.value}px`,
	)
})
onScopeDispose(() => {
	document.body.style.removeProperty('--project-review-right-panel-width')
})

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
		if (resetKey && previousResetKey) workbench.resetActiveTabs()
	},
)

useReviewShortcut('toggle-left', () => workbench.toggleRegion('left'))
useReviewShortcut('toggle-right', () => workbench.toggleRegion('right'))
useReviewShortcut('reveal-right', () => workbench.setRegionVisible('right', true))

const keybinds = useModerationKeybinds()
useReviewEventListener('keydown', (event) => {
	if (reviewShortcutBlocked(event)) return
	keybinds.value.handle(event, {
		scope: 'project-review',
		openTab: (tab) => {
			if (!props.tabs.includes(tab)) return
			openTab(tab)
			heldTab.value = tab
			shortcutTab.value = tab
			heldTabKey = event.code || event.key.toLowerCase()
		},
	})
})
useReviewEventListener('keyup', (event) => {
	if ((event.code || event.key.toLowerCase()) === heldTabKey) releaseHeldTab()
})
useReviewEventListener('blur', clearShortcutTab)
onScopeDispose(clearShortcutTab)
</script>

<style scoped>
.project-review {
	height: 100dvh;
	width: 100%;
}
</style>
