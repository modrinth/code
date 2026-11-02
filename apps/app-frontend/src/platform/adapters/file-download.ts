import {
	type AbstractModrinthClient,
	type DownloadSink,
	getNodeBaseUrl,
	ModrinthApiError,
} from '@modrinth/api-client'
import type { FileDownloadProvider } from '@modrinth/ui'
import { Channel } from '@tauri-apps/api/core'

import {
	type ExternalFileError,
	type ExternalFileProgress,
	releaseExternal,
	saveExternal,
	selectExternal,
} from '@/platform/app-lib/files/commands'

export interface FileDownloadTask {
	update: (progress: ExternalFileProgress) => void
	succeed: (filename: string) => void
	fail: (error: string) => void
	canceled: () => void
}

/** Registers a running download so the app can display its progress and cancel it. */
export type TrackFileDownload = (
	saveId: string,
	filename: string,
	onCancel: () => Promise<void>,
	server: { serverId?: string; serverName?: string },
) => FileDownloadTask

export function createFileDownload(
	client: AbstractModrinthClient,
	trackDownload: TrackFileDownload,
): FileDownloadProvider {
	return {
		async download(file) {
			const saveId = await selectExternal(
				file.filename,
				file.type === 'server-backup' || file.type === 'server-world'
					? { name: 'ZIP archive', extensions: ['zip'] }
					: null,
			).catch((error: ExternalFileError) => {
				throw new Error(error.message ?? String(error))
			})
			if (!saveId) return false

			let cancelled = false
			let finished = false
			let savedFilename = file.filename
			const task = trackDownload(
				saveId,
				file.filename,
				async () => {
					cancelled = true
					await releaseExternal(saveId)
				},
				{ serverId: file.serverId, serverName: file.serverName },
			)
			const onProgress = new Channel<ExternalFileProgress>((progress) => {
				if (cancelled || finished) return
				task.update(progress)
			})

			const saveUrl = async (url: string, headers: Record<string, string> = {}) => {
				if (cancelled) throw new Error('Download cancelled')
				try {
					savedFilename = await saveExternal(saveId, { url, headers }, onProgress)
				} catch (error) {
					const nativeError = error as ExternalFileError
					throw new ModrinthApiError(nativeError.message ?? String(error), {
						statusCode: nativeError.statusCode,
					})
				}
			}
			const sink: DownloadSink = (url, options) => saveUrl(url, options.headers)

			try {
				switch (file.type) {
					case 'server-file': {
						const { token } = await client.kyros.files_v1.authorizeFileDownload(
							file.nodeUrlHost,
							file.worldId,
							file.path,
						)
						await saveUrl(
							client.kyros.files_v1.getFileDownloadUrl(file.nodeUrlHost, file.worldId, token),
						)
						break
					}
					case 'server-world': {
						const { token } = await client.kyros.files_v1.authorizeFullWorldDownload(
							file.nodeUrlHost,
							file.worldId,
						)
						await client.download(
							`/worlds/${encodeURIComponent(file.worldId)}/files/download-full-zip`,
							{
								api: getNodeBaseUrl(file.nodeUrlHost),
								version: 'v1',
								params: { token },
								skipAuth: true,
							},
							sink,
						)
						break
					}
					case 'server-backup': {
						const { token } = await client.kyros.backups_v1.authorizeDownload(
							file.nodeUrlHost,
							file.worldId,
							file.backupId,
						)
						await saveUrl(
							client.kyros.backups_v1.getDownloadUrl(
								file.nodeUrlHost,
								file.worldId,
								file.backupId,
								token,
							),
						)
						break
					}
				}
				task.succeed(savedFilename)
				return true
			} catch (error) {
				if (cancelled) {
					task.canceled()
					return false
				}
				task.fail(error instanceof Error ? error.message : String(error))
				throw error
			} finally {
				finished = true
				await releaseExternal(saveId).catch(() => undefined)
			}
		},
	}
}
