import type { RouteLocationRaw } from 'vue-router'

export type InvitePlayersUserStatus = 'available' | 'requested' | 'pending' | 'added'
export type InvitePlayersUserProfileLink =
	| RouteLocationRaw
	| (() => void | Promise<void>)
	| undefined

export interface InvitePlayersUser {
	id: string
	username: string
	avatarUrl?: string | null
	status?: InvitePlayersUserStatus
	online?: boolean
}

export interface InvitePlayersSearchUser {
	id: string
	username: string
	avatarUrl?: string | null
	email?: string
}

export interface InvitePlayersInvitePayload {
	user: InvitePlayersUser
	source: 'friend' | 'search'
}

export interface InviteLinkSettings {
	expiresAt: Date
	maxUses: number
}
