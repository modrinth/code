<template>
	<div
		class="workbench-dock flex h-full w-full flex-col"
		:data-workbench-host="host"
		@contextmenu="openTabBarMenu"
	>
		<DockviewVue
			class="min-h-0 w-full flex-1"
			:theme="theme"
			:components="components"
			:tab-components="tabComponents"
			:prefix-header-actions-component="cornerToggles ? WorkbenchCornerToggle : undefined"
			:right-header-actions-component="WorkbenchHeaderActions"
			:watermark-component="WorkbenchWatermark"
			:popout-url="workbench.mode.popouts?.url"
			dnd-strategy="html5"
			disable-floating-groups
			disable-tabs-overflow-list
			no-panels-overlay="emptyGroup"
			@ready="(event) => workbench.onDockReady(host, event)"
		/>
		<component
			:is="workbench.slots['center-footer']"
			v-if="host === 'center' && workbench.slots['center-footer']"
		/>
	</div>
</template>

<script setup lang="ts">
import 'dockview-vue/dist/styles/dockview.css'
import '#ui/styles/dockview-theme.css'

import {
	type DockviewTheme,
	DockviewVue,
	type GridviewApi,
	type GridviewPanelApi,
} from 'dockview-vue'
import { onBeforeUnmount } from 'vue'

import {
	WORKBENCH_PANEL_COMPONENT,
	WORKBENCH_TAB_COMPONENT,
	type WorkbenchDockParams,
} from '../composables/workbench-layout'
import { injectWorkbench, provideWorkbenchDock } from '../providers/workbench'
import WorkbenchCornerToggle from './WorkbenchCornerToggle.vue'
import WorkbenchHeaderActions from './WorkbenchHeaderActions.vue'
import WorkbenchPanel from './WorkbenchPanel.vue'
import WorkbenchTab from './WorkbenchTab.vue'
import WorkbenchWatermark from './WorkbenchWatermark.vue'

/** Gridview hands its panels their params and api directly, unlike dockview's nested `params`. */
const props = defineProps<{
	params: WorkbenchDockParams
	api: GridviewPanelApi
	containerApi: GridviewApi
}>()

const workbench = injectWorkbench()
const host = props.params.host
provideWorkbenchDock(host)

/** Region toggles sit in the corners of the center's tab bars, when the mode has no toolbar for them. */
const cornerToggles = host === 'center' && workbench.mode.cornerToggles

const theme: DockviewTheme = {
	name: 'modrinth-workbench',
	className: 'dockview-theme-modrinth',
	dndOverlayMounting: 'absolute',
	dndTabIndicator: 'line',
}

const components = {
	[WORKBENCH_PANEL_COMPONENT]: WorkbenchPanel,
	...workbench.documents?.components,
}
const tabComponents = {
	[WORKBENCH_TAB_COMPONENT]: WorkbenchTab,
	...workbench.documents?.tabComponents,
}

/** Right-clicks on a tab open the tab's own menu; ones on the rest of the tab bar open this one. */
function openTabBarMenu(event: MouseEvent) {
	if (event.defaultPrevented) return
	const target = event.target as HTMLElement | null
	if (!target?.closest('.dv-tabs-and-actions-container') || target.closest('.dv-tab')) return
	event.preventDefault()
	workbench.openRegionMenu(event, host)
}

onBeforeUnmount(() => workbench.onDockDispose(host))
</script>
