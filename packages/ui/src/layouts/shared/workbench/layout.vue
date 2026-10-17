<template>
	<div
		ref="root"
		class="workbench-root dockview-theme-modrinth flex h-full min-h-0 w-full flex-col"
		:class="`workbench-${workbench.variant.value}`"
		@dblclick="workbench.onSashDoubleClick"
	>
		<template v-if="workbench.mode.toolbar">
			<div
				v-if="!toolbarCollapsed"
				class="flex min-h-10 shrink-0 flex-wrap items-center gap-2 border-0 border-b border-solid border-surface-4 bg-surface-3 px-2 py-1"
			>
				<slot name="toolbar" />
				<ToolbarControls class="ml-auto" @collapse="toolbarCollapsed = true" />
			</div>
			<div
				v-else
				class="flex shrink-0 items-center justify-center border-0 border-b border-solid border-surface-4 bg-surface-3"
			>
				<IconButton
					v-tooltip="formatMessage(messages.expandToolbar)"
					type="quiet"
					size="xs"
					:label="formatMessage(messages.expandToolbar)"
					@click="toolbarCollapsed = false"
				>
					<ChevronDownIcon aria-hidden="true" />
				</IconButton>
			</div>
		</template>
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
import { ChevronDownIcon } from '@modrinth/assets'
import { useLocalStorage } from '@vueuse/core'
import { GridviewVue, Orientation } from 'dockview-vue'
import { onMounted, ref, type VNode } from 'vue'

import { ContextMenu, IconButton } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import Dock from './components/dock/index.vue'
import ToolbarControls from './components/toolbar-controls.vue'
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
	expandToolbar: {
		id: 'workbench.expand-toolbar',
		defaultMessage: 'Show toolbar',
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

const toolbarCollapsed = props.storageKey
	? useLocalStorage(`workbench-toolbar-collapsed:${props.storageKey}`, false, {
			initOnMounted: true,
		})
	: ref(false)

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
