import { HomeIcon } from '@modrinth/assets'
import { defineComponent, h } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import type { WorkbenchDocuments } from '#ui/layouts/shared/workbench/types'

import FileEmptyWorkspace from '../components/FileEmptyWorkspace.vue'
import FileTab from '../components/tabs/FileTab.vue'
import FileTabPanel from '../components/tabs/FileTabPanel.vue'
import FileTabsActions from '../components/tabs/FileTabsActions.vue'
import { injectFileBrowserUI } from '../providers/file-browser-ui'
import { infoFrom } from '../utils'
import { FILE_TAB_COMPONENT, FILE_TAB_PANEL_COMPONENT } from './file-tabs'

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

/**
 * The files layout's tabs, as documents any workbench within the files layout (its `workspace`
 * slot, with `workbench` set on it) can host in its docks, next to its own panels.
 */
export function useFileDocuments(): WorkbenchDocuments {
	const ui = injectFileBrowserUI()
	const { formatMessage } = useVIntl()

	return {
		components: { [FILE_TAB_PANEL_COMPONENT]: FileTabPanel },
		tabComponents: { [FILE_TAB_COMPONENT]: FileTab },
		headerActions: FileTabsActions,
		addMenuShown: () => ui.newTabButton.value,
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
}
