import { computed } from 'vue'

import { injectWorkbench } from '#ui/layouts/shared/workbench/providers/workbench'

import { injectFileBrowserUI } from '../providers/file-browser-ui'
import type { FilesSidebarToggleLocation } from '../settings'

/**
 * Where the folder tree's button shows, or `null` within a workbench, which shows and hides the
 * tree as one of its panels from its own layout menu and tab bar.
 */
export function useSidebarToggleLocation() {
	const ui = injectFileBrowserUI()
	const workbench = injectWorkbench(null)

	return computed<FilesSidebarToggleLocation | null>(() =>
		workbench ? null : ui.sidebarToggleLocation.value,
	)
}
