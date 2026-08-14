import type {
	ContentCardProject,
	ContentCardVersion,
	ContentOwner,
} from '../layouts/shared/content-tab/types'

export type PendingServerContentInstallType = 'mod' | 'plugin' | 'datapack'
type PendingServerContentOwner = Omit<ContentOwner, 'link'> & { link?: string }

export interface PendingServerContentInstall {
	projectId: string
	versionId: string
	contentType: PendingServerContentInstallType
	title: ContentCardProject['title']
	versionName?: ContentCardVersion['version_number'] | null
	versionNumber?: ContentCardVersion['version_number'] | null
	fileName?: ContentCardVersion['file_name'] | null
	owner?: PendingServerContentOwner | null
	slug?: ContentCardProject['slug'] | null
	iconUrl?: ContentCardProject['icon_url'] | null
	createdAt: number
}

interface PendingServerContentInstallBaseline {
	contentKeys: string[]
	projectIds?: string[]
	createdAt: number
}

export const pendingServerContentInstallsEvent = 'modrinth:pending-server-content-installs'

const stalePendingInstallAge = 30 * 60 * 1000

function getPendingServerContentInstallsKey(serverId: string | null, worldId: string | null) {
	if (!serverId || !worldId) return null
	return `server-content-installing:${serverId}:${worldId}`
}

function getPendingServerContentInstallBaselineKey(
	serverId: string | null,
	worldId: string | null,
) {
	if (!serverId || !worldId) return null
	return `server-content-installing-baseline:${serverId}:${worldId}`
}

function isPendingServerContentInstall(value: unknown): value is PendingServerContentInstall {
	if (!value || typeof value !== 'object') return false
	const record = value as Record<string, unknown>
	return (
		typeof record.projectId === 'string' &&
		typeof record.versionId === 'string' &&
		(record.contentType === 'mod' ||
			record.contentType === 'plugin' ||
			record.contentType === 'datapack') &&
		typeof record.title === 'string' &&
		typeof record.createdAt === 'number'
	)
}

function isPendingServerContentInstallBaseline(
	value: unknown,
): value is PendingServerContentInstallBaseline {
	if (!value || typeof value !== 'object') return false
	const record = value as Record<string, unknown>
	const contentKeys = record.contentKeys ?? record.projectIds
	return (
		Array.isArray(contentKeys) &&
		contentKeys.every((contentKey) => typeof contentKey === 'string') &&
		typeof record.createdAt === 'number'
	)
}

function filterFreshPendingServerContentInstalls(items: PendingServerContentInstall[]) {
	const cutoff = Date.now() - stalePendingInstallAge
	return items.filter((item) => item.createdAt >= cutoff)
}

function isFreshPendingServerContentInstallBaseline(item: PendingServerContentInstallBaseline) {
	return item.createdAt >= Date.now() - stalePendingInstallAge
}

function emitPendingServerContentInstallsChanged(serverId: string | null, worldId: string | null) {
	if (typeof window === 'undefined') return
	window.dispatchEvent(
		new CustomEvent(pendingServerContentInstallsEvent, {
			detail: { serverId, worldId },
		}),
	)
}

export function readPendingServerContentInstalls(serverId: string | null, worldId: string | null) {
	const key = getPendingServerContentInstallsKey(serverId, worldId)
	if (!key || typeof localStorage === 'undefined') return []

	try {
		const raw = localStorage.getItem(key)
		if (!raw) return []
		const parsed = JSON.parse(raw)
		if (!Array.isArray(parsed)) return []
		const freshItems = filterFreshPendingServerContentInstalls(
			parsed.filter(isPendingServerContentInstall),
		)
		if (freshItems.length !== parsed.length) {
			writePendingServerContentInstalls(serverId, worldId, freshItems)
		}
		return freshItems
	} catch {
		return []
	}
}

export function writePendingServerContentInstalls(
	serverId: string | null,
	worldId: string | null,
	items: PendingServerContentInstall[],
) {
	const key = getPendingServerContentInstallsKey(serverId, worldId)
	if (!key || typeof localStorage === 'undefined') return

	const freshItems = filterFreshPendingServerContentInstalls(items)
	if (freshItems.length === 0) {
		localStorage.removeItem(key)
		const baselineKey = getPendingServerContentInstallBaselineKey(serverId, worldId)
		if (baselineKey) {
			localStorage.removeItem(baselineKey)
		}
	} else {
		localStorage.setItem(key, JSON.stringify(freshItems))
	}
	emitPendingServerContentInstallsChanged(serverId, worldId)
}

export function readPendingServerContentInstallBaseline(
	serverId: string | null,
	worldId: string | null,
) {
	const key = getPendingServerContentInstallBaselineKey(serverId, worldId)
	if (!key || typeof localStorage === 'undefined') return null

	try {
		const raw = localStorage.getItem(key)
		if (!raw) return null
		const parsed = JSON.parse(raw)
		if (!isPendingServerContentInstallBaseline(parsed)) return null
		if (!isFreshPendingServerContentInstallBaseline(parsed)) {
			localStorage.removeItem(key)
			return null
		}
		return new Set(parsed.contentKeys ?? parsed.projectIds)
	} catch {
		return null
	}
}

export function writePendingServerContentInstallBaseline(
	serverId: string | null,
	worldId: string | null,
	contentKeys: Iterable<string>,
) {
	const key = getPendingServerContentInstallBaselineKey(serverId, worldId)
	if (!key || typeof localStorage === 'undefined') return

	localStorage.setItem(
		key,
		JSON.stringify({
			contentKeys: Array.from(new Set(contentKeys)),
			createdAt: Date.now(),
		} satisfies PendingServerContentInstallBaseline),
	)
	emitPendingServerContentInstallsChanged(serverId, worldId)
}

export function addPendingServerContentInstalls(
	serverId: string | null,
	worldId: string | null,
	items: Omit<PendingServerContentInstall, 'createdAt'>[],
) {
	if (items.length === 0) return

	const now = Date.now()
	const next = new Map(
		readPendingServerContentInstalls(serverId, worldId).map((item) => [item.projectId, item]),
	)
	for (const item of items) {
		next.set(item.projectId, { ...item, createdAt: now })
	}
	writePendingServerContentInstalls(serverId, worldId, Array.from(next.values()))
}

export function removePendingServerContentInstall(
	serverId: string | null,
	worldId: string | null,
	projectId: string,
) {
	writePendingServerContentInstalls(
		serverId,
		worldId,
		readPendingServerContentInstalls(serverId, worldId).filter(
			(item) => item.projectId !== projectId,
		),
	)
}
