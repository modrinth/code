import type { Component, ComputedRef, Ref, ShallowRef } from 'vue'

import type { ButtonMenuOption } from '#ui/components'
import { createContext } from '#ui/providers/create-context'

import type { FileTabs } from '../composables/file-tabs'
import type { FileItem, FileSortField } from '../types'
import type { FileInfo, FileManagerContext } from './file-manager'

/**
 * API exposed by each tab's file editor, registered on `FileTabs` on mount
 * since dockview panels aren't reachable via template refs.
 */
export interface FileEditorBridge {
	hasUnsavedChanges: Ref<boolean> | ComputedRef<boolean>
	isFindOpen: Ref<boolean> | ComputedRef<boolean>
	saveFileContent: (exit?: boolean) => Promise<void>
	revertChanges: () => void
	shareToMclogs: () => Promise<void>
	toggleFind: () => void
}

/**
 * The file manager context as seen by the files layout's components. Members shared with
 * {@link FileManagerContext} are passed through from the host, except for the ones overridden
 * here, which additionally respect the busy/read-only state, record undo history and keep the
 * open tabs following moved, renamed and deleted entries.
 */
export interface FileBrowserUIContext extends FileManagerContext {
	baseId: string
	showDebugInfo: ComputedRef<boolean>
	showRefreshButton: ComputedRef<boolean>
	/** Whether the files tab rework feature flag is on, which offers the view settings. */
	reworkEnabled: ComputedRef<boolean>
	/** Whether the advanced view (sidebar tree, tabs and column picker) is in use. */
	advancedView: ComputedRef<boolean>
	setAdvancedView: (value: boolean) => Promise<void>
	/** Whether file and folder icons are tinted by type. */
	coloredIcons: ComputedRef<boolean>
	setColoredIcons: (value: boolean) => void

	items: ComputedRef<FileItem[]>
	filteredItems: ComputedRef<FileItem[]>
	isEditing: ComputedRef<boolean>
	/** Busy, or the active location is read-only. */
	isBusy: ComputedRef<boolean>
	busyTooltip: ComputedRef<string | undefined>
	activeLocation: ComputedRef<FileInfo>
	sidebarOpen: ComputedRef<boolean>
	setSidebarOpen: (value: boolean) => void
	/** Width of the main content column, i.e. the space `FileBrowserPanel` actually has. */
	containerWidth: Ref<number | undefined>

	searchQuery: Ref<string>
	sortField: Ref<FileSortField>
	sortDesc: Ref<boolean>
	handleSort: (field: FileSortField) => void

	/** Selected entries keyed by path; may span directories. */
	selectedItems: Ref<Map<string, FileItem>>
	toggleItemSelection: (item: FileItem) => void
	deselectAll: () => void
	toggleSelectAll: () => void
	allSelected: ComputedRef<boolean>
	someSelected: ComputedRef<boolean>

	editorComponent: ShallowRef<Component | null>
	fileTabs: FileTabs
	/** The editor of the active tab. */
	fileEditorApi: ComputedRef<FileEditorBridge | null>
	hasUnsavedChanges: ComputedRef<boolean>
	saveFileContent: (exit?: boolean) => Promise<void>
	revertChanges: () => void
	/** Shares the active tab's file to mclo.gs. */
	shareEditorToMclogs: () => Promise<void>
	toggleFind: () => void

	/** Navigates the active tab, rather than the host directly. */
	navigateTo: (file: FileInfo) => Promise<void>
	navigateToSegment: (index: number) => void
	handleOpenInNewTab: (item: FileInfo) => void
	handleEditorClose: () => Promise<void>
	handleHomePrefetch: () => void
	handleItemPrefetch: (item: FileInfo) => void

	showCreateModal: (type: 'file' | 'directory') => void
	showRenameModal: (item: FileItem) => void
	showMoveModal: (item: FileItem) => void
	showDeleteModal: (item: FileItem) => void
	showBulkDeleteModal: () => void
	showUnzipFromUrlModal: (cf: boolean) => void

	downloadFile: (file: FileInfo) => Promise<void>
	zipFolder?: (file: FileInfo) => Promise<void>
	handleExtractItem: (item: FileInfo) => Promise<void>

	/** Uploads into the current directory, unless busy or a file is open. */
	uploadFiles: (files: File[]) => void
	handleDropError: (error: unknown) => void
	initiateFileUpload: () => Promise<void>

	handleContextMenu: (event: MouseEvent, options: ButtonMenuOption[]) => void
}

export const [injectFileBrowserUI, provideFileBrowserUI] = createContext<FileBrowserUIContext>(
	'FilePageLayout',
	'fileBrowserUIContext',
)
