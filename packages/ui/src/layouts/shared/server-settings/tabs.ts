import type { Archon } from '@modrinth/api-client'
import {
	CardIcon,
	ListIcon,
	ModrinthIcon,
	SettingsIcon,
	TextQuoteIcon,
	VersionIcon,
	WrenchIcon,
} from '@modrinth/assets'
import type { Component } from 'vue'

export type ServerSettingsTabId =
	| 'general'
	| 'installation'
	| 'network'
	| 'properties'
	| 'advanced'
	| 'billing'
	| 'admin-billing'

export interface ServerSettingsTabContext {
	serverId: string
	ownerId: string
	serverStatus?: Archon.Servers.v0.Status | null
	isOwner: boolean
	isAdmin: boolean
}

export interface ServerSettingsTabDefinition {
	id: ServerSettingsTabId
	label: string
	icon: Component
	href: (ctx: ServerSettingsTabContext) => string
	external?: boolean
	shown?: (ctx: ServerSettingsTabContext) => boolean
}

export const serverSettingsTabDefinitions: ServerSettingsTabDefinition[] = [
	{
		id: 'general',
		label: 'General',
		icon: SettingsIcon,
		href: ({ serverId }) => `/hosting/manage/${serverId}/options`,
	},
	{
		id: 'installation',
		label: 'Installation',
		icon: WrenchIcon,
		href: ({ serverId }) => `/hosting/manage/${serverId}/options/loader`,
	},
	{
		id: 'network',
		label: 'Network',
		icon: VersionIcon,
		href: ({ serverId }) => `/hosting/manage/${serverId}/options/network`,
	},
	{
		id: 'properties',
		label: 'Properties',
		icon: ListIcon,
		href: ({ serverId }) => `/hosting/manage/${serverId}/options/properties`,
		shown: ({ serverStatus }) => serverStatus !== 'installing',
	},
	{
		id: 'advanced',
		label: 'Advanced',
		icon: TextQuoteIcon,
		href: ({ serverId }) => `/hosting/manage/${serverId}/options/advanced`,
	},
	{
		id: 'billing',
		label: 'Billing',
		icon: CardIcon,
		href: ({ serverId }) => `/settings/billing#server-${serverId}`,
		external: true,
		shown: ({ isOwner }) => isOwner,
	},
	{
		id: 'admin-billing',
		label: 'Admin Billing',
		icon: ModrinthIcon,
		href: ({ ownerId }) => `/admin/billing/${ownerId}`,
		external: true,
		shown: ({ isAdmin }) => isAdmin,
	},
]

export function getServerSettingsNavLinks(ctx: ServerSettingsTabContext) {
	return serverSettingsTabDefinitions.map((tab) => ({
		id: tab.id,
		icon: tab.icon,
		label: tab.label,
		href: tab.href(ctx),
		external: tab.external,
		shown: tab.shown ? tab.shown(ctx) : true,
	}))
}
