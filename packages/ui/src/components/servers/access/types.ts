export type ServerAccessRole = 'owner' | 'editor' | 'viewer'

export interface ServerAccessUser {
	id: string
	username: string
	avatarUrl?: string
}

export interface ServerAccessMember {
	id: string
	user: ServerAccessUser
	role: ServerAccessRole
	joinedAt: string | null
	inviteResendAvailableAt?: string | null
	pending?: boolean
	isOwner?: boolean
}

export interface ServerAuditLogEntry {
	id: string
	actor: ServerAccessUser | { id: 'support'; username: 'Support' }
	world: { id: string; name: string } | null
	timestamp: string
}

export interface ServerAuditLogFilters {
	userId: string | null
	worldId: string | null
}

export interface ServerAccessRoleOption {
	value: ServerAccessRole
	label: string
	description?: string
}

export interface ServerAccessInviteSuggestion {
	id: string
	username: string
	avatarUrl?: string
	email?: string
}

export interface GrantServerAccessPayload {
	target: string
	role: Exclude<ServerAccessRole, 'owner'>
}

export interface ServerListingOwner {
	username: string
	avatarUrl?: string
}
