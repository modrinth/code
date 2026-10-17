<template>
	<div class="workbench-dock h-full w-full" @contextmenu="openTabBarMenu">
		<DockviewVue
			class="h-full w-full"
			:theme="theme"
			:components="components"
			:tab-components="tabComponents"
			:right-header-actions-component="workbench.documents?.headerActions"
			:watermark-component="host === 'center' ? workbench.documents?.watermark : undefined"
			default-renderer="always"
			disable-floating-groups
			disable-tabs-overflow-list
			no-panels-overlay="emptyGroup"
			@ready="(event) => workbench.onDockReady(host, event)"
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
import { injectWorkbench } from '../providers/workbench'
import WorkbenchPanel from './WorkbenchPanel.vue'
import WorkbenchTab from './WorkbenchTab.vue'

/** Gridview hands its panels their params and api directly, unlike dockview's nested `params`. */
const props = defineProps<{
	params: WorkbenchDockParams
	api: GridviewPanelApi
	containerApi: GridviewApi
}>()

const workbench = injectWorkbench()
const host = props.params.host

const theme: DockviewTheme = {
	name: 'modrinth-workbench',
	className: 'dockview-theme-modrinth',
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
