import {
	defineMessages,
	injectPopupNotificationManager,
	type PopupNotification,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { watch } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings'
import { traceStartupStep } from '@/helpers/startup-debug'
import { instanceListQueryOptions } from '@/pages/instance/query-options'

import {
	markSyncInstancesUpdateNotificationShown,
	shouldShowSyncInstancesUpdateNotification,
} from './show-notification'

const messages = defineMessages({
	title: {
		id: 'app.sync-instances-update.notification.title',
		defaultMessage: 'Sync your instances',
	},
	description: {
		id: 'app.sync-instances-update.notification.description',
		defaultMessage:
			'Keep game settings, servers, resource packs, and more in sync across your instances.',
	},
	view: {
		id: 'app.sync-instances-update.notification.view-update',
		defaultMessage: 'View update',
	},
	dismiss: {
		id: 'app.sync-instances-update.notification.dismiss',
		defaultMessage: 'Dismiss',
	},
})

export function useNewUpdateNotification(showModal: () => void) {
	const appSettings = useAppSettings()
	const queryClient = useQueryClient()
	const popupNotificationManager = injectPopupNotificationManager()
	const { formatMessage } = useVIntl()
	let notificationId: PopupNotification['id'] | null = null

	function showNotification() {
		if (
			popupNotificationManager
				.getNotifications()
				.some((notification) => notification.id === notificationId)
		) {
			return
		}

		if (!shouldShowSyncInstancesUpdateNotification()) return

		const notification = popupNotificationManager.addPopupNotification({
			contentType: 'standard',
			title: formatMessage(messages.title),
			text: formatMessage(messages.description),
			type: 'info',
			hideIcon: true,
			autoCloseMs: null,
			buttons: [
				{
					label: formatMessage(messages.dismiss),
					color: 'standard',
					action: () => popupNotificationManager.removeNotification(notification.id),
				},
				{
					label: formatMessage(messages.view),
					color: 'brand',
					action: showModal,
				},
			],
		})
		notificationId = notification.id
	}

	async function notifyForVersion(version: string, pendingUpdateToastForVersion: string | null) {
		const isSyncUpdateVersion = version.startsWith('0.20.')
		if (isSyncUpdateVersion && pendingUpdateToastForVersion !== version) {
			markSyncInstancesUpdateNotificationShown()
		}
		if (
			appSettings.getFeatureFlag('show_sync_instances_update_modal') ||
			(isSyncUpdateVersion &&
				pendingUpdateToastForVersion === version &&
				(
					await traceStartupStep('Load instances for update notification', () =>
						queryClient.fetchQuery(instanceListQueryOptions()),
					)
				).length > 0)
		) {
			showNotification()
		}
	}

	watch(
		() => appSettings.getFeatureFlag('show_sync_instances_update_modal'),
		(enabled) => {
			if (enabled) showNotification()
		},
	)

	return { notifyForVersion }
}
