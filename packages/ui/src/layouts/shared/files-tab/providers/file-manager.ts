import type { Component, ComputedRef, Ref } from 'vue'

import { createContext } from '#ui/providers/create-context'

import type {
	EditingFile,
	ExtractDryRunResult,
	FileItem,
	FileOperation,
	UploadState,
} from '../types'

export interface FileManagerContext {
	// === Data ===
	items: Ref<FileItem[]>
	loading: Ref<boolean>
	error: Ref<Error | null>

	// === Path & Navigation ===
	currentPath: Ref<string>
	navigateTo: (path: string) => void

	// === Editing ===
	editingFile: Ref<EditingFile | null>
	startEditing: (file: EditingFile) => void
	stopEditing: () => void

	// === CRUD ===
	createItem: (name: string, type: 'file' | 'directory') => Promise<void>
	renameItem: (path: string, newName: string) => Promise<void>
	moveItem: (source: string, destination: string) => Promise<void>
	deleteItem: (path: string, recursive: boolean) => Promise<void>

	// === File I/O ===
	readFile: (path: string) => Promise<string>
	readFileAsBlob: (path: string) => Promise<Blob>
	writeFile: (path: string, content: string) => Promise<void>
	downloadFile: (path: string, fileName: string) => Promise<void>

	// === Upload ===
	uploadFiles: (files: File[]) => void
	cancelUpload?: () => void
	uploadState?: Ref<UploadState> | ComputedRef<UploadState>

	// === Refresh ===
	refresh: () => void

	// === Guards (optional) ===
	isBusy?: Ref<boolean> | ComputedRef<boolean>
	busyTooltip?: Ref<string | undefined> | ComputedRef<string | undefined>
	busyWarning?: Ref<string | null> | ComputedRef<string | null>

	// === Extraction (optional — hosting only) ===
	extractFile?: (
		path: string,
		override: boolean,
		dry: boolean,
	) => Promise<ExtractDryRunResult | void>
	activeOperations?: Ref<FileOperation[]> | ComputedRef<FileOperation[]>
	dismissOperation?: (id: string, action: 'dismiss' | 'cancel') => void

	// === Prefetch (optional) ===
	prefetchDirectory?: (path: string) => void
	prefetchFile?: (path: string) => void

	// === Editor (provider supplies the lazy-loaded component) ===
	editorComponent: Ref<Component | null>

	// === Optional capabilities ===
	canRestart?: boolean
	restartServer?: () => Promise<void>
	canShareToMclogs?: boolean
	shareToMclogs?: (content: string) => Promise<void>
}

export const [injectFileManager, provideFileManager] = createContext<FileManagerContext>(
	'FilePageLayout',
	'fileManagerContext',
)
