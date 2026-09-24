import type { Archon } from '@modrinth/api-client'
import type { QueryClient } from '@tanstack/vue-query'

export type AddonToggleChanges = {
	enabled?: boolean
	server?: boolean
	player?: boolean
}

type ToggleField = keyof AddonToggleChanges
type AddonIdentity = Pick<Archon.Content.v1.Addon, 'kind' | 'filename'>
type PendingToggle = {
	changes: AddonToggleChanges
	sent: Partial<Record<ToggleField, boolean>>
}

const pendingToggles = new WeakMap<QueryClient, Map<string, PendingToggle>>()

export function addonToggleKey(addon: AddonIdentity) {
	return `${addon.kind}:${normalizeAddonFilename(addon.filename)}`
}

export function normalizeAddonFilename(filename: string) {
	return filename.endsWith('.disabled') ? filename.slice(0, -'.disabled'.length) : filename
}

function scopedToggleKey(serverId: string, worldId: string, addon: AddonIdentity) {
	return `${serverId}:${worldId}:${addonToggleKey(addon)}`
}

function getPendingToggle(
	queryClient: QueryClient,
	serverId: string,
	worldId: string,
	addon: AddonIdentity,
) {
	return pendingToggles.get(queryClient)?.get(scopedToggleKey(serverId, worldId, addon))
}

function removeEmptyToggle(
	queryClient: QueryClient,
	serverId: string,
	worldId: string,
	addon: AddonIdentity,
	pending: PendingToggle,
) {
	if (Object.values(pending.changes).some((value) => value !== undefined)) return
	pendingToggles.get(queryClient)?.delete(scopedToggleKey(serverId, worldId, addon))
}

function clearPendingField(pending: PendingToggle, field: ToggleField) {
	switch (field) {
		case 'enabled':
			delete pending.changes.enabled
			delete pending.sent.enabled
			break
		case 'server':
			delete pending.changes.server
			delete pending.sent.server
			break
		case 'player':
			delete pending.changes.player
			delete pending.sent.player
			break
	}
}

export function queuePendingAddonToggle(
	queryClient: QueryClient,
	serverId: string,
	worldId: string,
	addon: AddonIdentity,
	changes: AddonToggleChanges,
) {
	let toggles = pendingToggles.get(queryClient)
	if (!toggles) {
		toggles = new Map()
		pendingToggles.set(queryClient, toggles)
	}
	const key = scopedToggleKey(serverId, worldId, addon)
	const pending: PendingToggle = toggles.get(key) ?? { changes: {}, sent: {} }
	for (const field of Object.keys(changes) as ToggleField[]) {
		pending.changes[field] = changes[field]
		pending.sent[field] = false
	}
	toggles.set(key, pending)
}

export function markPendingAddonToggleSent(
	queryClient: QueryClient,
	serverId: string,
	worldId: string,
	addon: AddonIdentity,
	changes: AddonToggleChanges,
) {
	const pending = getPendingToggle(queryClient, serverId, worldId, addon)
	if (!pending) return
	for (const field of Object.keys(changes) as ToggleField[]) {
		if (pending.changes[field] === changes[field]) pending.sent[field] = true
	}
}

export function confirmPendingAddonToggle(
	queryClient: QueryClient,
	serverId: string,
	worldId: string,
	addon: AddonIdentity,
	field: ToggleField,
	value: boolean,
) {
	const pending = getPendingToggle(queryClient, serverId, worldId, addon)
	if (!pending || !pending.sent[field] || pending.changes[field] !== value) return
	clearPendingField(pending, field)
	removeEmptyToggle(queryClient, serverId, worldId, addon, pending)
}

export function discardFailedAddonToggle(
	queryClient: QueryClient,
	serverId: string,
	worldId: string,
	addon: AddonIdentity,
	changes: AddonToggleChanges,
) {
	const pending = getPendingToggle(queryClient, serverId, worldId, addon)
	if (!pending) return
	for (const field of Object.keys(changes) as ToggleField[]) {
		if (!pending.sent[field] || pending.changes[field] !== changes[field]) continue
		clearPendingField(pending, field)
	}
	removeEmptyToggle(queryClient, serverId, worldId, addon, pending)
}

export function clearPendingAddonToggle(
	queryClient: QueryClient,
	serverId: string,
	worldId: string,
	addon: AddonIdentity,
) {
	pendingToggles.get(queryClient)?.delete(scopedToggleKey(serverId, worldId, addon))
}

export function clearPendingAddonTogglesForServer(queryClient: QueryClient, serverId: string) {
	const toggles = pendingToggles.get(queryClient)
	if (!toggles) return
	for (const key of toggles.keys()) {
		if (key.startsWith(`${serverId}:`)) toggles.delete(key)
	}
}

export function clearPendingAddonToggleField(
	queryClient: QueryClient,
	serverId: string,
	worldId: string,
	addon: AddonIdentity,
	field: ToggleField,
) {
	const pending = getPendingToggle(queryClient, serverId, worldId, addon)
	if (!pending?.sent[field]) return
	clearPendingField(pending, field)
	removeEmptyToggle(queryClient, serverId, worldId, addon, pending)
}

export function applyPendingAddonToggle(
	queryClient: QueryClient,
	serverId: string,
	worldId: string,
	addon: Archon.Content.v1.Addon,
): Archon.Content.v1.Addon {
	const changes = getPendingToggle(queryClient, serverId, worldId, addon)?.changes
	if (!changes) return addon
	return {
		...addon,
		...(changes.enabled !== undefined ? { disabled: !changes.enabled } : {}),
		...(changes.server !== undefined ? { disabled_server: !changes.server } : {}),
		...(changes.player !== undefined ? { disabled_player: !changes.player } : {}),
		...((changes.server !== undefined || changes.player !== undefined) &&
		addon.kind !== 'resourcepack' &&
		addon.kind !== 'shader'
			? { side_toggle_unlocked: true }
			: {}),
	}
}
