import type {
	AppNotification,
	SharedInstanceInvite,
	SharedInstanceInviteNotificationBody,
} from './shared-instance-invite-types'

function optionalString(value: unknown) {
	return typeof value === 'string' ? value : null
}

export function parseSharedInstanceInviteNotification(
	notification: AppNotification,
): SharedInstanceInvite | null {
	if (notification.body?.type !== 'shared_instance_invite') return null

	const body = notification.body as SharedInstanceInviteNotificationBody
	const sharedInstanceId = optionalString(body.shared_instance_id)
	const sharedInstanceName = optionalString(body.shared_instance_name)
	if (!sharedInstanceId || !sharedInstanceName) return null

	return {
		sharedInstanceId,
		sharedInstanceName,
		invitedById: optionalString(body.invited_by),
		invitedByUsername: optionalString(body.invited_by_username),
		invitedByAvatarUrl: optionalString(body.invited_by_avatar_url),
		instanceIconUrl: optionalString(body.instance_icon_url),
	}
}

export function parseSharedInstanceInviteLink(value: string) {
	const trimmedValue = value.trim()
	if (!trimmedValue) return null

	try {
		const url = new URL(trimmedValue)
		const match = /^\/share\/([^/]+)\/?$/.exec(url.pathname)
		return match ? decodeURIComponent(match[1]) : null
	} catch {
		return null
	}
}
