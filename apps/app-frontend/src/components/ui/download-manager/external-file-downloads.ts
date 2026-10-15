import { reactive, ref } from 'vue'

export interface ExternalFileDownloadProgress {
	stage: 'waiting' | 'downloading' | 'saving'
	downloadedBytes: number
	totalBytes: number | null
}

export interface ExternalFileDownloadTask {
	id: string
	filename: string
	serverId?: string
	serverName?: string
	status: 'running' | 'succeeded' | 'failed' | 'canceled'
	stage: 'preparing' | ExternalFileDownloadProgress['stage']
	downloadedBytes: number
	totalBytes: number | null
	rate: number
	lastRead: number
	createdAt: string
	finishedAt?: string
	error?: string
	canceling: boolean
	cancel: () => Promise<void>
}

export const externalFileDownloads = ref(new Map<string, ExternalFileDownloadTask>())

export function trackExternalFileDownload(
	id: string,
	filename: string,
	onCancel: () => Promise<void>,
	server: { serverId?: string; serverName?: string } = {},
) {
	const task = reactive<ExternalFileDownloadTask>({
		id,
		filename,
		...server,
		status: 'running',
		stage: 'preparing',
		downloadedBytes: 0,
		totalBytes: null,
		rate: 0,
		lastRead: performance.now(),
		createdAt: new Date().toISOString(),
		canceling: false,
		async cancel() {
			if (task.status !== 'running' || task.canceling) return
			task.canceling = true
			try {
				await onCancel()
				if (task.status === 'running') finish('canceled')
			} finally {
				task.canceling = false
			}
		},
	})
	externalFileDownloads.value.set(id, task)
	let previousBytes = 0
	let previousTime = performance.now()

	function finish(status: ExternalFileDownloadTask['status']) {
		task.status = status
		task.finishedAt = new Date().toISOString()
		task.rate = 0
	}

	return {
		update(progress: ExternalFileDownloadProgress) {
			if (task.status !== 'running' || task.canceling) return
			const now = performance.now()
			if (progress.stage !== 'downloading' || progress.downloadedBytes < previousBytes) {
				task.rate = 0
			} else if (progress.downloadedBytes > previousBytes) {
				const rate =
					((progress.downloadedBytes - previousBytes) * 1000) / Math.max(1, now - previousTime)
				task.rate = task.rate ? task.rate * 0.6 + rate * 0.4 : rate
				task.lastRead = now
			}
			task.stage = progress.stage
			task.downloadedBytes = progress.downloadedBytes
			task.totalBytes = progress.totalBytes
			previousBytes = progress.downloadedBytes
			previousTime = now
		},
		succeed(filename: string) {
			task.filename = filename
			finish('succeeded')
		},
		fail(error: string) {
			task.error = error
			finish('failed')
		},
		canceled() {
			finish('canceled')
		},
	}
}

export function getExternalFileDownloadRate(task: ExternalFileDownloadTask, now: number) {
	return task.status === 'running' &&
		task.stage === 'downloading' &&
		!task.canceling &&
		now - task.lastRead < 5000
		? task.rate
		: 0
}
