import type { Archon } from '@modrinth/api-client'
import type { Component } from 'vue'

export interface AuditActor {
	id: string
	username: string
	avatarUrl?: string
	profilePath?: string
}

export interface AuditWorld {
	id: string
	name: string
}

export interface BaseEventProps {
	action: string
	timestamp: string
	actor: AuditActor
	world: AuditWorld | null
}

export type EventRoute =
	| string
	| {
			path: string
			query?: Record<string, string | undefined>
	  }

export interface EventEntity {
	id: string
	label: string
	secondaryLabel?: string
	to?: EventRoute
	icon?: Component
	iconUrl?: string | null
	iconShape?: 'circle' | 'square'
	mono?: boolean
	muted?: boolean
	title?: string
}

export interface AuditAddonEventItem {
	addonId: string
	versionId: string
	project: EventEntity
	versionLabel: string
}

export interface AuditBackupEventItem extends EventEntity {
	backupId: string
	found: boolean
}

export interface ParsedAuditEvent {
	key: string
	component: Component
	props: BaseEventProps & Record<string, unknown>
	searchText: string
}

export interface AuditEventLookups {
	serverId: string
	users: Record<string, Archon.Actions.v1.UserResp>
	addons: Record<string, Archon.Actions.v1.AddonResp>
	versions: Record<string, Archon.Actions.v1.VersionResp>
	worldById: Map<string, AuditWorld>
	backupById: Map<string, Archon.Backups.v1.Backup>
}
