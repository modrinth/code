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

			const saveUrl = async (url: string, headers: Record<string, string> = {}) => {
				if (cancelled) throw new Error('Download cancelled')
				try {
					savedFilename = await invoke<string>('plugin:files|files_save_external', {
						saveId,
						request: { url, headers },
						onProgress,
					})
				} catch (error) {
					const nativeError = error as NativeSaveError
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
				await invoke('plugin:files|files_release_external', { saveId }).catch(() => undefined)
			}
		},
	})
}
