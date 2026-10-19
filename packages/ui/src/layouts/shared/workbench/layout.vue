<template>
	<div
		ref="root"
		class="workbench-root dockview-theme-modrinth flex h-full min-h-0 w-full flex-col"
		:class="`workbench-${workbench.variant.value}`"
		@dblclick="workbench.onSashDoubleClick"
	>
		<GridviewVue
			class="workbench-grid min-h-0 w-full flex-1"
			:orientation="Orientation.HORIZONTAL"
			:components="components"
			:proportional-layout="false"
			@ready="workbench.onGridReady"
		/>
		<ContextMenu ref="contextMenu" :label="formatMessage(messages.menuLabel)" />
	</div>
</template>

<script setup lang="ts">
import { GridviewVue, Orientation } from 'dockview-vue'
import { onMounted, ref, type VNode } from 'vue'

import { ContextMenu } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import Dock from './components/dock/index.vue'
import { createWorkbench, WORKBENCH_DOCK_COMPONENT } from './composables/create-workbench'
import { provideWorkbench, type WorkbenchContext } from './providers/workbench'
import type {
	WorkbenchDocuments,
	WorkbenchMode,
	WorkbenchModeName,
	WorkbenchPanel,
	WorkbenchRegion as Region,
	WorkbenchRegionSizes,
	WorkbenchVariant,
} from './types'

const props = defineProps<{
	/**
	 * A workbench made with `createWorkbench`, for hosts that need it before the layout renders.
	 * Without one, the layout makes its own from the props below.
	 */
	workbench?: WorkbenchContext
	panels?: WorkbenchPanel[]
	/**
	 * Identifies the layout in local storage. `null` keeps it in memory only, e.g. for an embedded
	 * or one-off view that shouldn't change the user's own arrangement.
	 */
	storageKey?: string | null
	mode?: WorkbenchModeName | WorkbenchMode
	variant?: WorkbenchVariant
	sizes?: Partial<WorkbenchRegionSizes>
	/** Regions hidden when opened, regardless of how they were left. */
	hiddenRegions?: Region[]
	/** Panels hosted for someone else (e.g. file tabs), which can live in any dock. */
	documents?: WorkbenchDocuments
}>()

const slots = defineSlots<{
	toolbar?: () => VNode[]
	/** Controls at the end of the toolbar, before the layout's own (e.g. settings). */
	'toolbar-end'?: () => VNode[]
	/** Content of a tab, in place of its icon and title. */
	tab?: (props: { panel: WorkbenchPanel; active: boolean; selected: boolean }) => VNode[]
	/** Shown below the center, spanning it. */
	'center-footer'?: () => VNode[]
	[panel: `panel-${string}`]: (() => VNode[]) | undefined
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	menuLabel: {
		id: 'workbench.menu.label',
		defaultMessage: 'Panel actions',
	},
})

const components = { [WORKBENCH_DOCK_COMPONENT]: Dock }

const workbench =
	props.workbench ??
	createWorkbench({
		panels: () => props.panels ?? [],
		storageKey: props.storageKey ?? null,
		mode: props.mode,
		variant: props.variant,
		sizes: props.sizes,
		hiddenRegions: props.hiddenRegions,
		documents: props.documents,
	})

workbench.slots = slots

const root = ref<HTMLElement | null>(null)
const contextMenu = ref<InstanceType<typeof ContextMenu>>()

workbench.setMenuOpener((event, options) => contextMenu.value?.open(event, options))
onMounted(() => (workbench.root.value = root.value))

provideWorkbench(workbench)

defineExpose({
	workbench,
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
