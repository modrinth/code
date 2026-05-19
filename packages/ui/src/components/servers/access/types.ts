import type { AuditActor, AuditWorld, ParsedAuditEvent } from './events/types'

export type ServerAccessRole = 'owner' | 'editor' | 'viewer'

export interface ServerAccessUser extends AuditActor {
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
	actor: AuditActor
	world: AuditWorld | null
	event: ParsedAuditEvent
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
