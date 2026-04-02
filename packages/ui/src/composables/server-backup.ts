import type { Archon } from '@modrinth/api-client'

import { injectModrinthClient } from '../providers/api-client'
import { injectNotificationManager } from '../providers/web-notifications'

export function useServerBackupDownload() {
	const client = injectModrinthClient()
	const { addNotification } = injectNotificationManager()

	function getLatestBackupDownload(
		serverId: string,
		serverFullList: Archon.Servers.v1.ServerFull[] | null | undefined,
	): (() => Promise<void>) | null {
		const serverFull = serverFullList?.find((s) => s.id === serverId)
		if (!serverFull) return null

		const activeWorld = serverFull.worlds.find((w) => w.is_active) ?? serverFull.worlds[0]
		if (!activeWorld?.backups?.length) return null

		const latestBackup = activeWorld.backups
			.filter((b) => b.status === 'done')
			.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())[0]
		if (!latestBackup) return null

		return async () => {
			try {
				const server = await client.archon.servers_v0.get(serverId)
				const kyrosUrl = server.node?.instance
				const jwt = server.node?.token
				if (!kyrosUrl || !jwt) {
					addNotification({
						title: 'Download unavailable',
						text: 'Server connection info is not available. Please contact support.',
						type: 'error',
					})
					return
				}

				window.open(
					`https://${kyrosUrl}/modrinth/v0/backups/${latestBackup.id}/download?auth=${jwt}`,
					'_blank',
				)
			} catch {
				addNotification({
					title: 'Download failed',
					text: 'An error occurred while trying to download the backup.',
					type: 'error',
				})
			}
		}
	}

	return { getLatestBackupDownload }
}
