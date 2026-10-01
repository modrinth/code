import type { Kyros } from '@modrinth/api-client'
import type { ComputedRef, Ref } from 'vue'

import { createContext } from '#ui/providers/create-context'

import type {
	EditingFile,
	ExtractDryRunResult,
	FileItem,
	FileOperation,
	UploadState,
} from '../types'

export interface DirectoryEntries {
	items: ComputedRef<FileItem[]>
	isLoading: Ref<boolean>
	loadError: Ref<Error | null>
}

export interface DirectoryQuery extends DirectoryEntries {
	filesReadyPending: ComputedRef<boolean>
}

/**
 * Lazily loaded, cached directory listings used by the sidebar tree. Paths are absolute
 * (`/`, `/config`, `/config/sub`), independent of `currentPath`.
 */
export interface DirectoryTree {
	/** Returns the (cached) listing for `path`, starting to load it on first access. */
	get: (path: string) => DirectoryEntries
	prefetch: (path: string) => void
	/** Absolute paths of the directories expanded in the tree. Owned by the host so it survives remounts. */
	expandedEntries: Ref<string[]>
}

export interface FileManagerContext {
	currentItems: ComputedRef<FileItem[]>
	directoryTree: DirectoryTree

	loading: ComputedRef<boolean>
	error: ComputedRef<Error | null>

	currentPath: Ref<string>
	navigateTo: (path: string) => void

	editingFile: Ref<EditingFile | null>
	startEditing: (file: EditingFile) => void
	stopEditing: () => void

	createItem: (name: string, type: 'file' | 'directory') => Promise<void>
	renameItem: (path: string, newName: string) => Promise<void>
	moveItem: (source: string, destination: string) => Promise<void>
	deleteItem: (path: string, recursive: boolean) => Promise<void>

	readFile: (path: string) => Promise<string>
	readFileAsBlob: (path: string) => Promise<Blob>
	writeFile: (path: string, content: string) => Promise<void>
	downloadFile: (path: string, fileName: string) => Promise<void>
	statFile?: (path: string) => Promise<Kyros.Files.v1.FileStatResponse>
	zipFolder?: (path: string) => Promise<void>
	zipPaths?: (parent: string, include: string[], target: string) => Promise<void>

	uploadFiles: (files: File[]) => void
	cancelUpload?: () => void
	uploadState?: Ref<UploadState> | ComputedRef<UploadState>

	isRefreshing: Ref<boolean>
	refresh: () => Promise<void>

	isBusy?: Ref<boolean> | ComputedRef<boolean>
	busyTooltip?: Ref<string | undefined> | ComputedRef<string | undefined>
	busyWarning?: Ref<string | null> | ComputedRef<string | null>
	isReadOnly?: (path: string) => boolean
	readOnlyReason?: Ref<string> | ComputedRef<string>

	extractFile?: (
		path: string,
		override: boolean,
		dry: boolean,
	) => Promise<ExtractDryRunResult | void>
	activeOperations?: Ref<FileOperation[]> | ComputedRef<FileOperation[]>
	dismissOperation?: (id: string, action: 'dismiss' | 'cancel') => void

	prefetchDirectory?: (path: string) => void
	prefetchFile?: (path: string) => void

	showInstallFromUrl?: boolean
	basePath?: Ref<string> | ComputedRef<string>
	openInFolder?: (path: string) => void

	downloadButtonLabel?: string
	uploadingLabel?: (completed: number, total: number) => string

	canRestart?: boolean
	restartServer?: () => Promise<void>
	canShareToMclogs?: boolean
	shareToMclogs?: (content: string) => Promise<void>
}

export const [injectFileManager, provideFileManager] = createContext<FileManagerContext>(
	'FilePageLayout',
	'fileManagerContext',
)
