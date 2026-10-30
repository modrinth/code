import { Channel, invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { computed, reactive, ref } from 'vue'

import { toError } from '@/helpers/errors'

export interface DebugInfoExportProgress {
	stage: 'preparing' | 'exporting' | 'finishing'
	processedBytes: number
	totalBytes: number | null
}

export interface DebugInfoExportTask extends DebugInfoExportProgress {
	id: string
	filename: string
	status: 'running' | 'succeeded' | 'failed' | 'canceled'
	started: boolean
	canceling: boolean
	createdAt: string
	finishedAt?: string
	error?: string
	cancel: () => Promise<void>
}

export const debugInfoExports = ref(new Map<string, DebugInfoExportTask>())
const selectingDestination = ref(false)
export const exportingDebugInfo = computed(
	() =>
		selectingDestination.value ||
		[...debugInfoExports.value.values()].some((task) => task.status === 'running'),
)

export async function exportDebugInfo(title: string) {
	if (exportingDebugInfo.value) return
	selectingDestination.value = true
	let path: string | null
	try {
		path = await save({
			title,
			defaultPath: `modrinth-debug-${new Date().toISOString().replace(/[:.]/g, '-')}.zip`,
			filters: [{ name: 'ZIP', extensions: ['zip'] }],
		})
	} finally {
		selectingDestination.value = false
	}
	if (!path) return

	const task = reactive<DebugInfoExportTask>({
		id: crypto.randomUUID(),
		filename: path.split(/[/\\]/).pop() ?? path,
		status: 'running',
		stage: 'preparing',
		processedBytes: 0,
		totalBytes: null,
		started: false,
		canceling: false,
		createdAt: new Date().toISOString(),
		async cancel() {
			if (
				task.status !== 'running' ||
				!task.started ||
				task.canceling ||
				task.stage === 'finishing'
			)
				return
			task.canceling = true
			try {
				await invoke('plugin:utils|cancel_debug_info_export', { id: task.id })
			} catch (error) {
				task.canceling = false
				throw error
			}
		},
	})
	debugInfoExports.value.set(task.id, task)
	const onProgress = new Channel<DebugInfoExportProgress>((progress) => {
		if (task.status !== 'running') return
		task.started = true
		task.stage = progress.stage
		task.processedBytes = progress.processedBytes
		task.totalBytes = progress.totalBytes
	})
	try {
		const saved = await invoke<boolean>('plugin:utils|export_debug_info', {
			id: task.id,
			path,
			onProgress,
		})
		task.status = saved ? 'succeeded' : 'canceled'
	} catch (error) {
		task.status = 'failed'
		task.error = toError(error).message
		throw error
	} finally {
		task.canceling = false
		task.finishedAt = new Date().toISOString()
	}
}
