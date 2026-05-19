import type { Archon } from '@modrinth/api-client'
import { PackageIcon, TrashIcon } from '@modrinth/assets'
import type { Component } from 'vue'

import AddonEvent from './AddonEvent.vue'
import BackupEvent from './BackupEvent.vue'
import BasicStringEvent from './BasicStringEvent.vue'
import ConfigEvent from './ConfigEvent.vue'
import ConsoleEvent from './ConsoleEvent.vue'
import FileEvent from './FileEvent.vue'
import ModpackEvent from './ModpackEvent.vue'
import NetworkEvent from './NetworkEvent.vue'
import ServerMetaEvent from './ServerMetaEvent.vue'
import UnknownEvent from './UnknownEvent.vue'
import UserAccessEvent from './UserAccessEvent.vue'
import type {
	AuditActor,
	AuditAddonEventItem,
	AuditBackupEventItem,
	AuditEventLookups,
	AuditWorld,
	BaseEventProps,
	EventEntity,
	ParsedAuditEvent,
} from './types'

const basicEvents: Record<string, string> = {
	server_created: 'Server created',
	server_reallocated: 'Server reallocated',
	modpack_unlinked: 'Modpack unlinked',
	server_repaired: 'Server repaired',
	server_reset: 'Server reset',
	server_started: 'Server started',
	server_stopped: 'Server stopped',
	server_restarted: 'Server restarted',
	server_killed: 'Server killed',
	sftp_login: 'SFTP login',
	console_cleared: 'Console cleared',
}

export function parseAuditEvent(
	entry: Archon.Actions.v1.ActionEntry,
	lookups: AuditEventLookups,
): ParsedAuditEvent {
	const action = entry.action?.action || 'unknown'
	const metadata = entry.action?.metadata
	const base = baseProps(entry, lookups, action)

	try {
		const basicEvent = basicEvents[action]
		if (basicEvent) {
			return parsed(BasicStringEvent, base, { label: basicEvent }, [basicEvent])
		}

		switch (action) {
			case 'changed_server_name': {
				const record = metadataRecord(metadata)
				const name = stringField(record, 'name')
				if (!name) return unknown(base, action)
				return parsed(ServerMetaEvent, base, { kind: 'name', name }, ['server name', name])
			}
			case 'changed_server_subdomain': {
				const record = metadataRecord(metadata)
				const subdomain = stringField(record, 'subdomain')
				if (!subdomain) return unknown(base, action)
				return parsed(ServerMetaEvent, base, { kind: 'subdomain', subdomain }, [
					'server subdomain',
					subdomain,
				])
			}
			case 'server_plan_changed': {
				const record = metadataRecord(metadata)
				const newSpecs = objectField(record, 'new_specs')
				if (!newSpecs) return unknown(base, action)
				return parsed(ServerMetaEvent, base, { kind: 'plan', newSpecs }, [
					'server plan',
					...Object.values(newSpecs).map(String),
				])
			}
			case 'user_invited':
			case 'user_permission_modified': {
				const record = metadataRecord(metadata)
				const userId = stringField(record, 'user_id')
				if (!userId) return unknown(base, action)
				const permissions = valueToString(record?.permissions)
				const kind = action === 'user_invited' ? 'invited' : 'permission_modified'
				const targetUser = userEntity(userId, lookups.users)
				return parsed(UserAccessEvent, base, { kind, targetUser, permissions }, [
					targetUser.label,
					permissions,
				])
			}
			case 'user_invite_revoked':
			case 'user_removed': {
				const record = metadataRecord(metadata)
				const userId = stringField(record, 'user_id')
				if (!userId) return unknown(base, action)
				const kind = action === 'user_invite_revoked' ? 'invite_revoked' : 'removed'
				const targetUser = userEntity(userId, lookups.users)
				return parsed(UserAccessEvent, base, { kind, targetUser }, [targetUser.label])
			}
			case 'addon_added':
			case 'addon_disabled':
			case 'addon_enabled':
			case 'addon_deleted':
			case 'addon_updated': {
				const addons = addonList(metadataRecord(metadata), lookups)
				if (!addons) return unknown(base, action)
				const kind = action.replace('addon_', '')
				return parsed(AddonEvent, base, { kind, addons }, [
					kind,
					...addons.flatMap((addon) => [
						addon.project.label,
						addon.addonId,
						addon.versionId,
						addon.versionLabel,
					]),
				])
			}
			case 'addon_uploaded': {
				const fileNames = stringArrayField(metadataRecord(metadata), 'file_names')
				if (!fileNames) return unknown(base, action)
				const files = fileNames.map((name) => fileEntity(name, lookups.serverId, false))
				return parsed(AddonEvent, base, { kind: 'uploaded', fileNames: files }, [
					'uploaded',
					...fileNames,
				])
			}
			case 'modpack_changed': {
				const record = metadataRecord(metadata)
				if (!record || !('new_version' in record)) return unknown(base, action)
				const newVersionId = record.new_version == null ? null : valueToString(record.new_version)
				return parsed(ModpackEvent, base, { newVersionId }, ['modpack', newVersionId])
			}
			case 'port_allocation_added':
			case 'port_allocation_removed': {
				const port = numberField(metadataRecord(metadata), 'port')
				if (port == null) return unknown(base, action)
				return parsed(
					NetworkEvent,
					base,
					{ kind: action === 'port_allocation_added' ? 'added' : 'removed', port },
					['port', String(port)],
				)
			}
			case 'loader_version_edited': {
				const record = metadataRecord(metadata)
				if (!record || !('new_version' in record)) return unknown(base, action)
				const newVersion = record.new_version == null ? null : valueToString(record.new_version)
				return parsed(ConfigEvent, base, { kind: 'loader_version', newVersion }, [
					'loader version',
					newVersion,
				])
			}
			case 'game_version_edited': {
				const newVersion = stringField(metadataRecord(metadata), 'new_version')
				if (!newVersion) return unknown(base, action)
				return parsed(ConfigEvent, base, { kind: 'game_version', newVersion }, [
					'game version',
					newVersion,
				])
			}
			case 'server_properties_modified': {
				const properties = objectField(metadataRecord(metadata), 'properties')
				if (!properties) return unknown(base, action)
				const items = Object.entries(properties).map(
					([key, value]): EventEntity => ({
						id: key,
						label: `${key}: ${valueToString(value) ?? ''}`,
						mono: true,
					}),
				)
				return parsed(ConfigEvent, base, { kind: 'properties', properties: items }, [
					'server properties',
					...items.map((item) => item.label),
				])
			}
			case 'startup_command_modified': {
				const command = stringField(metadataRecord(metadata), 'command')
				if (!command) return unknown(base, action)
				return parsed(ConfigEvent, base, { kind: 'startup_command', command }, [
					'startup command',
					command,
				])
			}
			case 'java_runtime_modified': {
				const vendor = stringField(metadataRecord(metadata), 'vendor')
				if (!vendor) return unknown(base, action)
				return parsed(ConfigEvent, base, { kind: 'java_runtime', vendor }, [
					'java runtime',
					vendor,
				])
			}
			case 'java_version_modified': {
				const version = numberField(metadataRecord(metadata), 'version')
				if (version == null) return unknown(base, action)
				return parsed(ConfigEvent, base, { kind: 'java_version', version }, [
					'java version',
					String(version),
				])
			}
			case 'file_uploaded':
			case 'file_deleted':
			case 'file_edited': {
				const path = stringField(metadataRecord(metadata), 'path')
				if (!path) return unknown(base, action)
				const kind = action.replace('file_', '')
				return parsed(FileEvent, base, { kind, file: fileEntity(path, lookups.serverId) }, [
					kind,
					path,
				])
			}
			case 'file_renamed': {
				const record = metadataRecord(metadata)
				const from = stringField(record, 'from')
				const to = stringField(record, 'to')
				if (!from || !to) return unknown(base, action)
				return parsed(
					FileEvent,
					base,
					{
						kind: 'renamed',
						from: fileEntity(from, lookups.serverId),
						to: fileEntity(to, lookups.serverId),
					},
					['renamed', from, to],
				)
			}
			case 'console_command_executed': {
				const command = stringField(metadataRecord(metadata), 'command')
				if (!command) return unknown(base, action)
				return parsed(ConsoleEvent, base, { command }, ['console command', command])
			}
			case 'backup_created':
			case 'backup_restored':
			case 'backup_deleted': {
				const id = stringField(metadataRecord(metadata), 'id')
				if (!id) return unknown(base, action)
				const kind = action.replace('backup_', '')
				const backup = backupEntity(id, lookups)
				return parsed(BackupEvent, base, { kind, backup, backupId: id }, [
					kind,
					backup.label,
					id,
				])
			}
			case 'backup_renamed': {
				const record = metadataRecord(metadata)
				const id = stringField(record, 'id')
				const from = stringField(record, 'from')
				const to = stringField(record, 'to')
				if (!id || !from || !to) return unknown(base, action)
				const backup = backupEntity(id, lookups)
				return parsed(BackupEvent, base, { kind: 'renamed', backup, backupId: id, from, to }, [
					'renamed',
					from,
					to,
					id,
				])
			}
			default:
				return unknown(base, action)
		}
	} catch {
		return unknown(base, action)
	}
}

function baseProps(
	entry: Archon.Actions.v1.ActionEntry,
	lookups: AuditEventLookups,
	action: string,
): BaseEventProps {
	return {
		action,
		timestamp: entry.timestamp,
		actor: actorFromEntry(entry.actor, lookups.users),
		world: worldFromId(entry.world_id ?? null, lookups.worldById),
	}
}

function parsed(
	component: Component,
	base: BaseEventProps,
	props: Record<string, unknown>,
	searchParts: unknown[],
): ParsedAuditEvent {
	return {
		key: base.action,
		component,
		props: { ...base, ...props },
		searchText: searchParts
			.filter((part): part is string => typeof part === 'string' && part.length > 0)
			.join(' ')
			.toLowerCase(),
	}
}

function unknown(base: BaseEventProps, rawAction: string): ParsedAuditEvent {
	return parsed(UnknownEvent, base, { rawAction }, [rawAction])
}

function actorFromEntry(
	actor: Archon.Actions.v1.ActionUser,
	users: Record<string, Archon.Actions.v1.UserResp>,
): AuditActor {
	if (actor.type === 'support') return { id: 'support', username: 'Support' }

	const user = users[actor.user_id]
	return {
		id: actor.user_id,
		username: user?.username ?? actor.user_id,
		avatarUrl: user?.avatar_url || undefined,
		profilePath: user?.username ? `/user/${encodeURIComponent(user.username)}` : undefined,
	}
}

function worldFromId(worldId: string | null, worldById: Map<string, AuditWorld>): AuditWorld | null {
	if (!worldId) return null
	return worldById.get(worldId) ?? { id: worldId, name: worldId }
}

function userEntity(
	userId: string,
	users: Record<string, Archon.Actions.v1.UserResp>,
): EventEntity {
	const user = users[userId]
	const label = user?.username ?? userId
	return {
		id: userId,
		label,
		iconUrl: user?.avatar_url || undefined,
		iconShape: 'circle',
		to: user?.username ? `/user/${encodeURIComponent(user.username)}` : undefined,
	}
}

function addonList(
	record: Record<string, unknown> | null,
	lookups: AuditEventLookups,
): AuditAddonEventItem[] | null {
	const list = arrayField(record, 'addons')
	if (!list) return null

	const addons: AuditAddonEventItem[] = []
	for (const item of list) {
		const addonRecord = metadataRecord(item)
		const addonId = stringField(addonRecord, 'addon_id')
		const versionId = stringField(addonRecord, 'version_id')
		if (!addonId || !versionId) return null
		addons.push(addonEntity(addonId, versionId, lookups.addons))
	}
	return addons
}

function addonEntity(
	addonId: string,
	versionId: string,
	addons: Record<string, Archon.Actions.v1.AddonResp>,
): AuditAddonEventItem {
	const addon = addons[addonId]
	const versionLabel = resolveVersionLabel(versionId)
	const projectIdOrSlug = addon?.slug || addonId
	return {
		addonId,
		versionId,
		versionLabel,
		project: {
			id: addonId,
			label: addon?.title || shortId(addonId),
			secondaryLabel: versionLabel,
			icon: PackageIcon,
			iconUrl: addon?.icon_url || undefined,
			iconShape: 'square',
			to: `/project/${encodeURIComponent(projectIdOrSlug)}`,
		},
	}
}

function resolveVersionLabel(versionId: string): string {
	// TODO: Replace this fallback with lookups.versions[version_id] once the backend exposes a versions map in ActionLogResponse.
	return shortId(versionId)
}

function backupEntity(id: string, lookups: AuditEventLookups): AuditBackupEventItem {
	const backup = lookups.backupById.get(id)
	return {
		id,
		backupId: id,
		found: !!backup,
		label: backup?.name ?? shortId(id),
		icon: backup ? undefined : TrashIcon,
		muted: !backup,
		to: backup
			? {
					path: `/hosting/manage/${lookups.serverId}/backups`,
					query: { backup: id },
				}
			: undefined,
	}
}

function fileEntity(path: string, serverId: string, link = true): EventEntity {
	return {
		id: path,
		label: path,
		mono: true,
		to: link
			? {
					path: `/hosting/manage/${serverId}/files`,
					query: {
						path: parentPath(path),
						editing: path,
					},
				}
			: undefined,
	}
}

function parentPath(path: string): string {
	const normalized = path.startsWith('/') ? path : `/${path}`
	const lastSlash = normalized.lastIndexOf('/')
	if (lastSlash <= 0) return '/'
	return normalized.slice(0, lastSlash)
}

function metadataRecord(value: unknown): Record<string, unknown> | null {
	if (!value || typeof value !== 'object' || Array.isArray(value)) return null
	return value as Record<string, unknown>
}

function objectField(
	record: Record<string, unknown> | null,
	key: string,
): Record<string, unknown> | null {
	return metadataRecord(record?.[key])
}

function arrayField(record: Record<string, unknown> | null, key: string): unknown[] | null {
	const value = record?.[key]
	return Array.isArray(value) ? value : null
}

function stringArrayField(record: Record<string, unknown> | null, key: string): string[] | null {
	const array = arrayField(record, key)
	if (!array || !array.every((item) => typeof item === 'string')) return null
	return array
}

function stringField(record: Record<string, unknown> | null, key: string): string | null {
	const value = record?.[key]
	return typeof value === 'string' && value.length > 0 ? value : null
}

function numberField(record: Record<string, unknown> | null, key: string): number | null {
	const value = record?.[key]
	return typeof value === 'number' && Number.isFinite(value) ? value : null
}

function valueToString(value: unknown): string | null {
	if (typeof value === 'string') return value
	if (typeof value === 'number' || typeof value === 'boolean') return String(value)
	return null
}

function shortId(id: string): string {
	if (id.length <= 12) return id
	return id.slice(0, 8)
}
