import type { Component, ComputedRef, Ref, ShallowRef } from 'vue'

import type { ButtonMenuOption } from '#ui/components'
import type { FileInfo } from '#ui/layouts/shared/files-tab/providers/file-manager.ts'
import { createContext } from '#ui/providers/create-context'

import type { FileTabs } from '../composables/file-tabs'
import type { FileItem, FileSortField } from '../types'

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

export interface FileBrowserUIContext {
	baseId: string
	showDebugInfo: ComputedRef<boolean>
	showRefreshButton: ComputedRef<boolean>

	items: ComputedRef<FileItem[]>
	filteredItems: ComputedRef<FileItem[]>
	isEditing: ComputedRef<boolean>
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
	shareToMclogs: () => Promise<void>
	toggleFind: () => void

	navigateToSegment: (index: number) => void
	handleNavigateTo: (item: FileInfo) => void
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

	handleDownload: (item: FileItem) => Promise<void>
	handleZip: (item: FileItem) => Promise<void>
	handleDirectMove: (moveData: FileInfo, destination: string) => Promise<void>
	handleExtractItem: (item: FileInfo) => Promise<void>

	handleDroppedFiles: (files: File[]) => void
	handleDropError: (error: unknown) => void
	initiateFileUpload: () => Promise<void>

	handleContextMenu: (event: MouseEvent, options: ButtonMenuOption[]) => void
}

export const [injectFileBrowserUI, provideFileBrowserUI] = createContext<FileBrowserUIContext>(
	'FilePageLayout',
	'fileBrowserUIContext',
)
