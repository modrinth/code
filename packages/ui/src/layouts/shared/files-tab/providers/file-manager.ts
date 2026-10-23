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

/** The content variant asking for a file's stored bytes, rather than any transformed view of them. */
export const RAW_CONTENT_VARIANT = 'raw'

/**
 * How a tab shows a file: as it would by default (e.g. decompiled, or as an image), or its stored
 * bytes as text, as a hex dump, or as formatted JSON.
 */
export type FileInterpretation = 'auto' | 'text' | 'hex' | 'json'

/**
 * Ways a file's contents can be produced besides the default, e.g. which decompiler reads a
 * class file. Each tab picks one for the file it shows.
 */
export interface FileContentVariants {
	label: string
	options: { id: string; label: string }[]
	/** The one used until the tab picks another. */
	default: string
}

/**
 * Lazily loaded, cached directory listings used by the sidebar tree. Paths are absolute
 * (`/`, `/config`, `/config/sub`), independent of `currentPath`.
 */
export interface DirectoryTree {
	/**
	 * Returns the (cached) listing for `path`, starting to load it on first access. Files can be
	 * read as one of their content variants (see `FileManagerContext.contentVariants`), or as
	 * their stored bytes with `RAW_CONTENT_VARIANT`, which hosts that transform files (e.g. by
	 * decompiling them) honour and others can ignore.
	 */
	get: <T extends FileTypes>(file: FileInfo<T>, variant?: string) => FileItemResultFrom<T>
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
	/**
	 * Whether every entry is read-only (e.g. an archive being inspected), which hides the actions
	 * that create or upload entries rather than only disabling them.
	 */
	readOnly?: boolean
	/** Whether a file can be opened in the editor; defaults to its extension being viewable. */
	canOpenInEditor?: (file: FileInfo) => boolean
	/** The variants a file's contents can be read as, if it has any (e.g. decompilers for class files). */
	contentVariants?: (file: FileInfo<'file'>) => FileContentVariants | null
	/** The editor language of a file, for files whose contents differ from what their extension suggests. */
	editorLanguage?: (file: FileInfo<'file'>) => string | undefined

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
