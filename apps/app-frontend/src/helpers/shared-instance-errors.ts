import { defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'

import { getErrorMessage, type SharedInstanceUnavailableReason } from '@/helpers/install'

export const sharedInstanceErrorMessages = defineMessages({
	unavailableTitle: {
		id: 'instance.shared-instance.unavailable.title',
		defaultMessage: 'Shared instance no longer available',
	},
	lockedTitle: {
		id: 'instance.shared-instance.unavailable.locked-title',
		defaultMessage: 'Instance locked',
	},
	unavailableText: {
		id: 'instance.shared-instance.unavailable.text',
		defaultMessage:
			"Your local instance is still available, but it is no longer linked and won't receive updates.",
	},
	deletedText: {
		id: 'instance.shared-instance.unavailable.deleted-text',
		defaultMessage:
			'The primary instance was deleted. This instance is still available, but it is no longer linked and will no longer receive updates.',
	},
	accessRevokedText: {
		id: 'instance.shared-instance.unavailable.access-revoked-text',
		defaultMessage:
			'Your access to the shared instance was revoked. This instance is still available, but it is no longer linked and will no longer receive updates.',
	},
	lockedText: {
		id: 'instance.shared-instance.unavailable.locked-text',
		defaultMessage:
			'This shared instance was locked by the Content Moderation team. It will no longer receive updates from the primary instance and cannot be played.',
	},
	unavailableFallbackManager: {
		id: 'instance.shared-instance.unavailable.manager-fallback',
		defaultMessage: 'the instance manager',
	},
	errorTitle: {
		id: 'instance.shared-instance.error.title',
		defaultMessage: 'Something has gone wrong',
	},
})

export function sharedInstanceUnavailableTextMessage(
	reason: SharedInstanceUnavailableReason | null,
) {
	if (reason === 'deleted') return sharedInstanceErrorMessages.deletedText
	if (reason === 'access_revoked') return sharedInstanceErrorMessages.accessRevokedText
	if (reason === 'quarantined') return sharedInstanceErrorMessages.lockedText
	return sharedInstanceErrorMessages.unavailableText
}

export function sharedInstanceUnavailableTitleMessage(
	reason: SharedInstanceUnavailableReason | null,
) {
	return reason === 'quarantined'
		? sharedInstanceErrorMessages.lockedTitle
		: sharedInstanceErrorMessages.unavailableTitle
}

export function useSharedInstanceErrors() {
	const { formatMessage } = useVIntl()
	const { addNotification } = injectNotificationManager()

	function formatSharedInstanceUnavailable(
		reason: SharedInstanceUnavailableReason | null = null,
		manager?: string | null,
	) {
		return formatMessage(sharedInstanceUnavailableTextMessage(reason), {
			manager: manager || formatMessage(sharedInstanceErrorMessages.unavailableFallbackManager),
		})
	}

	function notifySharedInstanceUnavailable(
		reason: SharedInstanceUnavailableReason | null = null,
		manager?: string | null,
	) {
		addNotification({
			type: 'warning',
			title: formatMessage(sharedInstanceUnavailableTitleMessage(reason)),
			text: formatSharedInstanceUnavailable(reason, manager),
		})
	}

	function notifySharedInstanceError(error: unknown) {
		addNotification({
			type: 'error',
			title: formatMessage(sharedInstanceErrorMessages.errorTitle),
			text: getErrorMessage(error),
		})
	}

	return {
		formatSharedInstanceUnavailable,
		notifySharedInstanceError,
		notifySharedInstanceUnavailable,
	}
}
