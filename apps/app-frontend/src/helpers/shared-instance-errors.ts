import { ModrinthApiError } from '@modrinth/api-client'
import { defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'

import {
	getErrorMessage,
	isSharedInstanceFileNotAllowedError,
	isSharedInstanceLimitReachedError,
	isSharedInstanceOwnerBehindError,
	isSharedInstancesApiError,
	type SharedInstanceUnavailableReason,
} from '@/helpers/install'
import type { GameInstance } from '@/helpers/types'

type SharedInstanceRole = NonNullable<GameInstance['shared_instance']>['role']

export const sharedInstanceErrorMessages = defineMessages({
	unavailableTitle: {
		id: 'instance.shared-instance.unavailable.title',
		defaultMessage: 'Shared instance no longer available',
	},
	notFoundTitle: {
		id: 'instance.shared-instance.unavailable.not-found-title',
		defaultMessage: 'Shared instance unavailable',
	},
	lockedTitle: {
		id: 'instance.shared-instance.unavailable.locked-title',
		defaultMessage: 'Instance locked',
	},
	lockedOwnerTitle: {
		id: 'instance.shared-instance.unavailable.locked-owner-title',
		defaultMessage: 'Instance no longer sharable',
	},
	unavailableText: {
		id: 'instance.shared-instance.unavailable.text',
		defaultMessage:
			"Your local instance is still available, but it is no longer linked and won't receive updates.",
	},
	deletedText: {
		id: 'instance.shared-instance.unavailable.not-found-text',
		defaultMessage:
			'We couldn’t find this shared instance. Your installed content is still on this device.',
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
	lockedOwnerText: {
		id: 'instance.shared-instance.unavailable.locked-owner-text',
		defaultMessage:
			'The shared instance was locked by the Content Moderation team. You are no longer allowed to push updates or re-share this instance.',
	},
	unavailableFallbackManager: {
		id: 'instance.shared-instance.unavailable.manager-fallback',
		defaultMessage: 'the instance manager',
	},
	errorTitle: {
		id: 'instance.shared-instance.error.title',
		defaultMessage: 'Something has gone wrong',
	},
	networkErrorTitle: {
		id: 'instance.shared-instance.network-error.title',
		defaultMessage: 'Network error',
	},
	networkErrorText: {
		id: 'instance.shared-instance.network-error.text',
		defaultMessage: 'Unable to connect to shared instances API',
	},
	ownerBehindTitle: {
		id: 'instance.shared-instance.owner-behind.title',
		defaultMessage: "Couldn't push update",
	},
	ownerBehindText: {
		id: 'instance.shared-instance.owner-behind.text',
		defaultMessage:
			'A newer version was pushed from another Modrinth App installation. Download it, then try again.',
	},
	fileNotAllowedTitle: {
		id: 'instance.shared-instance.file-not-allowed.title',
		defaultMessage: 'File type not allowed',
	},
	limitReachedTitle: {
		id: 'instance.shared-instance.limit-reached.title',
		defaultMessage: 'Shared instance limit reached',
	},
	limitReachedText: {
		id: 'instance.shared-instance.limit-reached.text',
		defaultMessage: "You can't share any more instances.",
	},
})

export function sharedInstanceUnavailableTextMessage(
	reason: SharedInstanceUnavailableReason | null,
	role?: SharedInstanceRole | null,
) {
	if (reason === 'deleted') return sharedInstanceErrorMessages.deletedText
	if (reason === 'access_revoked') return sharedInstanceErrorMessages.accessRevokedText
	if (reason === 'quarantined') {
		return role === 'owner'
			? sharedInstanceErrorMessages.lockedOwnerText
			: sharedInstanceErrorMessages.lockedText
	}
	return sharedInstanceErrorMessages.unavailableText
}

export function sharedInstanceUnavailableTitleMessage(
	reason: SharedInstanceUnavailableReason | null,
	role?: SharedInstanceRole | null,
) {
	if (reason === 'deleted') return sharedInstanceErrorMessages.notFoundTitle
	if (reason === 'quarantined') {
		return role === 'owner'
			? sharedInstanceErrorMessages.lockedOwnerTitle
			: sharedInstanceErrorMessages.lockedTitle
	}
	return sharedInstanceErrorMessages.unavailableTitle
}

export function useSharedInstanceErrors() {
	const { formatMessage } = useVIntl()
	const { addNotification } = injectNotificationManager()

	function formatSharedInstanceUnavailable(
		reason: SharedInstanceUnavailableReason | null = null,
		manager?: string | null,
		role?: SharedInstanceRole | null,
	) {
		return formatMessage(sharedInstanceUnavailableTextMessage(reason, role), {
			manager: manager || formatMessage(sharedInstanceErrorMessages.unavailableFallbackManager),
		})
	}

	function notifySharedInstanceUnavailable(
		reason: SharedInstanceUnavailableReason | null = null,
		manager?: string | null,
		role?: SharedInstanceRole | null,
	) {
		addNotification({
			type: 'warning',
			title: formatMessage(sharedInstanceUnavailableTitleMessage(reason, role)),
			text: formatSharedInstanceUnavailable(reason, manager, role),
		})
	}

	function notifySharedInstanceConnectionError() {
		addNotification({
			type: 'error',
			title: formatMessage(sharedInstanceErrorMessages.networkErrorTitle),
			text: formatMessage(sharedInstanceErrorMessages.networkErrorText),
		})
	}

	function notifySharedInstanceError(error: unknown) {
		if (isSharedInstancesApiError(error)) {
			notifySharedInstanceConnectionError()
			return
		}

		if (isSharedInstanceOwnerBehindError(error)) {
			addNotification({
				type: 'error',
				title: formatMessage(sharedInstanceErrorMessages.ownerBehindTitle),
				text: formatMessage(sharedInstanceErrorMessages.ownerBehindText),
			})
			return
		}

		if (isSharedInstanceLimitReachedError(error)) {
			addNotification({
				type: 'error',
				title: formatMessage(sharedInstanceErrorMessages.limitReachedTitle),
				text: formatMessage(sharedInstanceErrorMessages.limitReachedText),
			})
			return
		}

		const message = getErrorMessage(error)
		const status = error instanceof ModrinthApiError ? error.statusCode : undefined

		addNotification({
			type: 'error',
			title: formatMessage(
				isSharedInstanceFileNotAllowedError(error)
					? sharedInstanceErrorMessages.fileNotAllowedTitle
					: sharedInstanceErrorMessages.errorTitle,
			),
			text: status && !message.includes(`HTTP ${status}`) ? `HTTP ${status}: ${message}` : message,
		})
	}

	return {
		formatSharedInstanceUnavailable,
		notifySharedInstanceConnectionError,
		notifySharedInstanceError,
		notifySharedInstanceUnavailable,
	}
}
