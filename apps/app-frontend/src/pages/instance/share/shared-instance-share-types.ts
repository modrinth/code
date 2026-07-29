export type ShareMethod = 'direct' | 'link'
export type MethodFilter = ShareMethod | 'all'
export type ShareTableColumn = 'username' | 'lastPlayed' | 'joined' | 'method' | 'actions'

export const SHARED_INSTANCE_USER_LIMIT = 50

export type ShareRow = {
	id: string
	username: string
	avatarUrl?: string
	lastPlayedAt: Date | null
	joinedAt: Date | null
	method: ShareMethod
	pending?: boolean
}

export const methodLabels: Record<ShareMethod, string> = {
	direct: 'Direct invite',
	link: 'Share link',
}

export { normalizeInviteKey } from '@modrinth/ui'
