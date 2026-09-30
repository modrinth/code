import type { Component, ComputedRef, Ref, ShallowRef } from 'vue'

import type { ButtonMenuOption } from '#ui/components'
import { createContext } from '#ui/providers/create-context'

import type { FileItem, FileSortField } from '../types'

export type FileInfo = Pick<FileItem, 'name' | 'type' | 'path'>

/**
 * API exposed by whichever panel currently hosts the file editor (FileBrowserPanel),
 * registered here on mount since dockview panels aren't reachable via template refs.
 */
export interface FileEditorBridge {
	hasUnsavedChanges: Ref<boolean> | ComputedRef<boolean>
	isEditingImage: Ref<boolean> | ComputedRef<boolean>
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
	breadcrumbSegments: ComputedRef<string[]>
	sidebarOpen: ComputedRef<boolean>
	setSidebarOpen: (value: boolean) => void;
	containerWidth: Ref<number | undefined>

	searchQuery: Ref<string>
	sortField: Ref<FileSortField>
	sortDesc: Ref<boolean>
	handleSort: (field: FileSortField) => void

	selectedItems: Ref<Set<string>>
	toggleItemSelection: (path: string) => void
	deselectAll: () => void
	toggleSelectAll: () => void
	allSelected: ComputedRef<boolean>
	someSelected: ComputedRef<boolean>

	editorComponent: ShallowRef<Component | null>
	fileEditorApi: ShallowRef<FileEditorBridge | null>
	hasUnsavedChanges: ComputedRef<boolean>
	saveFileContent: (exit?: boolean) => Promise<void>
	revertChanges: () => void
	shareToMclogs: () => Promise<void>
	toggleFind: () => void

	navigateToSegment: (index: number) => void
	handleNavigateToFolder: (item: FileItem) => void
	handleEditFile: (item: FileInfo) => void
	handleEditorClose: () => Promise<void>
	handlePrefetchHome: () => void
	handleItemHover: (item: FileInfo) => void

	showCreateModal: (type: 'file' | 'directory') => void
	showRenameModal: (item: FileItem) => void
	showMoveModal: (item: FileItem) => void
	showDeleteModal: (item: FileItem) => void
	showBulkDeleteModal: () => void
	showUnzipFromUrlModal: (cf: boolean) => void

	handleDownload: (item: FileItem) => Promise<void>
	handleZip: (item: FileItem) => Promise<void>
	handleDirectMove: (moveData: FileInfo & { destination: string }) => void
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
