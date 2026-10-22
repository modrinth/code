import { FolderOpenIcon, InfoIcon, SettingsIcon, TerminalSquareIcon } from '@modrinth/assets'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import type { WorkbenchHost, WorkbenchPanel } from '#ui/layouts/shared/workbench/types'
import { commonMessages } from '#ui/utils/common-messages'

import { SETTINGS_PANEL_ID } from './settings'

/** The archive viewer's panels, which any workbench within an archive session can host. */
export const ARCHIVE_PANEL_IDS = ['tree', 'details', 'logs', SETTINGS_PANEL_ID] as const

export type ArchivePanelId = (typeof ARCHIVE_PANEL_IDS)[number]

const DEFAULT_REGIONS: Record<ArchivePanelId, WorkbenchHost> = {
	tree: 'left',
	details: 'right',
	logs: 'bottom',
	[SETTINGS_PANEL_ID]: 'center',
}

const messages = defineMessages({
	tree: { id: 'archive-viewer.panel.tree', defaultMessage: 'Files' },
	details: { id: 'archive-viewer.panel.details', defaultMessage: 'Details' },
	logs: { id: 'archive-viewer.panel.logs', defaultMessage: 'Logs' },
})

export interface ArchivePanelOptions {
	/** Which of the panels to include, all of them by default. */
	include?: MaybeRefOrGetter<readonly ArchivePanelId[]>
	/** Where each panel starts out, for hosts laying them out differently. */
	regions?: Partial<Record<ArchivePanelId, WorkbenchHost>>
	/** Panels that start closed until opened from the workbench's menus. */
	closedByDefault?: readonly ArchivePanelId[]
}

/**
 * The archive viewer's panels, to list among a workbench's own; their content is `ArchivePanel`
 * with the panel's id. The settings panel starts closed unless a host says otherwise.
 */
export function useArchivePanels(options: ArchivePanelOptions = {}) {
	const { formatMessage } = useVIntl()

	return computed<WorkbenchPanel[]>(() => {
		const include = toValue(options.include) ?? ARCHIVE_PANEL_IDS
		const closed = options.closedByDefault ?? [SETTINGS_PANEL_ID]
		const regionOf = (id: ArchivePanelId) => options.regions?.[id] ?? DEFAULT_REGIONS[id]

		const panels: Record<ArchivePanelId, WorkbenchPanel> = {
			tree: {
				id: 'tree',
				title: formatMessage(messages.tree),
				region: 'left',
				icon: FolderOpenIcon,
			},
			details: {
				id: 'details',
				title: formatMessage(messages.details),
				region: 'right',
				icon: InfoIcon,
			},
			logs: {
				id: 'logs',
				title: formatMessage(messages.logs),
				region: 'bottom',
				icon: TerminalSquareIcon,
			},
			[SETTINGS_PANEL_ID]: {
				id: SETTINGS_PANEL_ID,
				title: formatMessage(commonMessages.settingsLabel),
				region: 'right',
				icon: SettingsIcon,
				padded: true,
			},
		}

		return include.map((id) => ({
			...panels[id],
			region: regionOf(id),
			closedByDefault: closed.includes(id),
		}))
	})
}
