import {
	type AbstractModrinthClient,
	type DownloadSink,
	getNodeBaseUrl,
	ModrinthApiError,
} from '@modrinth/api-client'
import { provideFileDownload } from '@modrinth/ui'
import { Channel, invoke } from '@tauri-apps/api/core'

import {
	type ExternalFileDownloadProgress,
	trackExternalFileDownload,
} from '@/components/ui/download-manager/external-file-downloads'

interface NativeSaveError {
	message?: string
	statusCode?: number
}

export function setupFileDownloadProvider(client: AbstractModrinthClient) {
	provideFileDownload({
		async download(file) {
			const saveId = await invoke<string | null>('plugin:files|files_select_external', {
				filename: file.filename,
				filter:
					file.type === 'mrpack'
						? { name: 'Modrinth Modpack', extensions: ['mrpack'] }
						: file.type === 'server-backup' || file.type === 'server-world'
							? { name: 'ZIP archive', extensions: ['zip'] }
							: null,
			}).catch((error: NativeSaveError) => {
				throw new Error(error.message ?? String(error))
			})
			if (!saveId) return false

			let cancelled = false
			let finished = false
			let savedFilename = file.filename
			const task = trackExternalFileDownload(
				saveId,
				file.filename,
				async () => {
					cancelled = true
					await invoke('plugin:files|files_release_external', { saveId })
				},
				{ serverId: file.serverId, serverName: file.serverName },
			)
			const onProgress = new Channel<ExternalFileDownloadProgress>((progress) => {
				if (cancelled || finished) return
				task.update(progress)
			})

			const sink: DownloadSink = async (url, options) => {
				if (cancelled) throw new Error('Download cancelled')
				try {
					savedFilename = await invoke<string>('plugin:files|files_save_external', {
						saveId,
						request: { url, headers: options.headers ?? {} },
						onProgress,
					})
				} catch (error) {
					const nativeError = error as NativeSaveError
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
				await invoke('plugin:files|files_release_external', { saveId }).catch(() => undefined)
			}
		},
	})
}
