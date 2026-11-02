import {
	defineMessages,
	injectAuth,
	injectModrinthClient,
	injectPopupNotificationManager,
	type PopupNotification,
	resolveServerSharingUpdateTarget,
	type ServerSharingUpdateTarget,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { onScopeDispose, ref, watch } from 'vue'

import { debugStartup, traceStartupStep } from '@/helpers/startup-debug'

import {
	markServerSharingUpdateNotificationShown,
	shouldShowServerSharingUpdateNotification,
} from './show-notification'

const messages = defineMessages({
	title: {
		id: 'app.server-sharing-update.title',
		defaultMessage: 'Share servers with friends',
	},
	description: {
		id: 'app.server-sharing-update.notification.description',
		defaultMessage: 'Invite friends to play with an instance managed by your server.',
	},
	view: {
		id: 'app.server-sharing-update.notification.view-update',
		defaultMessage: 'View update',
	},
	dismiss: {
		id: 'app.server-sharing-update.notification.dismiss',
		defaultMessage: 'Dismiss',
	},
})

export function useNewUpdateNotification(showModal: () => void) {
	const auth = injectAuth()
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const popupNotificationManager = injectPopupNotificationManager()
	const { formatMessage } = useVIntl()
	const inviteTarget = ref<ServerSharingUpdateTarget>({ path: '/hosting/manage', playPage: false })
	let notificationId: PopupNotification['id'] | null = null
	let pendingUpdate = false
	let disposed = false

	function removeNotification() {
		if (notificationId === null) return
		popupNotificationManager.removeNotification(notificationId)
		notificationId = null
	}

	function canNotify() {
		return (
			!disposed &&
			notificationId === null &&
			pendingUpdate &&
			shouldShowServerSharingUpdateNotification()
		)
	}

	async function showNotification() {
		const userId = auth.user.value?.id
		const session = auth.session_token.value
		if (!userId || !session || !canNotify()) return

		try {
			const target = await traceStartupStep('Load servers for update notification', () =>
				resolveServerSharingUpdateTarget(client, queryClient, userId),
			)
			if (
				auth.user.value?.id !== userId ||
				auth.session_token.value !== session ||
				!canNotify() ||
				!target
			) {
				return
			}

			inviteTarget.value = target
			if (!markServerSharingUpdateNotificationShown()) return

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
						action: removeNotification,
					},
					{
						label: formatMessage(messages.view),
						color: 'brand',
						action: () => {
							removeNotification()
							showModal()
						},
					},
				],
			})
			notificationId = notification.id
		} catch {
			debugStartup('Could not load servers for update notification')
		}
	}

	async function notifyForVersion(version: string, pendingUpdateToastForVersion: string | null) {
		const isServerSharingUpdateVersion = version.startsWith('0.23.')
		pendingUpdate = isServerSharingUpdateVersion && pendingUpdateToastForVersion === version
		if (isServerSharingUpdateVersion && !pendingUpdate) {
			markServerSharingUpdateNotificationShown()
		}
		await showNotification()
	}

	watch(
		[() => auth.session_token.value, () => auth.user.value?.id],
		() => {
			removeNotification()
			inviteTarget.value = { path: '/hosting/manage', playPage: false }
			void showNotification()
		},
	)

	onScopeDispose(() => {
		disposed = true
		removeNotification()
	})

	return { notifyForVersion, inviteTarget }
}
