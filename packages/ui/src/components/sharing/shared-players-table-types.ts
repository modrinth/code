export type SharedPlayersTableMethod = 'direct' | 'link'
export type SharedPlayersTableMethodFilter = SharedPlayersTableMethod | 'all'
export type SharedPlayersTableColumn = 'username' | 'lastPlayed' | 'joined' | 'method' | 'actions'
export type SharedPlayersTableUserProfileLink = string | (() => void) | undefined

export type SharedPlayersTableRow = {
	id: string
	username: string
	avatarUrl?: string
	lastPlayedAt: Date | null
	joinedAt: Date | null
	method: SharedPlayersTableMethod
	pending?: boolean
}

export const sharedPlayersTableMethodLabels: Record<SharedPlayersTableMethod, string> = {
	direct: 'Direct invite',
	link: 'Share link',
}
