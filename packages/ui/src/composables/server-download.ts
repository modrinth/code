import type { Archon } from '@modrinth/api-client'

import { injectModrinthClient } from '../providers/api-client'
import { injectNotificationManager } from '../providers/web-notifications'
import { defineMessages, useVIntl } from './i18n'

const messages = defineMessages({
	downloadUnavailableTitle: {
		id: 'servers.download.unavailable.title',
		defaultMessage: 'Download unavailable',
	},
	downloadUnavailableDescription: {
		id: 'servers.download.unavailable.description',
		defaultMessage: "This world's files cannot currently be downloaded.",
	},
	downloadFailedTitle: {
		id: 'servers.download.failed.title',
		defaultMessage: 'Download failed',
	},
	downloadFailedDescription: {
		id: 'servers.download.failed.description',
		defaultMessage: "An error occurred while trying to download the world's files.",
	},
})

export function hasAvailableWorldDownload(
	serverId: string,
	serverFullList: Archon.Servers.v1.ServerFull[] | null | undefined,
): boolean {
	const world = serverFullList?.find((server) => server.id === serverId)?.worlds[0]
	return world?.download_method.method_type === 'direct_node_download'
}

export function useServerWorldDownload() {
	const client = injectModrinthClient()
	const { addNotification } = injectNotificationManager()
	const { formatMessage } = useVIntl()

	function showUnavailableNotification() {
		addNotification({
			title: formatMessage(messages.downloadUnavailableTitle),
			text: formatMessage(messages.downloadUnavailableDescription),
			type: 'error',
		})
	}

	function showDownloadFailedNotification() {
		addNotification({
			title: formatMessage(messages.downloadFailedTitle),
			text: formatMessage(messages.downloadFailedDescription),
			type: 'error',
		})
	}

	async function downloadWorldFiles(nodeUrlHost: string, worldId: string) {
		try {
			const { token } = await client.kyros.files_v1.authorizeFullWorldDownload(nodeUrlHost, worldId)
			const downloadUrl = client.kyros.files_v1.getFullWorldDownloadUrl(nodeUrlHost, worldId, token)
			window.location.assign(downloadUrl)
		} catch {
			showDownloadFailedNotification()
		}
	}

	function getWorldDownload(
		serverId: string,
		serverFullList: Archon.Servers.v1.ServerFull[] | null | undefined,
	): (() => Promise<void>) | null {
		const serverFull = serverFullList?.find((candidate) => candidate.id === serverId)
		const world = serverFull?.worlds[0]
		if (!serverFull || !world || world.download_method.method_type !== 'direct_node_download')
			return null

		return async () => {
			try {
				const downloadMethod = await client.archon.servers_v1.selectWorldDownload(
					serverId,
					world.id,
				)

				if (
					downloadMethod.method_type !== 'direct_node_download' ||
					serverFull.location.status !== 'assigned'
				) {
					showUnavailableNotification()
					return
				}

				const nodeUrlHost = serverFull.location.location_metadata.url_host
				await downloadWorldFiles(nodeUrlHost, world.id)
			} catch {
				showDownloadFailedNotification()
			}
		}
	}

	return { getWorldDownload, downloadWorldFiles }
}
