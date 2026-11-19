import type { BrowseInstallPlan, BrowseInstallProject } from './plan'
import type { BrowseInstallContentType, BrowseInstallPreferences } from './preferences'

const serverInstallQueueStoragePrefix = 'server-install-queue'
const serverInstallQueueLockStoragePrefix = 'server-install-queue-lock'
const serverInstallQueueLockTtl = 15 * 60 * 1000
const serverInstallQueueLockRefreshInterval = 30 * 1000
const activeInstallQueueFlushes = new Map<string, Promise<unknown>>()

export function getStoredServerInstallQueueKey(serverId: string | null, worldId: string | null) {
	if (!serverId || !worldId) return null
	return `${serverInstallQueueStoragePrefix}:${serverId}:${worldId}`
}

export function readStoredServerInstallQueue<
	TProject extends BrowseInstallProject = BrowseInstallProject,
>(serverId: string | null, worldId: string | null) {
	const key = getStoredServerInstallQueueKey(serverId, worldId)
	if (!key || typeof localStorage === 'undefined') {
		return new Map<string, BrowseInstallPlan<TProject>>()
	}

	try {
		const raw = localStorage.getItem(key)
		if (!raw) return new Map<string, BrowseInstallPlan<TProject>>()

		const parsed = JSON.parse(raw)
		if (!Array.isArray(parsed)) return new Map<string, BrowseInstallPlan<TProject>>()

		return new Map<string, BrowseInstallPlan<TProject>>(
			parsed.filter(isStoredServerInstallQueueEntry),
		)
	} catch {
		return new Map<string, BrowseInstallPlan<TProject>>()
	}
}

export function writeStoredServerInstallQueue<
	TProject extends BrowseInstallProject = BrowseInstallProject,
>(
	serverId: string | null,
	worldId: string | null,
	plans: Map<string, BrowseInstallPlan<TProject>>,
) {
	const key = getStoredServerInstallQueueKey(serverId, worldId)
	if (!key || typeof localStorage === 'undefined') return

	if (plans.size === 0) {
		localStorage.removeItem(key)
		return
	}

	localStorage.setItem(key, JSON.stringify(Array.from(plans.entries())))
}

function getServerInstallQueueLockName(lockKey: string) {
	return `${serverInstallQueueStoragePrefix}:flush:${lockKey}`
}

function getStoredServerInstallQueueLockKey(lockName: string) {
	return `${serverInstallQueueLockStoragePrefix}:${lockName}`
}

function isStoredServerInstallQueueLock(value: unknown): value is StoredServerInstallQueueLock {
	if (!value || typeof value !== 'object') return false
	const record = value as Record<string, unknown>
	return typeof record.token === 'string' && typeof record.expiresAt === 'number'
}

function readStoredServerInstallQueueLock(key: string) {
	try {
		const raw = localStorage.getItem(key)
		if (!raw) return null
		const parsed = JSON.parse(raw)
		return isStoredServerInstallQueueLock(parsed) ? parsed : null
	} catch {
		return null
	}
}

function createServerInstallQueueLockToken() {
	return `${Date.now()}:${Math.random().toString(36).slice(2)}`
}

function tryAcquireStoredServerInstallQueueLock(
	lockName: string,
): AcquiredStoredServerInstallQueueLock | null {
	const key = getStoredServerInstallQueueLockKey(lockName)
	const existingLock = readStoredServerInstallQueueLock(key)
	if (existingLock && existingLock.expiresAt > Date.now()) return null

	const token = createServerInstallQueueLockToken()
	localStorage.setItem(
		key,
		JSON.stringify({
			token,
			expiresAt: Date.now() + serverInstallQueueLockTtl,
		} satisfies StoredServerInstallQueueLock),
	)

	const storedLock = readStoredServerInstallQueueLock(key)
	return storedLock?.token === token ? { key, token } : null
}

function refreshStoredServerInstallQueueLock(lock: AcquiredStoredServerInstallQueueLock) {
	const storedLock = readStoredServerInstallQueueLock(lock.key)
	if (storedLock?.token !== lock.token) return false

	localStorage.setItem(
		lock.key,
		JSON.stringify({
			token: lock.token,
			expiresAt: Date.now() + serverInstallQueueLockTtl,
		} satisfies StoredServerInstallQueueLock),
	)
	return true
}

function releaseStoredServerInstallQueueLock(lock: AcquiredStoredServerInstallQueueLock) {
	const storedLock = readStoredServerInstallQueueLock(lock.key)
	if (storedLock?.token === lock.token) {
		localStorage.removeItem(lock.key)
	}
}

function wait(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms))
}

async function withStoredServerInstallQueueLock<T>(
	lockName: string,
	callback: () => T | Promise<T>,
) {
	if (typeof localStorage === 'undefined') {
		return await callback()
	}

	let lock = tryAcquireStoredServerInstallQueueLock(lockName)
	while (!lock) {
		await wait(100)
		lock = tryAcquireStoredServerInstallQueueLock(lockName)
	}

	const acquiredLock = lock
	const refreshInterval = setInterval(
		() => refreshStoredServerInstallQueueLock(acquiredLock),
		serverInstallQueueLockRefreshInterval,
	)

	try {
		return await callback()
	} finally {
		clearInterval(refreshInterval)
		releaseStoredServerInstallQueueLock(acquiredLock)
	}
}

async function runWithServerInstallQueueLock<T>(lockName: string, callback: () => T | Promise<T>) {
	const locks =
		typeof navigator === 'undefined' ? undefined : (navigator as NavigatorWithLocks).locks

	if (locks) {
		return await locks.request(lockName, { mode: 'exclusive' }, callback)
	}

	return await withStoredServerInstallQueueLock(lockName, callback)
}

export async function withServerInstallQueueLock<T>(
	lockKey: string | null | undefined,
	callback: () => T | Promise<T>,
) {
	if (!lockKey) return await callback()

	const lockName = getServerInstallQueueLockName(lockKey)
	for (;;) {
		const activeFlush = activeInstallQueueFlushes.get(lockName)
		if (!activeFlush) break
		await activeFlush.catch(() => undefined)
	}

	const flush = runWithServerInstallQueueLock(lockName, callback)
	activeInstallQueueFlushes.set(lockName, flush)

	try {
		return await flush
	} finally {
		if (activeInstallQueueFlushes.get(lockName) === flush) {
			activeInstallQueueFlushes.delete(lockName)
		}
	}
}

export async function withStoredServerInstallQueueFlushLock<T>(
	serverId: string | null,
	worldId: string | null,
	callback: () => T | Promise<T>,
) {
	return await withServerInstallQueueLock(
		getStoredServerInstallQueueKey(serverId, worldId),
		callback,
	)
}

interface StoredServerInstallQueueLock {
	token: string
	expiresAt: number
}

interface AcquiredStoredServerInstallQueueLock {
	key: string
	token: string
}

type NavigatorWithLocks = {
	locks?: {
		request: <T>(
			name: string,
			options: { mode: 'exclusive' },
			callback: () => T | Promise<T>,
		) => Promise<T>
	}
}

export function getStoredServerAddonInstallQueue<
	TProject extends BrowseInstallProject = BrowseInstallProject,
>(serverId: string, worldId: string) {
	const storedPlans = readStoredServerInstallQueue<TProject>(serverId, worldId)
	const addonPlans = new Map(
		Array.from(storedPlans).filter(([, plan]) => plan.contentType !== 'modpack'),
	)

	if (addonPlans.size !== storedPlans.size) {
		writeStoredServerInstallQueue(serverId, worldId, addonPlans)
	}

	return addonPlans
}

function isStoredServerInstallQueueEntry(
	value: unknown,
): value is [string, BrowseInstallPlan<BrowseInstallProject>] {
	if (!Array.isArray(value) || value.length !== 2) return false
	const [key, plan] = value
	return typeof key === 'string' && isStoredBrowseInstallPlan(plan)
}

function isStoredBrowseInstallPlan(
	value: unknown,
): value is BrowseInstallPlan<BrowseInstallProject> {
	if (!value || typeof value !== 'object') return false
	const record = value as Record<string, unknown>
	return (
		isStoredBrowseInstallProject(record.project) &&
		typeof record.projectId === 'string' &&
		typeof record.versionId === 'string' &&
		isStoredBrowseInstallContentType(record.contentType) &&
		isStoredBrowseInstallPreferences(record.preferences) &&
		(record.source === 'filtered' || record.source === 'target') &&
		isOptionalStoredResolvedContent(record.resolvedContent)
	)
}

function isOptionalStoredResolvedContent(value: unknown) {
	return (
		value === undefined ||
		(Array.isArray(value) &&
			value.every(
				(item) =>
					!!item &&
					typeof item === 'object' &&
					typeof (item as Record<string, unknown>).projectId === 'string' &&
					typeof (item as Record<string, unknown>).versionId === 'string',
			))
	)
}

function isStoredBrowseInstallProject(value: unknown): value is BrowseInstallProject {
	return (
		!!value &&
		typeof value === 'object' &&
		typeof (value as Record<string, unknown>).project_id === 'string'
	)
}

function isStoredBrowseInstallContentType(value: unknown): value is BrowseInstallContentType {
	return (
		value === 'modpack' ||
		value === 'mod' ||
		value === 'plugin' ||
		value === 'datapack' ||
		value === 'resourcepack' ||
		value === 'shader'
	)
}

function isStoredBrowseInstallPreferences(value: unknown): value is BrowseInstallPreferences {
	if (!value || typeof value !== 'object') return false
	const record = value as Record<string, unknown>
	return isOptionalStringArray(record.gameVersions) && isOptionalStringArray(record.loaders)
}

function isOptionalStringArray(value: unknown) {
	return (
		value === undefined || (Array.isArray(value) && value.every((item) => typeof item === 'string'))
	)
}
