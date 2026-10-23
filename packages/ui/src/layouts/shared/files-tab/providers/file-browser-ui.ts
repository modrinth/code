import type { ComputedRef, Ref } from 'vue'

import type { ButtonMenuOption } from '#ui/components'
import { createContext } from '#ui/providers/create-context'

import type { DirectoryView } from '../composables/directory-view'
import type { FileTabs } from '../composables/file-tabs'
import type { FilesBrowsing, FilesSidebarToggleLocation } from '../settings'
import type { FileItem } from '../types'
import type {
	FileContentVariants,
	FileInfo,
	FileInterpretation,
	FileManagerContext,
} from './file-manager'

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
	/** Whether the sidebar search notes that it only covers folders opened so far. */
	searchLoadedOnlyNotice: ComputedRef<boolean>
	/** Whether the files tab rework feature flag is on, which offers the view settings. */
	canUseAdvancedView: ComputedRef<boolean>
	/** Whether the advanced view (sidebar tree, tabs and column picker) is in use. */
	advancedView: ComputedRef<boolean>
	setAdvancedView: (value: boolean) => Promise<void>
	/** Smaller navigation whose breadcrumbs wrap instead of scrolling, for tighter spaces. */
	compactNavigation: ComputedRef<boolean>
	/** Whether file and folder icons are tinted by type. */
	coloredIcons: ComputedRef<boolean>
	setColoredIcons: (value: boolean) => void
	/** Whether the tree folds chains of folders that each hold just one folder into a single row. */
	compactFolders: ComputedRef<boolean>
	setCompactFolders: (value: boolean) => void
	/** The content variants of the file a tab shows, with the one picked for it (or the default). */
	contentVariantOf: (
		tabId: string,
		file: FileInfo<'file'>,
	) => (FileContentVariants & { selected: string }) | null
	setContentVariant: (tabId: string, file: FileInfo<'file'>, variant: string) => void
	/** How formatted JSON is indented, as `JSON.stringify` takes it. */
	jsonIndent: ComputedRef<string | number>
	/**
	 * How a tab shows the file it's on: the way it picked, else formatted JSON for JSON files that
	 * can't be edited (when the settings say so), else `auto`.
	 */
	interpretationOf: (tabId: string, file: FileInfo<'file'>) => FileInterpretation
	setInterpretation: (
		tabId: string,
		file: FileInfo<'file'>,
		interpretation: FileInterpretation,
	) => void
	/** Whether the tab strip offers a button for opening a new tab. */
	showNewTabButton: ComputedRef<boolean>
	/** The user's "new tab button" setting, which also shows a workbench's add menus. */
	newTabButton: ComputedRef<boolean>
	/** How folders are browsed: in tabs listing them, in the sidebar alone, or in tabs of trees. */
	browsing: ComputedRef<FilesBrowsing>
	/** Brings forward a tab already on the entry, or opens one for it. */
	revealInTab: (item: FileInfo) => void
	/** Where the user keeps the button for showing and hiding the folder tree. */
	sidebarToggleLocation: ComputedRef<FilesSidebarToggleLocation>
	setSidebarToggleLocation: (location: FilesSidebarToggleLocation) => void

	/**
	 * Whether the tabs are hosted by an enclosing workbench, so each tab shows its own navigation
	 * and listing rather than the shared ones of `FileBrowserPanel`.
	 */
	inWorkbench: boolean
	/** The view of the given tab: its location, listing, search and sort. */
	viewFor: (tabId: string) => DirectoryView
	/** The active tab's view. */
	activeView: ComputedRef<DirectoryView>
	isEditing: ComputedRef<boolean>
	/** Busy, or the active location is read-only. */
	isBusy: ComputedRef<boolean>
	busyTooltip: ComputedRef<string | undefined>
	activeLocation: ComputedRef<FileInfo>
	sidebarOpen: ComputedRef<boolean>
	setSidebarOpen: (value: boolean) => void

	/** Selected entries keyed by path; may span directories and tabs. */
	selectedItems: Ref<Map<string, FileItem>>
	toggleItemSelection: (item: FileItem) => void
	deselectAll: () => void

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
	/** Whether the entry opens in a tab, i.e. it is a directory or a file the editor can show. */
	canOpenInTab: (file: FileInfo) => boolean
	handleEditorClose: () => Promise<void>
	handleHomePrefetch: () => void
	handleItemPrefetch: (item: FileInfo) => void

	showCreateModal: (type: 'file' | 'directory') => void
	showRenameModal: (item: FileInfo) => void
	showMoveModal: (item: FileInfo) => void
	showDeleteModal: (item: FileInfo) => void
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
