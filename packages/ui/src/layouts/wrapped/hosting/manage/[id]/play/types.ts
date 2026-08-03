export type ServerPlayerMethod = 'direct' | 'link'

export type ServerPlayerRow = {
	id: string
	username: string
	avatarUrl?: string
	lastPlayedAt: Date | null
	joinedAt: Date | null
	method: ServerPlayerMethod
	pending?: boolean
}
