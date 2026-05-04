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
	pending?: boolean
	isOwner?: boolean
}

export type ServerAuditAction =
	| { type: 'file_edited'; file: string }
	| { type: 'world_started'; worldName: string }
	| { type: 'content_installed'; contentType: 'mod' | 'modpack'; name: string; iconUrl?: string }
	| { type: 'member_invited' | 'member_removed' | 'role_changed'; target: string }

export type ServerAuditActionType = ServerAuditAction['type']

export interface ServerAuditLogEntry {
	id: string
	actor: ServerAccessUser | { id: 'support'; username: 'Support' }
	world: { id: string; name: string } | null
	action: ServerAuditAction
	timestamp: string
}

export interface ServerAuditLogFilters {
	userId: string | null
	worldId: string | null
	actionType: ServerAuditActionType | null
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
	sendFriendRequest: boolean
}

export interface ServerListingOwner {
	username: string
	avatarUrl?: string
}
