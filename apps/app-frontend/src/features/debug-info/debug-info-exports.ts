import { reactive, ref } from 'vue'

import { toError } from '@/helpers/errors'
import { cancelDebugInfoExport, exportDebugInfo } from '@/platform/app-lib/debug-info/commands'
import { pickSaveFile } from '@/platform/dialogs'

import type {
	DebugInfoExportInstance,
	DebugInfoExportsContext,
	DebugInfoExportTask,
} from './context'

function defaultFileName(instance: DebugInfoExportInstance | null) {
	const timestamp = new Date().toISOString().replace(/[:.]/g, '-')
	const name = instance?.name
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-+|-+$/g, '')
	return `modrinth-debug-${name ? `${name}-` : ''}${timestamp}.zip`
}

export function createDebugInfoExports(): DebugInfoExportsContext {
	const tasks = ref(new Map<string, DebugInfoExportTask>())
	const selectingDestination = ref(new Set<string | null>())

	function isExporting(instanceId: string | null = null) {
		return (
			selectingDestination.value.has(instanceId) ||
			[...tasks.value.values()].some(
				(task) => task.status === 'running' && (task.instance?.id ?? null) === instanceId,
			)
		)
	}

	async function startExport(title: string, instance: DebugInfoExportInstance | null = null) {
		const instanceId = instance?.id ?? null
		if (isExporting(instanceId)) return
		selectingDestination.value.add(instanceId)
		let path: string | null
		try {
			path = await pickSaveFile({
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
					await cancelDebugInfoExport(task.id)
				} catch (error) {
					task.canceling = false
					throw error
				}
			},
		})
		tasks.value.set(task.id, task)
		try {
			const saved = await exportDebugInfo(task.id, path, instanceId, (progress) => {
				if (task.status !== 'running') return
				task.started = true
				task.stage = progress.stage
				task.processedBytes = progress.processedBytes
				task.totalBytes = progress.totalBytes
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

	return { tasks, isExporting, exportDebugInfo: startExport }
}
