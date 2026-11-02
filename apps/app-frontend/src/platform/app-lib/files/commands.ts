import { type Channel, invoke } from '@tauri-apps/api/core'

export interface ExternalFileFilter {
	name: string
	extensions: string[]
}

export interface ExternalFileRequest {
	url: string
	headers: Record<string, string>
}

export interface ExternalFileProgress {
	stage: 'waiting' | 'downloading' | 'saving'
	downloadedBytes: number
	totalBytes: number | null
}

export interface ExternalFileError {
	message?: string
	statusCode?: number
}

export function selectExternal(
	filename: string,
	filter: ExternalFileFilter | null,
): Promise<string | null> {
	return invoke('plugin:files|files_select_external', { filename, filter })
}

export function saveExternal(
	saveId: string,
	request: ExternalFileRequest,
	onProgress: Channel<ExternalFileProgress>,
): Promise<string> {
	return invoke('plugin:files|files_save_external', { saveId, request, onProgress })
}

export function releaseExternal(saveId: string): Promise<void> {
	return invoke('plugin:files|files_release_external', { saveId })
}

export interface LocalFileInfo {
	name: string
	size: number
}

export interface LocalUploadRequest {
	url: string
	headers: Record<string, string>
	path: string
	fileName?: string
	multipartField?: string
}

export interface LocalUploadProgress {
	loaded: number
	total: number
}

export interface LocalUploadResponse {
	status: number
	body: string
}

export function getLocalFileInfo(path: string): Promise<LocalFileInfo> {
	return invoke('plugin:files|file_local_info', { path })
}

export function uploadLocal(
	uploadId: string,
	request: LocalUploadRequest,
	onProgress: Channel<LocalUploadProgress>,
): Promise<LocalUploadResponse> {
	return invoke('plugin:files|file_upload_local', { uploadId, request, onProgress })
}

export function cancelLocalUpload(uploadId: string): Promise<void> {
	return invoke('plugin:files|file_cancel_local_upload', { uploadId })
}
