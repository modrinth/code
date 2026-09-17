export type InvitedPlayerMethod = 'direct' | 'link'

export type InvitedPlayerRow = {
	id: string
	username: string
	avatarUrl?: string
	lastPlayedAt: Date | null
	joinedAt: Date | null
	method: InvitedPlayerMethod
	pending?: boolean
}

export const invitedPlayerMethodLabels: Record<InvitedPlayerMethod, string> = {
	direct: 'Direct invite',
	link: 'Share link',
}
