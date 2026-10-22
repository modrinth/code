import type { AbstractModrinthClient } from '@modrinth/api-client'
import { type FileDownload, provideFileDownload } from '@modrinth/ui'
import { Channel, invoke } from '@tauri-apps/api/core'

import {
	type ExternalFileDownloadProgress,
	trackExternalFileDownload,
} from '@/components/ui/download-manager/external-file-downloads'

interface NativeSaveError {
	message?: string
}

async function authorizeDownloadUrl(client: AbstractModrinthClient, file: FileDownload) {
	if (file.type === 'server-backup') {
		const { token } = await client.kyros.backups_v1.authorizeDownload(
			file.nodeUrlHost,
			file.worldId,
			file.backupId,
		)
		return client.kyros.backups_v1.getDownloadUrl(
			file.nodeUrlHost,
			file.worldId,
			file.backupId,
			token,
		)
	}

	const { token } = await client.kyros.files_v1.authorizeFileDownload(
		file.nodeUrlHost,
		file.worldId,
		file.path,
	)
	return client.kyros.files_v1.getFileDownloadUrl(file.nodeUrlHost, file.worldId, token)
}

export function setupFileDownloadProvider(client: AbstractModrinthClient) {
	provideFileDownload({
		async download(file) {
			const saveId = await invoke<string | null>('plugin:files|files_select_external', {
				filename: file.filename,
				filter: null,
			}).catch((error: NativeSaveError) => {
				throw new Error(error.message ?? String(error))
			})
			if (!saveId) return false

			let cancelled = false
			let finished = false
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

			try {
				const url = await authorizeDownloadUrl(client, file)
				if (cancelled) return false
				const savedFilename = await invoke<string>('plugin:files|files_save_external', {
					saveId,
					request: { url },
					onProgress,
				})
				task.succeed(savedFilename)
				return true
			} catch (error) {
				if (cancelled) {
					task.canceled()
					return false
				}
				const message = (error as NativeSaveError).message ?? String(error)
				task.fail(message)
				throw new Error(message)
			} finally {
				finished = true
				await invoke('plugin:files|files_release_external', { saveId }).catch(() => undefined)
			}
		},
	})
}
