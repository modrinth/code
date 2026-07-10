export type ShareMethod = 'direct' | 'link'
export type MethodFilter = ShareMethod | 'all'
export type ShareTableColumn = 'username' | 'lastPlayed' | 'joined' | 'method' | 'actions'

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

export function normalizeInviteKey(value: string) {
	return value.trim().toLowerCase()
}

export function toError(error: unknown) {
	if (error instanceof Error) return error
	if (typeof error === 'string') return new Error(error)
	if (error && typeof error === 'object') {
		const record = error as Record<string, unknown>
		const message = record.message ?? record.error
		if (typeof message === 'string') return new Error(message)
		return new Error(JSON.stringify(error))
	}
	return new Error(String(error))
}
