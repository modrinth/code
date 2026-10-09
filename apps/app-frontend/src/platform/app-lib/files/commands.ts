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
