import type { Archon } from '@modrinth/api-client'
import { PackageIcon } from '@modrinth/assets'
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
import UnknownEvent from './UnknownEvent.vue'
import UserAccessEvent from './UserAccessEvent.vue'

const basicEvents = new Set([
	'server_created',
	'server_reallocated',
	'modpack_unlinked',
	'server_repaired',
	'server_reset',
	'server_started',
	'server_stopped',
	'server_restarted',
	'server_killed',
	'sftp_login',
	'console_cleared',
])

export function parseAuditEvent(
	entry: Archon.Actions.v1.ActionEntry,
	lookups: AuditEventLookups,
): ParsedAuditEvent {
	const action = entry.action?.action || 'unknown'
	const metadata = entry.action?.metadata
	const base = baseProps(entry, lookups, action)

	try {
		if (basicEvents.has(action)) {
			return parsed(BasicStringEvent, base, {}, actionSearchParts(action))
		}

		switch (action) {
			case 'changed_server_name': {
				const record = metadataRecord(metadata)
				const name = stringField(record, 'name')
				if (!name) return unknown(base, action)
				return parsed(
					ServerMetaEvent,
					base,
					{ kind: 'name', name },
					actionSearchParts(action, name),
				)
			}
			case 'changed_server_subdomain': {
				const record = metadataRecord(metadata)
				const subdomain = stringField(record, 'subdomain')
				if (!subdomain) return unknown(base, action)
				return parsed(ServerMetaEvent, base, { kind: 'subdomain', subdomain }, [
					...actionSearchParts(action),
					subdomain,
				])
			}
			case 'server_plan_changed': {
				const record = metadataRecord(metadata)
				const newSpecs = objectField(record, 'new_specs')
				if (!newSpecs) return unknown(base, action)
				return parsed(ServerMetaEvent, base, { kind: 'plan', newSpecs }, [
					...actionSearchParts(action),
					...Object.values(newSpecs).map(String),
				])
			}
			case 'user_invited':
			case 'user_permission_modified': {
				const actionMetadata = userPermissionsActionMetadata(metadataRecord(metadata))
				if (!actionMetadata) return unknown(base, action)
				const kind = action === 'user_invited' ? 'invited' : 'permission_modified'
				const targetUser = userEntity(actionMetadata.user_id, lookups.users)
				return parsed(
					UserAccessEvent,
					base,
					{ kind, targetUser, permissions: actionMetadata.permissions },
					[...actionSearchParts(action), targetUser.label, actionMetadata.permissions],
				)
			}
			case 'user_invite_revoked':
			case 'user_removed': {
				const record = metadataRecord(metadata)
				const userId = stringField(record, 'user_id')
				if (!userId) return unknown(base, action)
				const kind = action === 'user_invite_revoked' ? 'invite_revoked' : 'removed'
				const targetUser = userEntity(userId, lookups.users)
				return parsed(UserAccessEvent, base, { kind, targetUser }, [
					...actionSearchParts(action),
					targetUser.label,
				])
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
					...actionSearchParts(action),
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
					...actionSearchParts(action),
					...fileNames,
				])
			}
			case 'modpack_changed': {
				const newVersionId = modpackVersionFromMetadata(metadataRecord(metadata))
				return parsed(ModpackEvent, base, { newVersionId }, [
					...actionSearchParts(action),
					newVersionId,
				])
			}
			case 'port_allocation_added':
			case 'port_allocation_removed': {
				const port = numberField(metadataRecord(metadata), 'port')
				if (port == null) return unknown(base, action)
				return parsed(
					NetworkEvent,
					base,
					{ kind: action === 'port_allocation_added' ? 'added' : 'removed', port },
					[...actionSearchParts(action), String(port)],
				)
			}
			case 'loader_version_edited': {
				const record = metadataRecord(metadata)
				if (!record || !('new_version' in record)) return unknown(base, action)
				const newVersion = record.new_version == null ? null : valueToString(record.new_version)
				return parsed(ConfigEvent, base, { kind: 'loader_version', newVersion }, [
					...actionSearchParts(action),
					newVersion,
				])
			}
			case 'game_version_edited': {
				const newVersion = stringField(metadataRecord(metadata), 'new_version')
				if (!newVersion) return unknown(base, action)
				return parsed(ConfigEvent, base, { kind: 'game_version', newVersion }, [
					...actionSearchParts(action),
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
					...actionSearchParts(action),
					...items.map((item) => item.label),
				])
			}
			case 'startup_command_modified': {
				const command = stringField(metadataRecord(metadata), 'command')
				if (!command) return unknown(base, action)
				return parsed(ConfigEvent, base, { kind: 'startup_command', command }, [
					...actionSearchParts(action),
					command,
				])
			}
			case 'java_runtime_modified': {
				const vendor = stringField(metadataRecord(metadata), 'vendor')
				if (!vendor) return unknown(base, action)
				return parsed(ConfigEvent, base, { kind: 'java_runtime', vendor }, [
					...actionSearchParts(action),
					vendor,
				])
			}
			case 'java_version_modified': {
				const version = numberField(metadataRecord(metadata), 'version')
				if (version == null) return unknown(base, action)
				return parsed(ConfigEvent, base, { kind: 'java_version', version }, [
					...actionSearchParts(action),
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
					...actionSearchParts(action),
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
					[...actionSearchParts(action), from, to],
				)
			}
			case 'console_command_executed': {
				const command = stringField(metadataRecord(metadata), 'command')
				if (!command) return unknown(base, action)
				return parsed(ConsoleEvent, base, { command }, [...actionSearchParts(action), command])
			}
			case 'backup_created':
			case 'backup_restored':
			case 'backup_deleted': {
				const id = stringField(metadataRecord(metadata), 'id')
				if (!id) return unknown(base, action)
				const kind = action.replace('backup_', '')
				const backup = backupEntity(id, lookups)
				return parsed(BackupEvent, base, { kind, backup: backup ?? undefined, backupId: id }, [
					...actionSearchParts(action),
					backup?.label,
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
				return parsed(
					BackupEvent,
					base,
					{ kind: 'renamed', backup: backup ?? undefined, backupId: id, from, to },
					[...actionSearchParts(action), backup?.label, from, to, id],
				)
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

function actionSearchParts(action: string, ...extra: unknown[]): unknown[] {
	return [action, action.replaceAll('_', ' '), ...extra]
}

function actorFromEntry(
	actor: Archon.Actions.v1.ActionUser,
	users: Record<string, Archon.Actions.v1.UserResp>,
): AuditActor {
	if (actor.type === 'support') {
		const user = actor.user_id ? users[actor.user_id] : undefined
		return {
			id: 'support',
			username: user?.username ? `Support (${user.username})` : 'support',
		}
	}

	const user = users[actor.user_id]
	return {
		id: actor.user_id,
		username: user?.username ?? actor.user_id,
		avatarUrl: user?.avatar_url || undefined,
		profilePath: user?.username ? `/user/${encodeURIComponent(user.username)}` : undefined,
	}
}

function userPermissionsActionMetadata(
	record: Record<string, unknown> | null,
): Archon.Actions.v1.UserPermissionsActionMetadata | null {
	const userId = stringField(record, 'user_id')
	if (!userId) return null

	return {
		user_id: userId,
		permissions: permissionField(record?.permissions),
	}
}

function permissionField(value: unknown): Archon.ServerUsers.v1.UserScope | null {
	if (typeof value === 'number' && Number.isFinite(value)) return value
	if (typeof value === 'string') return value.trim() || null
	if (Array.isArray(value)) {
		const permissions = value
			.filter((permission): permission is string => typeof permission === 'string')
			.map((permission) => permission.trim())
			.filter(Boolean)
		return permissions.length > 0 ? permissions.join(' | ') : null
	}
	return null
}

function worldFromId(
	worldId: string | null,
	worldById: Map<string, AuditWorld>,
): AuditWorld | null {
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
		addons.push(addonEntity(addonId, versionId, lookups.addons, lookups.versions))
	}
	return addons
}

function addonEntity(
	addonId: string,
	versionId: string,
	addons: Record<string, Archon.Actions.v1.AddonResp>,
	versions: Record<string, Archon.Actions.v1.VersionResp>,
): AuditAddonEventItem {
	const addon = addons[addonId]
	const versionLabel = resolveVersionLabel(versionId, versions)
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
			to: `/project/${encodeURIComponent(projectIdOrSlug)}/version/${encodeURIComponent(versionId)}`,
		},
	}
}

function resolveVersionLabel(
	versionId: string,
	versions: Record<string, Archon.Actions.v1.VersionResp>,
): string {
	const version = versions[versionId]
	return version?.version_number || version?.name || shortId(versionId)
}

function backupEntity(id: string, lookups: AuditEventLookups): AuditBackupEventItem | null {
	const backup = lookups.backupById.get(id)
	if (!backup) return null

	return {
		id,
		backupId: id,
		found: true,
		label: backup.name,
		to: {
			path: `/hosting/manage/${lookups.serverId}/backups`,
			query: { backup: id },
		},
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

function modpackVersionFromMetadata(record: Record<string, unknown> | null): string | null {
	const direct = valueToString(record?.new_version)
	if (direct != null) return direct

	const spec = metadataRecord(record?.spec)
	if (!spec) return null

	const versionId = valueToString(spec.version_id)
	if (versionId != null) return versionId

	return spec.platform === 'local_file' ? stringField(spec, 'filename') : null
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
