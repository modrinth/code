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
				file.type === 'mrpack'
					? { name: 'Modrinth Modpack', extensions: ['mrpack'] }
					: file.type === 'server-backup' || file.type === 'server-world'
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

			const sink: DownloadSink = async (url, options) => {
				if (cancelled) throw new Error('Download cancelled')
				try {
					savedFilename = await saveExternal(
						saveId,
						{ url, headers: options.headers ?? {} },
						onProgress,
					)
				} catch (error) {
					const nativeError = error as ExternalFileError
					throw new ModrinthApiError(nativeError.message ?? String(error), {
						statusCode: nativeError.statusCode,
					})
				}
			}

			try {
				switch (file.type) {
					case 'server-file': {
						for (let attempt = 0; ; attempt++) {
							const auth = await client.archon.servers_v0.getFilesystemAuth(file.serverId)
							try {
								await client.kyros.files_v0.downloadFileTo(auth, file.path, sink)
								break
							} catch (error) {
								if (
									!(error instanceof ModrinthApiError) ||
									error.statusCode !== 401 ||
									attempt >= 2
								) {
									throw error
								}
							}
						}
						break
					}
					case 'mrpack':
						await client.sharedinstances.instances_v1.downloadMrpackTo(
							file.instanceId,
							file.version,
							sink,
						)
						break
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
						const server = await client.archon.servers_v0.get(file.serverId)
						if (!server.node) throw new Error('Server is not assigned to a node')
						await client.download(
							`/backups/${encodeURIComponent(file.backupId)}/download`,
							{
								api: getNodeBaseUrl(server.node.instance),
								version: 'modrinth/v0',
								params: { auth: server.node.token },
								skipAuth: true,
							},
							sink,
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
