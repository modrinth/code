import type { Kyros } from '@modrinth/api-client'
import type { ComputedRef, MaybeRefOrGetter, Ref } from 'vue'

import { createContext } from '#ui/providers/create-context'

import type {
	ExtractDryRunResult,
	FileEntryAction,
	FileEntryDetail,
	FileItem,
	FileOperation,
	UploadState,
} from '../types'

export type FileInfo<T extends FileTypes = FileTypes> = Pick<FileItem, 'path'> & {
	name: string
	type: T
}

export interface FileItemResult<R, T extends FileTypes> extends FileInfo<T> {
	data: ComputedRef<R>
	isLoading: Ref<boolean>
	loadError: Ref<Error | null>
}

export interface FileQueryResult {
	filesReadyPending: ComputedRef<boolean>
}

export type DirectoryResult = FileItemResult<FileItem[], 'directory'>

export type FileResult = FileItemResult<ArrayBuffer | null, 'file'>

export type FileTypes = FileItem['type']

export type FileItemResultFrom<T extends FileTypes> = T extends 'directory'
	? DirectoryResult
	: T extends 'file'
		? FileResult
		: never

/**
 * Lazily loaded, cached directory listings used by the sidebar tree. Paths are absolute
 * (`/`, `/config`, `/config/sub`), independent of `currentPath`.
 */
export interface DirectoryTree {
	/** Returns the (cached) listing for `path`, starting to load it on first access. */
	get: <T extends FileTypes>(file: FileInfo<T>) => FileItemResultFrom<T>
	prefetch: <T extends FileTypes>(file: FileInfo<T>) => void
	/** Absolute paths of the directories expanded in the tree. Owned by the host so it survives remounts. */
	expandedEntries: Ref<string[]>
	/** Every entry of the listings loaded so far, which the sidebar search looks through. */
	loadedEntries?: ComputedRef<FileItem[]>
}

export interface FileManagerContext {
	/**
	 * Identifies the files being browsed (e.g. a server or an instance), so the files layout can
	 * keep a separate, persisted workspace (open tabs and their history) for each of them.
	 */
	workspaceId?: MaybeRefOrGetter<string | null>

	currentFile: ComputedRef<FileInfo>
	currentDirectory: ComputedRef<FileInfo<'directory'>>

	directoryTree: DirectoryTree

	loading: ComputedRef<boolean>
	error: ComputedRef<Error | null>

	navigateTo: (file: FileInfo) => void

	createItem: (name: string, type: 'file' | 'directory') => Promise<FileInfo | null>
	renameItem: (file: FileInfo, newName: string) => Promise<FileInfo | null>
	moveItem: (file: FileInfo, destination: string) => Promise<FileInfo | null>
	/** Resolves `true` once the entry has been deleted. */
	deleteItem: (file: FileInfo, recursive: boolean) => Promise<boolean>

	writeFile: (file: FileInfo, content: ArrayBuffer) => Promise<void>
	downloadFile: (file: FileInfo) => Promise<void>
	statFile?: (path: string) => Promise<Kyros.Files.v1.FileStatResponse>
	zipFolder?: (file: FileInfo<'directory'>) => Promise<void>
	zipPaths?: (
		files: FileInfo[],
		targetDirectory: FileInfo<'directory'>,
		archiveName: string,
	) => Promise<void>

	uploadFiles: (files: File[]) => void
	cancelUpload?: () => void
	uploadState?: Ref<UploadState> | ComputedRef<UploadState>

	isRefreshing: Ref<boolean>
	refresh: () => Promise<void>

	isBusy?: Ref<boolean> | ComputedRef<boolean>
	busyTooltip?: Ref<string | undefined> | ComputedRef<string | undefined>
	busyWarning?: Ref<string | null> | ComputedRef<string | null>
	isReadOnly?: (file: FileInfo | null) => boolean
	readOnlyReason?: Ref<string> | ComputedRef<string>

	extractFile?: (
		path: string,
		override: boolean,
		dry: boolean,
	) => Promise<ExtractDryRunResult | void>
	activeOperations?: Ref<FileOperation[]> | ComputedRef<FileOperation[]>
	dismissOperation?: (id: string, action: 'dismiss' | 'cancel') => void

	showInstallFromUrl?: boolean
	basePath?: Ref<string> | ComputedRef<string>
	openInFolder?: (path: string) => void

	downloadButtonLabel?: string
	uploadingLabel?: (completed: number, total: number) => string

	canRestart?: boolean
	restartServer?: () => Promise<void>
	canShareToMclogs?: boolean
	shareToMclogs?: (content: string) => Promise<void>

	/** Extra actions offered in the menus of files and folders. */
	entryActions?: FileEntryAction[]
	/** Extra details listed in an entry's details, e.g. a hash. */
	entryDetails?: FileEntryDetail[]
}

export const [injectFileManager, provideFileManager] = createContext<FileManagerContext>(
	'FilePageLayout',
	'fileManagerContext',
)
