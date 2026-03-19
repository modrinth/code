export interface FileItem {
	name: string
	type: 'file' | 'directory' | 'symlink'
	path: string
	modified: number
	created: number
	size?: number
	count?: number
	target?: string
}

export interface EditingFile {
	name: string
	path: string
}

export type FileSortField = 'name' | 'size' | 'created' | 'modified'

export type FileViewFilter = 'all' | 'filesOnly' | 'foldersOnly'

export interface FileOperation {
	id?: string
	op: string
	src: string
	state: string
	progress?: number
	bytes_processed?: number
	files_processed?: number
	current_file?: string
}

export interface UndoableOperation {
	type: 'move' | 'rename'
	itemType: string
	fileName: string
}

export interface MoveOperation extends UndoableOperation {
	type: 'move'
	sourcePath: string
	destinationPath: string
}

export interface RenameOperation extends UndoableOperation {
	type: 'rename'
	path: string
	oldName: string
	newName: string
}

export type Operation = MoveOperation | RenameOperation

export interface ExtractDryRunResult {
	modpack_name: string | null
	conflicting_files: string[]
}

export interface FileUploadHandle {
	promise: Promise<void>
	cancel: () => void
	onProgress: (cb: (progress: { progress: number; loaded: number; total: number }) => void) => void
}
