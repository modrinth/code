import { Channel, invoke } from '@tauri-apps/api/core'

export interface DebugInfoExportProgress {
	stage: 'preparing' | 'exporting' | 'finishing'
	processedBytes: number
	totalBytes: number | null
}

export function exportDebugInfo(
	id: string,
	path: string,
	instanceId: string | null,
	onProgress: (progress: DebugInfoExportProgress) => void,
): Promise<boolean> {
	return invoke('plugin:utils|export_debug_info', {
		id,
		path,
		instanceId,
		onProgress: new Channel<DebugInfoExportProgress>(onProgress),
	})
}

export function cancelDebugInfoExport(id: string): Promise<void> {
	return invoke('plugin:utils|cancel_debug_info_export', { id })
}
