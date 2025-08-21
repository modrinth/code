import type { JWTAuth } from './api'
import type { FilesystemOp, FSQueuedOp } from './websocket'

export interface DirectoryItem {
	name: string
	type: 'directory' | 'file'
	count?: number
	modified: number
	created: number
	path: string
}

export interface FileUploadQuery {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	promise: Promise<any>
	onProgress: (
		callback: (progress: { loaded: number; total: number; progress: number }) => void,
	) => void
	cancel: () => void
}

export interface DirectoryResponse {
	items: DirectoryItem[]
	total: number
	current?: number
}

export interface FSModule {
	auth: JWTAuth
	ops: FilesystemOp[]
	queuedOps: FSQueuedOp[]
	opsQueuedForModification: string[]
}
