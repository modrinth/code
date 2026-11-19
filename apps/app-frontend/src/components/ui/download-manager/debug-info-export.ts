import { Channel, invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { computed, reactive, ref } from 'vue'

import { toError } from '@/helpers/errors'

export interface DebugInfoExportProgress {
	stage: 'preparing' | 'exporting' | 'finishing'
	processedBytes: number
	totalBytes: number | null
}

export interface DebugInfoExportInstance {
	id: string
	name: string
	iconPath: string | null
}

export interface DebugInfoExportTask extends DebugInfoExportProgress {
	id: string
	path: string
	instance: DebugInfoExportInstance | null
	status: 'running' | 'succeeded' | 'failed' | 'canceled'
	started: boolean
	canceling: boolean
	createdAt: string
	finishedAt?: string
	error?: string
	cancel: () => Promise<void>
}

export const debugInfoExports = ref(new Map<string, DebugInfoExportTask>())
const selectingDestination = ref(new Set<string | null>())

export function isExportingDebugInfo(instanceId: string | null = null) {
	return (
		selectingDestination.value.has(instanceId) ||
		[...debugInfoExports.value.values()].some(
			(task) => task.status === 'running' && (task.instance?.id ?? null) === instanceId,
		)
	)
}

export const exportingDebugInfo = computed(() => isExportingDebugInfo())

function defaultFileName(instance: DebugInfoExportInstance | null) {
	const timestamp = new Date().toISOString().replace(/[:.]/g, '-')
	const name = instance?.name
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-+|-+$/g, '')
	return `modrinth-debug-${name ? `${name}-` : ''}${timestamp}.zip`
}

export async function exportDebugInfo(
	title: string,
	instance: DebugInfoExportInstance | null = null,
) {
	const instanceId = instance?.id ?? null
	if (isExportingDebugInfo(instanceId)) return
	selectingDestination.value.add(instanceId)
	let path: string | null
	try {
		path = await save({
			title,
			defaultPath: defaultFileName(instance),
			filters: [{ name: 'ZIP', extensions: ['zip'] }],
		})
	} finally {
		selectingDestination.value.delete(instanceId)
	}
	if (!path) return

	const task = reactive<DebugInfoExportTask>({
		id: crypto.randomUUID(),
		path,
		instance,
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
			instanceId,
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
