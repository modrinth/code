import type {FileInfo} from "#ui/layouts/shared/files-tab/providers/file-manager.ts";

export interface FileItem {
	name: string
	type: 'file' | 'directory' | 'symlink'
	path: string
	modified: number
	created: number
	size?: number | null
	count?: number | null
	target?: string
	readOnly?: boolean
}

export interface EditingFile {
	name: string
	path: string
}

export type FileSortField = 'name' | 'size' | 'items' | 'created' | 'modified'

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
	cancellable?: boolean
	error?: string
}

export interface UndoableOperation {
	type: 'move' | 'rename'
	prevFile: FileInfo
	newFile: FileInfo
}

export interface MoveOperation extends UndoableOperation {
	type: 'move'
}

export interface RenameOperation extends UndoableOperation {
	type: 'rename'
}

export type Operation = MoveOperation | RenameOperation

export interface ExtractDryRunResult {
	modpack_name: string | null
	conflicting_files: string[]
}

export type { UploadState } from '@modrinth/api-client'
