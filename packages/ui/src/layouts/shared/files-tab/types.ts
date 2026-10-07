import type { Component } from 'vue'

import type { ButtonMenuItemBase } from '#ui/components/base/buttons/types'
import type { FileInfo } from '#ui/layouts/shared/files-tab/providers/file-manager.ts'

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

/** A custom action offered in the menus of files and folders, next to the built-in ones. */
export interface FileEntryAction {
	id: string
	label: string
	icon?: Component
	tone?: ButtonMenuItemBase['tone']
	/** Whether the action applies to the entry; offered for every entry when omitted. */
	shown?: (entry: FileInfo) => boolean
	/** Whether the action changes files, which disables it while busy or for read-only entries. */
	writes?: boolean
	action: (entry: FileInfo) => void | Promise<void>
}

/** A custom detail listed after the built-in ones (size, items, dates) in an entry's details. */
export interface FileEntryDetail {
	id: string
	label: string
	/** Whether the detail applies to the entry; listed for every entry when omitted. */
	shown?: (entry: FileItem) => boolean
	/** The entry's value, or `null` when it has none. Promises are awaited once the details show. */
	value: (entry: FileItem) => string | null | Promise<string | null>
}

export interface ExtractDryRunResult {
	modpack_name: string | null
	conflicting_files: string[]
}

export type { UploadState } from '@modrinth/api-client'
