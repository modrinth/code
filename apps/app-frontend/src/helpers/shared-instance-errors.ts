import { defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'

import { getErrorMessage, type SharedInstanceUnavailableReason } from '@/helpers/install'

export const sharedInstanceErrorMessages = defineMessages({
	unavailableTitle: {
		id: 'instance.shared-instance.unavailable.title',
		defaultMessage: 'Shared instance no longer available',
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
	return sharedInstanceErrorMessages.unavailableText
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
			title: formatMessage(sharedInstanceErrorMessages.unavailableTitle),
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
