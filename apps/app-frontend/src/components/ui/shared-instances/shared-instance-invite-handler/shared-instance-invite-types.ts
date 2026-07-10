export type SharedInstanceInviteNotificationBody = {
	type: 'shared_instance_invite'
	shared_instance_id?: unknown
	shared_instance_name?: unknown
	invited_by?: unknown
	invited_by_username?: unknown
	invited_by_avatar_url?: unknown
	instance_icon_url?: unknown
}

export type AppNotification = {
	id: string | number
	read?: boolean
	body?: { type?: unknown } & Record<string, unknown>
}

export type SharedInstanceInvite = {
	sharedInstanceId: string
	sharedInstanceName: string
	invitedById: string | null
	invitedByUsername: string | null
	invitedByAvatarUrl: string | null
	instanceIconUrl: string | null
}

export type SharedInstanceInviteHandler = {
	handleNotification(notification: AppNotification): Promise<boolean>
	installFromInviteId(inviteId: string): Promise<void>
	showManualInviteLinkModal(event?: MouseEvent): void
}
