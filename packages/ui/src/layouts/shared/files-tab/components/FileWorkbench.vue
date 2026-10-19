<template>
	<WorkbenchLayout
		ref="layout"
		:panels="panels"
		:storage-key="storageKey"
		:sizes="sizes"
		:hidden-regions="hiddenRegions"
		:mode="mode"
		:variant="variant"
		:documents="documents"
	>
		<template v-for="(_, name) in $slots" #[name]="slotProps">
			<slot :name="name" v-bind="slotProps ?? {}" />
		</template>
	</WorkbenchLayout>
</template>

<script setup lang="ts">
import { HomeIcon } from '@modrinth/assets'
import { defineComponent, h, ref } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import WorkbenchLayout from '#ui/layouts/shared/workbench/layout.vue'
import type {
	WorkbenchDocuments,
	WorkbenchMode,
	WorkbenchModeName,
	WorkbenchPanel,
	WorkbenchRegion,
	WorkbenchRegionSizes,
	WorkbenchVariant,
} from '#ui/layouts/shared/workbench/types'

import { FILE_TAB_COMPONENT, FILE_TAB_PANEL_COMPONENT } from '../composables/file-tabs'
import { infoFrom } from '../utils'
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
	mode?: WorkbenchModeName | WorkbenchMode
	variant?: WorkbenchVariant
}>()

const ui = injectFileBrowserUI()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	newTab: { id: 'files.tabs.new-tab', defaultMessage: 'New tab' },
})

/** Shown in the center once every tab is closed. */
const EmptyWorkspace = defineComponent({
	name: 'FileWorkbenchEmpty',
	inheritAttrs: false,
	setup: () => () =>
		h('div', { class: 'h-full overflow-auto bg-surface-2' }, [h(FileEmptyWorkspace)]),
})

const documents: WorkbenchDocuments = {
	components: { [FILE_TAB_PANEL_COMPONENT]: FileTabPanel },
	tabComponents: { [FILE_TAB_COMPONENT]: FileTab },
	headerActions: FileTabsActions,
	create: {
		label: () => formatMessage(messages.newTab),
		icon: HomeIcon,
		open: (group) => ui.fileTabs.openTab(infoFrom('/'), { group }),
		shown: () => ui.showNewTabButton.value,
	},
	watermark: EmptyWorkspace,
	confirmMove: (id) => ui.fileTabs.confirmMove(id),
	attach: (api, host) => ui.fileTabs.attach(api, { primary: host === 'center' }),
	detach: (api) => ui.fileTabs.detach(api),
	restore: () => ui.fileTabs.syncPanels(),
}

const layout = ref<InstanceType<typeof WorkbenchLayout>>()

defineExpose({ layout })
</script>
