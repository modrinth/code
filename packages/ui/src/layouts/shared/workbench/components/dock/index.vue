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
			:prefix-header-actions-component="host === 'center' ? CornerToggle : undefined"
			:right-header-actions-component="HeaderActions"
			:watermark-component="Watermark"
			:popout-url="workbench.mode.popouts?.url"
			dnd-strategy="html5"
			disable-floating-groups
			disable-tabs-overflow-list
			no-panels-overlay="emptyGroup"
			@ready="(event) => workbench.onDockReady(host, event)"
		/>
		<SlotOutlet
			v-if="host === 'center' && workbench.slots['center-footer']"
			:slots="workbench.slots"
			name="center-footer"
		/>
		<WorkbenchToolbar v-if="host === 'center' && workbench.mode.toolbar" />
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
} from '../../composables/create-workbench'
import { injectWorkbench, provideWorkbenchDock } from '../../providers/workbench'
import { SlotOutlet } from '../slot-outlet'
import WorkbenchToolbar from '../toolbar.vue'
import CornerToggle from './corner-toggle.vue'
import HeaderActions from './header-actions.vue'
import Panel from './panel.vue'
import Tab from './tab.vue'
import Watermark from './watermark.vue'

/** Gridview hands its panels their params and api directly, unlike dockview's nested `params`. */
const props = defineProps<{
	params: WorkbenchDockParams
	api: GridviewPanelApi
	containerApi: GridviewApi
}>()

const workbench = injectWorkbench()
const host = props.params.host
provideWorkbenchDock(host)

const theme: DockviewTheme = {
	name: 'modrinth-workbench',
	className: 'dockview-theme-modrinth',
	dndOverlayMounting: 'absolute',
	dndTabIndicator: 'line',
}

const components = {
	[WORKBENCH_PANEL_COMPONENT]: Panel,
	...workbench.documents?.components,
}
const tabComponents = {
	[WORKBENCH_TAB_COMPONENT]: Tab,
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
