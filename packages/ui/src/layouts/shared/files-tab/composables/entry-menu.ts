import {
	ClipboardCopyIcon,
	DownloadIcon,
	EditIcon,
	FileIcon,
	FolderArchiveIcon,
	FolderOpenIcon,
	PackageOpenIcon,
	PlusIcon,
	RightArrowIcon,
	TrashIcon,
} from '@modrinth/assets'
import { toValue } from 'vue'

import type { ButtonMenuLeafOption, ButtonMenuOption } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectNotificationManager } from '#ui/providers/web-notifications'
import { commonMessages } from '#ui/utils/common-messages'
import { canOpenInFileEditor, getFileExtension } from '#ui/utils/file-extensions'

import { injectFileBrowserUI } from '../providers/file-browser-ui'
import type { FileInfo } from '../providers/file-manager'
import type { FileItem } from '../types'
import { fileIconFor, infoFrom, joinDisplayPath } from '../utils'
import { useFileActions } from './folder-actions'

const messages = defineMessages({
	createZip: {
		id: 'files.row.create-zip',
		defaultMessage: 'Create ZIP',
	},
	openInNewTab: {
		id: 'files.row.open-in-new-tab',
		defaultMessage: 'Open in new tab',
	},
	newOrUpload: {
		id: 'files.row.upload-or-create',
		defaultMessage: 'New Entry',
	},
})

export interface EntryMenuOptions {
	/** Offers selecting the entry from the menu, for rows without a selection checkbox. */
	selectable?: boolean
}

/**
 * Builds the actions menu of a file or folder: the built-in actions followed by the host's
 * {@link FileManagerContext.entryActions}. Shared by the listing rows, the sidebar tree, the
 * breadcrumbs and the empty space of a folder.
 */
export function useEntryMenu() {
	const ui = injectFileBrowserUI()
	const { formatMessage } = useVIntl()
	const { addNotification } = injectNotificationManager()

	const { options: newEntryOptions } = useFileActions(
		(type) => ui.showCreateModal(type),
		() => ui.initiateFileUpload(),
	)

	function copy(text: string, title: string) {
		navigator.clipboard.writeText(text)
		addNotification({ title, type: 'success' })
	}

	function menuFor(entry: FileInfo | FileItem, options: EntryMenuOptions = {}): ButtonMenuOption[] {
		const info = infoFrom(entry)
		const isDirectory = info.type === 'directory'
		const isRoot = isDirectory && info.path === '/'
		const fullPath = joinDisplayPath(toValue(ui.basePath), info.path)

		const readOnly = !!ui.isReadOnly?.(info)
		const writeDisabled = ui.isBusy.value || readOnly
		const writeTooltip = writeDisabled
			? readOnly
				? toValue(ui.readOnlyReason)
				: ui.busyTooltip.value
			: undefined
		const write = { disabled: writeDisabled, tooltip: writeTooltip }

		const canOpenInTab = ui.advancedView.value && (isDirectory || canOpenInFileEditor(info.name))
		const canExtract =
			info.type === 'file' && getFileExtension(info.name) === 'zip' && !!ui.extractFile
		const canZip = isDirectory && !!ui.zipFolder

		const customActions = (ui.entryActions ?? []).filter((action) => action.shown?.(info) ?? true)

		return [
			{
				type: 'submenu',
				id: 'upload-create',
				label: formatMessage(messages.newOrUpload),
				icon: FileIcon,
				options: newEntryOptions.value as ButtonMenuLeafOption[],
			},
			{
				id: 'select-entry',
				label: formatMessage(commonMessages.selectEntryLabel),
				icon: fileIconFor(info).icon,
				shown: !!options.selectable && !isRoot,
				action: () => ui.toggleItemSelection(entry as FileItem),
			},
			{
				id: 'open-in-new-tab',
				label: formatMessage(messages.openInNewTab),
				icon: PlusIcon,
				shown: canOpenInTab,
				action: () => ui.handleOpenInNewTab(info),
			},
			{ type: 'divider' },
			{
				id: 'copy-filename',
				label: formatMessage(commonMessages.copyFilenameButton),
				icon: ClipboardCopyIcon,
				shown: !isRoot,
				action: () => copy(info.name, formatMessage(commonMessages.copiedFilenameLabel)),
			},
			{
				id: 'copy-full-path',
				label: formatMessage(commonMessages.copyFullPathButton),
				icon: ClipboardCopyIcon,
				action: () => copy(fullPath, formatMessage(commonMessages.copiedPathLabel)),
			},
			{
				id: 'open-in-folder',
				label: formatMessage(commonMessages.openInFolderButton),
				icon: FolderOpenIcon,
				shown: !!ui.openInFolder,
				action: () => ui.openInFolder?.(fullPath),
			},
			{ type: 'divider' },
			{
				id: 'extract',
				label: formatMessage(commonMessages.extractButton),
				icon: PackageOpenIcon,
				shown: canExtract,
				...write,
				action: () => ui.handleExtractItem(info),
			},
			{ type: 'divider', shown: canExtract },
			{
				id: 'zip',
				label: formatMessage(messages.createZip),
				icon: FolderArchiveIcon,
				shown: canZip,
				...write,
				action: () => ui.zipFolder?.(info),
			},
			{ type: 'divider', shown: canZip },
			...customActions.map(
				(action): ButtonMenuOption => ({
					id: `custom-${action.id}`,
					label: action.label,
					icon: action.icon,
					tone: action.tone,
					...(action.writes ? write : {}),
					action: () => action.action(info),
				}),
			),
			{ type: 'divider', shown: customActions.length > 0 },
			{
				id: 'rename',
				label: formatMessage(commonMessages.renameButton),
				icon: EditIcon,
				shown: !isRoot,
				...write,
				action: () => ui.showRenameModal(info),
			},
			{
				id: 'move',
				label: formatMessage(commonMessages.moveButton),
				icon: RightArrowIcon,
				shown: !isRoot,
				...write,
				action: () => ui.showMoveModal(info),
			},
			{
				id: 'download',
				label: ui.downloadButtonLabel ?? formatMessage(commonMessages.downloadButton),
				icon: DownloadIcon,
				shown: info.type === 'file',
				action: () => ui.downloadFile(info),
			},
			{
				id: 'delete',
				label: formatMessage(commonMessages.deleteLabel),
				icon: TrashIcon,
				shown: !isRoot,
				...write,
				action: () => ui.showDeleteModal(info),
				tone: 'red',
			},
		]
	}

	/** Opens the entry's menu at the pointer. */
	function openMenu(event: MouseEvent, entry: FileInfo | FileItem, options?: EntryMenuOptions) {
		event.preventDefault()
		ui.handleContextMenu(event, menuFor(entry, options))
	}

	return { menuFor, openMenu }
}
