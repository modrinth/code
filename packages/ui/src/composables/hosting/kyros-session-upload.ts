import type {
	AbstractModrinthClient,
	Kyros,
	UploadProgress,
	UploadState,
} from '@modrinth/api-client'
import type { Ref } from 'vue'

import type { CancelUploadHandler } from '#ui/providers/server-context'

export type UploadSessionUploadFile = {
	file: File
	filename: string
}

export type UploadSessionUploadResult = 'completed' | 'cancelled'

export function useUploadSessionUpload(options: {
	client: AbstractModrinthClient
	scope: Kyros.UploadSessions.v1.Scope
	worldId: Ref<string | null>
	uploadState: Ref<UploadState>
	cancelUpload: Ref<CancelUploadHandler | null>
}) {
	let activeUploadCancel: CancelUploadHandler | null = null

	function getUploadByteCount(files: File[]) {
		return files.reduce((sum, file) => sum + file.size, 0)
	}

	function resetUploadState() {
		options.uploadState.value = {
			isUploading: false,
			currentFileName: null,
			currentFileProgress: 0,
			uploadedBytes: 0,
			totalBytes: 0,
			completedFiles: 0,
			totalFiles: 0,
		}
	}

	function startUploadState(files: File[]) {
		options.uploadState.value = {
			isUploading: true,
			currentFileName: files[0]?.name ?? null,
			currentFileProgress: 0,
			uploadedBytes: 0,
			totalBytes: getUploadByteCount(files),
			completedFiles: 0,
			totalFiles: files.length,
		}
	}

	function setUploadProgressFromBytes(files: File[], uploadedBytes: number) {
		const totalBytes = getUploadByteCount(files)
		const boundedUploadedBytes = Math.max(0, Math.min(totalBytes, uploadedBytes))
		let previousBytes = 0

		for (let i = 0; i < files.length; i++) {
			const file = files[i]
			const nextBytes = previousBytes + file.size
			if (boundedUploadedBytes >= nextBytes) {
				previousBytes = nextBytes
				continue
			}

			options.uploadState.value.currentFileName = file.name
			options.uploadState.value.currentFileProgress =
				file.size === 0 ? 1 : (boundedUploadedBytes - previousBytes) / file.size
			options.uploadState.value.uploadedBytes = boundedUploadedBytes
			options.uploadState.value.totalBytes = totalBytes
			options.uploadState.value.completedFiles = i
			return
		}

		options.uploadState.value.currentFileName =
			files.length > 0 ? files[files.length - 1].name : null
		options.uploadState.value.currentFileProgress = files.length > 0 ? 1 : 0
		options.uploadState.value.uploadedBytes = totalBytes
		options.uploadState.value.totalBytes = totalBytes
		options.uploadState.value.completedFiles = files.length
	}

	function setUploadProgressFromXhr(files: File[], progress: UploadProgress) {
		const totalBytes = getUploadByteCount(files)
		const uploadedBytes =
			progress.total > 0
				? Math.round(totalBytes * progress.progress)
				: Math.min(progress.loaded, totalBytes)
		setUploadProgressFromBytes(files, uploadedBytes)
	}

	async function cancelUploadSession(worldId: string, uploadId: string) {
		try {
			await options.client.kyros.upload_sessions_v1.cancel(options.scope, worldId, uploadId)
		} catch {
		}
	}

	async function cancelUpload() {
		await activeUploadCancel?.()
	}

	async function uploadFiles(
		files: UploadSessionUploadFile[],
	): Promise<UploadSessionUploadResult> {
		if (files.length === 0) return 'cancelled'
		if (options.uploadState.value.isUploading) return 'cancelled'
		const worldId = options.worldId.value
		if (!worldId) return 'cancelled'

		const sourceFiles = files.map(({ file }) => file)
		startUploadState(sourceFiles)

		let cancelled = false
		let finalized = false
		let uploadId: string | null = null
		let uploadHandle: { cancel: () => void } | null = null
		let cancelRequest: Promise<void> | null = null
		let cancelCompletion: Promise<void> | null = null
		let resolveCancelCompletion: (() => void) | null = null
		const waitForCancelCompletion = () => {
			cancelCompletion ??= new Promise<void>((resolve) => {
				resolveCancelCompletion = resolve
			})
			return cancelCompletion
		}
		const completeCancel = () => {
			resolveCancelCompletion?.()
			resolveCancelCompletion = null
			cancelCompletion = null
		}
		const cancelSessionOnce = async () => {
			if (!uploadId) return
			cancelRequest ??= cancelUploadSession(worldId, uploadId)
			await cancelRequest
		}
		const finishCancellation = async () => {
			await cancelSessionOnce()
			completeCancel()
		}
		const cancelCurrentUpload = async () => {
			cancelled = true
			uploadHandle?.cancel()
			if (!uploadId) {
				await waitForCancelCompletion()
				return
			}
			await finishCancellation()
		}

		activeUploadCancel = cancelCurrentUpload
		options.cancelUpload.value = cancelCurrentUpload

		try {
			const session = await options.client.kyros.upload_sessions_v1.create(options.scope, worldId)
			uploadId = session.upload_id

			if (cancelled) {
				await finishCancellation()
				return 'cancelled'
			}

			uploadHandle = options.client.kyros.upload_sessions_v1.uploadFiles(
				options.scope,
				worldId,
				uploadId,
				files,
				{
					onProgress: (progress) => setUploadProgressFromXhr(sourceFiles, progress),
				},
			)

			await uploadHandle.promise
			if (cancelled) {
				await finishCancellation()
				return 'cancelled'
			}

			setUploadProgressFromBytes(sourceFiles, getUploadByteCount(sourceFiles))
			await options.client.kyros.upload_sessions_v1.finalize(options.scope, worldId, uploadId)
			finalized = true
			return 'completed'
		} catch (error) {
			if (uploadId && !finalized) {
				await finishCancellation()
			} else if (cancelled) {
				completeCancel()
			}
			if (cancelled || (error instanceof Error && error.message === 'Upload cancelled')) {
				return 'cancelled'
			}
			throw error
		} finally {
			if (activeUploadCancel === cancelCurrentUpload) {
				activeUploadCancel = null
			}
			if (options.cancelUpload.value === cancelCurrentUpload) {
				options.cancelUpload.value = null
			}
			if (cancelled) {
				completeCancel()
			}
			resetUploadState()
		}
	}

	return {
		cancelUpload,
		uploadFiles,
	}
}
