<template>
	<div class="flex h-full min-h-0 w-full flex-col">
		<div
			v-if="!toolbarCollapsed"
			class="flex min-h-10 shrink-0 flex-wrap items-center gap-2 border-0 border-b border-solid border-surface-5 bg-surface-3 px-2 py-1"
		>
			<slot name="toolbar" />
			<WorkbenchToggles class="ml-auto" @collapse="toolbarCollapsed = true" />
		</div>
		<div
			v-else
			class="flex h-4 shrink-0 items-center justify-center border-0 border-b border-solid border-surface-5 bg-surface-3"
		>
			<button
				v-tooltip="formatMessage(messages.expandToolbar)"
				type="button"
				class="flex h-4 w-12 items-center justify-center rounded-b-md border-none bg-surface-4 p-0 text-secondary hover:text-contrast focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
				:aria-label="formatMessage(messages.expandToolbar)"
				@click="toolbarCollapsed = false"
			>
				<ChevronDownIcon class="size-4" aria-hidden="true" />
			</button>
		</div>
		<GridviewVue
			class="workbench dockview-theme-modrinth min-h-0 w-full flex-1"
			:orientation="Orientation.HORIZONTAL"
			:components="components"
			:proportional-layout="false"
			@ready="workbench.onGridReady"
		/>
		<ContextMenu ref="contextMenu" :label="formatMessage(messages.menuLabel)" />
	</div>
</template>

<script setup lang="ts">
import { ChevronDownIcon } from '@modrinth/assets'
import { useLocalStorage } from '@vueuse/core'
import { GridviewVue, Orientation } from 'dockview-vue'
import { ref, type VNode } from 'vue'

import { ContextMenu } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import WorkbenchDock from './components/WorkbenchDock.vue'
import WorkbenchToggles from './components/WorkbenchToggles.vue'
import { useWorkbenchLayout, WORKBENCH_DOCK_COMPONENT } from './composables/workbench-layout'
import { useWorkbenchMenus } from './composables/workbench-menus'
import { provideWorkbench } from './providers/workbench'
import type {
	WorkbenchDocuments,
	WorkbenchPanel,
	WorkbenchRegion as Region,
	WorkbenchRegionSizes,
} from './types'

const props = defineProps<{
	panels: WorkbenchPanel[]
	/**
	 * Identifies the layout in local storage. `null` keeps it in memory only, e.g. for an embedded
	 * or one-off view that shouldn't change the user's own arrangement.
	 */
	storageKey: string | null
	sizes?: Partial<WorkbenchRegionSizes>
	/** Regions hidden when opened, regardless of how they were left. */
	hiddenRegions?: Region[]
	/** Panels hosted for someone else (e.g. file tabs), which can live in any dock. */
	documents?: WorkbenchDocuments
}>()

const slots = defineSlots<{
	toolbar?: () => VNode[]
	[panel: `panel-${string}`]: (() => VNode[]) | undefined
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	menuLabel: {
		id: 'workbench.menu.label',
		defaultMessage: 'Panel actions',
	},
	expandToolbar: {
		id: 'workbench.expand-toolbar',
		defaultMessage: 'Show toolbar',
	},
})

const components = { [WORKBENCH_DOCK_COMPONENT]: WorkbenchDock }

const toolbarCollapsed = props.storageKey
	? useLocalStorage(`workbench-toolbar-collapsed:${props.storageKey}`, false, {
			initOnMounted: true,
		})
	: ref(false)

const contextMenu = ref<InstanceType<typeof ContextMenu>>()

const layout = useWorkbenchLayout({
	slots,
	panels: () => props.panels,
	storageKey: props.storageKey,
	sizes: props.sizes,
	hiddenRegions: props.hiddenRegions,
	documents: props.documents,
})

const workbench = {
	...layout,
	...useWorkbenchMenus(layout, (event, options) => contextMenu.value?.open(event, options)),
}

provideWorkbench(workbench)

defineExpose({
	visible: workbench.visible,
	setRegionVisible: workbench.setRegionVisible,
	toggleRegion: workbench.toggleRegion,
	regionOf: workbench.regionOf,
	isOpen: workbench.isOpen,
	revealPanel: workbench.revealPanel,
	closePanel: workbench.closePanel,
	movePanel: workbench.movePanel,
	resetLayout: workbench.resetLayout,
})
</script>

<style scoped>
.workbench :deep(.dv-sash:not(.dv-disabled)::after) {
	content: '';
	position: absolute;
	pointer-events: none;
	background-color: var(--color-brand);
	opacity: 0;
}

.workbench :deep(.dv-horizontal > .dv-sash-container > .dv-sash::after) {
	top: 0;
	left: 50%;
	width: 1px;
	height: 100%;
}

.workbench :deep(.dv-vertical > .dv-sash-container > .dv-sash::after) {
	top: 50%;
	left: 0;
	width: 100%;
	height: 1px;
}

.workbench :deep(.dv-sash:not(.dv-disabled):hover::after) {
	opacity: 1;
	transition: opacity 0s 0.3s;
}

.workbench :deep(.dv-sash:not(.dv-disabled):active::after) {
	opacity: 1;
	transition: none;
}

.workbench :deep(.workbench-dock .dv-tabs-and-actions-container) {
	border-bottom: 1px solid var(--surface-5);
}

.workbench :deep(.workbench-dock .dv-tab.dv-active-tab) {
	box-shadow: inset 0 -2px 0 var(--color-brand);
}

.workbench :deep(.workbench-dock .dv-inactive-group .dv-tab.dv-active-tab) {
	box-shadow: inset 0 -2px 0 var(--surface-5);
}
</style>
