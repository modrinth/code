import { defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'

import { getErrorMessage, type SharedInstanceUnavailableReason } from '@/helpers/install'

export const sharedInstanceErrorMessages = defineMessages({
	unavailableTitle: {
		id: 'instance.shared-instance.unavailable.title',
		defaultMessage: 'Shared instance no longer available',
	},
	quarantinedTitle: {
		id: 'instance.shared-instance.unavailable.quarantined-title',
		defaultMessage: 'This instance has been quarantined',
	},
	unavailableText: {
		id: 'instance.shared-instance.unavailable.text-v2',
		defaultMessage:
			"Your local instance is still available, but it is no longer linked and won't receive updates.",
	},
	deletedText: {
		id: 'instance.shared-instance.unavailable.deleted-text-v2',
		defaultMessage:
			"The shared instance was deleted. Your local instance is still available, but it is no longer linked and won't receive updates.",
	},
	accessRevokedText: {
		id: 'instance.shared-instance.unavailable.access-revoked-text-v2',
		defaultMessage:
			"Your access to the shared instance was revoked. Your local instance is still available, but it is no longer linked and won't receive updates.",
	},
	quarantinedText: {
		id: 'instance.shared-instance.unavailable.quarantined-text',
		defaultMessage:
			'The user who managed this shared instance has been restricted from sharing by the moderation team. This instance has been locked and cannot be played due to safety issues.',
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
	if (reason === 'quarantined') return sharedInstanceErrorMessages.quarantinedText
	return sharedInstanceErrorMessages.unavailableText
}

export function sharedInstanceUnavailableTitleMessage(
	reason: SharedInstanceUnavailableReason | null,
) {
	return reason === 'quarantined'
		? sharedInstanceErrorMessages.quarantinedTitle
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
