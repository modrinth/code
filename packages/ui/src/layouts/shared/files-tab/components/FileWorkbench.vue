<template>
	<WorkbenchLayout
		ref="layout"
		:panels="panels"
		:storage-key="storageKey"
		:sizes="sizes"
		:hidden-regions="hiddenRegions"
		:documents="documents"
	>
		<template v-for="(_, name) in $slots" #[name]="slotProps">
			<slot :name="name" v-bind="slotProps ?? {}" />
		</template>
	</WorkbenchLayout>
</template>

<script setup lang="ts">
import { defineComponent, h, ref } from 'vue'

import WorkbenchLayout from '#ui/layouts/shared/workbench/layout.vue'
import type {
	WorkbenchDocuments,
	WorkbenchPanel,
	WorkbenchRegion,
	WorkbenchRegionSizes,
} from '#ui/layouts/shared/workbench/types'

import { FILE_TAB_COMPONENT, FILE_TAB_PANEL_COMPONENT } from '../composables/file-tabs'
import { injectFileBrowserUI } from '../providers/file-browser-ui'
import FileEmptyWorkspace from './FileEmptyWorkspace.vue'
import FileTab from './tabs/FileTab.vue'
import FileTabPanel from './tabs/FileTabPanel.vue'
import FileTabsActions from './tabs/FileTabsActions.vue'

/**
 * A workbench whose docks also host the files layout's tabs, each showing its own navigation and
 * listing or editor. Placed through the files layout's `workspace` slot, with `workbench` set on
 * it. Takes the same panels and slots as `WorkbenchLayout`.
 */
defineProps<{
	panels: WorkbenchPanel[]
	storageKey: string | null
	sizes?: Partial<WorkbenchRegionSizes>
	hiddenRegions?: WorkbenchRegion[]
}>()

const ui = injectFileBrowserUI()

/** Shown in the center once every tab is closed. */
const EmptyWorkspace = defineComponent({
	name: 'FileWorkbenchEmpty',
	inheritAttrs: false,
	setup: () => () => h('div', { class: 'h-full overflow-auto bg-surface-2' }, [h(FileEmptyWorkspace)]),
})

const documents: WorkbenchDocuments = {
	components: { [FILE_TAB_PANEL_COMPONENT]: FileTabPanel },
	tabComponents: { [FILE_TAB_COMPONENT]: FileTab },
	headerActions: FileTabsActions,
	watermark: EmptyWorkspace,
	confirmMove: (id) => ui.fileTabs.confirmMove(id),
	attach: (api, host) => ui.fileTabs.attach(api, { primary: host === 'center' }),
	detach: (api) => ui.fileTabs.detach(api),
	restore: () => ui.fileTabs.syncPanels(),
}

const layout = ref<InstanceType<typeof WorkbenchLayout>>()

defineExpose({ layout })
</script>
