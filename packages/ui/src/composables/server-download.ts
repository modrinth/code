import type { Archon } from '@modrinth/api-client'

import { injectModrinthClient } from '../providers/api-client'
import { injectNotificationManager } from '../providers/web-notifications'

export function hasAvailableWorldDownload(
	serverId: string,
	serverFullList: Archon.Servers.v1.ServerFull[] | null | undefined,
): boolean {
	const world = serverFullList?.find((server) => server.id === serverId)?.worlds[0]
	return !!world && world.download_method.method_type !== 'unavailable'
}

export function useServerWorldDownload() {
	const client = injectModrinthClient()
	const { addNotification } = injectNotificationManager()

	function showUnavailableNotification() {
		addNotification({
			title: 'Download unavailable',
			text: "This world's files cannot currently be downloaded.",
			type: 'error',
		})
	}

	function getWorldDownload(
		server: Archon.Servers.v0.Server,
		serverFullList: Archon.Servers.v1.ServerFull[] | null | undefined,
	): (() => Promise<void>) | null {
		const serverFull = serverFullList?.find((candidate) => candidate.id === server.server_id)
		const world = serverFull?.worlds[0]
		if (!serverFull || !world || world.download_method.method_type === 'unavailable') return null

		return async () => {
			try {
				const downloadMethod = await client.archon.servers_v1.selectWorldDownload(
					server.server_id,
					world.id,
				)

				switch (downloadMethod.method_type) {
					case 'direct_node_download': {
						if (serverFull.location.status !== 'assigned') {
							showUnavailableNotification()
							return
						}

						const nodeUrlHost = serverFull.location.location_metadata.url_host
						const { token } = await client.kyros.files_v1.authorizeFullWorldDownload(
							nodeUrlHost,
							world.id,
						)
						const downloadUrl = client.kyros.files_v1.getFullWorldDownloadUrl(
							nodeUrlHost,
							world.id,
							token,
						)
						window.open(downloadUrl, '_blank', 'noopener,noreferrer')
						break
					}
					case 'backup': {
						if (!server.node) {
							showUnavailableNotification()
							return
						}

						const nodeBaseUrl = server.node.instance.match(/^https?:\/\//)
							? server.node.instance
							: `https://${server.node.instance}`
						const downloadUrl = new URL(
							`/modrinth/v0/backups/${downloadMethod.backup_id}/download`,
							nodeBaseUrl,
						)
						downloadUrl.searchParams.set('auth', server.node.token)
						window.open(downloadUrl.toString(), '_blank', 'noopener,noreferrer')
						break
					}
					case 'unavailable':
						showUnavailableNotification()
						break
				}
			} catch {
				addNotification({
					title: 'Download failed',
					text: "An error occurred while trying to download the world's files.",
					type: 'error',
				})
			}
		}
	}

	return { getWorldDownload }
}
