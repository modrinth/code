<template>
	<div
		ref="dockElement"
		class="workbench-dock flex h-full w-full flex-col"
		:data-workbench-host="host"
		@contextmenu="openTabBarMenu"
	>
		<CollapsedStrip v-if="collapsedRegion" :region="collapsedRegion" />
		<div v-show="!collapsedRegion" class="workbench-dockview min-h-0 w-full flex-1">
			<DockviewVue
				class="h-full w-full"
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
				@ready="onReady"
			/>
		</div>
		<WorkbenchToolbar v-if="host === 'center' && workbench.mode.toolbar" />
	</div>
</template>

<script setup lang="ts">
import 'dockview-vue/dist/styles/dockview.css'
import '#ui/styles/dockview-theme.css'

import {
	type DockviewApi,
	type DockviewIDisposable,
	type DockviewReadyEvent,
	type DockviewTheme,
	DockviewVue,
	type GridviewApi,
	type GridviewPanelApi,
} from 'dockview-vue'
import { computed, onBeforeUnmount, onMounted, shallowRef } from 'vue'

import {
	WORKBENCH_PANEL_COMPONENT,
	WORKBENCH_TAB_COMPONENT,
	type WorkbenchDockParams,
} from '../../composables/create-workbench'
import { useDetachedLayout } from '../../composables/use-detached-layout'
import { injectWorkbench, provideWorkbenchDock } from '../../providers/workbench'
import { WORKBENCH_REGIONS, type WorkbenchRegion } from '../../types'
import WorkbenchToolbar from '../toolbar.vue'
import CollapsedStrip from './collapsed-strip.vue'
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

/** The region, while it's collapsed to a strip of its tabs (its dock staying set up, out of sight). */
const collapsedRegion = computed(() =>
	(WORKBENCH_REGIONS as readonly string[]).includes(host) &&
	workbench.collapsed[host as WorkbenchRegion].value
		? (host as WorkbenchRegion)
		: undefined,
)

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
	const groupElement = target.closest('.dv-groupview')
	const group = dockApi?.groups.find((candidate) => candidate.element === groupElement)
	workbench.openRegionMenu(event, host, group?.panels[0]?.id)
}

let dockApi: DockviewApi | undefined
const dockElement = shallowRef<HTMLElement | null>(null)
const dockviewElement = shallowRef<HTMLElement | null>(null)

/**
 * Popped out groups sit in windows of their own, outside the dock's element, so their tab bars'
 * right-clicks are listened for on those windows, for the groups of this dock.
 */
const popoutWindows = new Set<Window>()
let popoutSubscriptions: DockviewIDisposable[] = []

function onPopoutContextMenu(event: MouseEvent) {
	const groupElement = (event.target as HTMLElement | null)?.closest('.dv-groupview')
	if (dockApi?.groups.some((group) => group.element === groupElement)) openTabBarMenu(event)
}

function syncPopoutWindows() {
	const open = new Set(dockApi?.getPopouts().map((popout) => popout.window) ?? [])
	for (const child of popoutWindows) {
		if (open.has(child)) continue
		child.removeEventListener('contextmenu', onPopoutContextMenu)
		popoutWindows.delete(child)
	}
	for (const child of open) {
		if (popoutWindows.has(child)) continue
		child.addEventListener('contextmenu', onPopoutContextMenu)
		popoutWindows.add(child)
	}
}

function onReady(event: DockviewReadyEvent) {
	dockApi = event.api
	for (const subscription of popoutSubscriptions) subscription.dispose()
	popoutSubscriptions = [
		dockApi.onDidAddPopoutGroup(syncPopoutWindows),
		dockApi.onDidRemovePopoutGroup(syncPopoutWindows),
	]
	workbench.onDockReady(host, event)
}

onMounted(() => {
	dockviewElement.value =
		dockElement.value?.querySelector<HTMLElement>(':scope > .workbench-dockview') ?? null
})

useDetachedLayout(dockviewElement, (width, height) => dockApi?.layout(width, height))

onBeforeUnmount(() => {
	for (const subscription of popoutSubscriptions) subscription.dispose()
	for (const child of popoutWindows) child.removeEventListener('contextmenu', onPopoutContextMenu)
	popoutWindows.clear()
	workbench.onDockDispose(host)
})
</script>
