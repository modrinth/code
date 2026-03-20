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

export type FileContextMenuOption =
	| {
			id: string
			action?: () => void
			disabled?: boolean
			tooltip?: string
			color?: 'standard' | 'brand' | 'red' | 'orange' | 'green' | 'blue' | 'purple' | 'medal-promo'
			shown?: boolean
	  }
	| { divider: true; shown?: boolean }

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

export interface UploadState {
	isUploading: boolean
	currentFileName: string | null
	currentFileProgress: number
	uploadedBytes: number
	totalBytes: number
	completedFiles: number
	totalFiles: number
}
