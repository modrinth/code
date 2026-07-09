import { dbDelete, dbGet, dbPut } from './moderation-db.ts'

export interface PersistedModerationQueueState {
	version: 1
	savedAt: string
	currentQueue: {
		items: string[]
		total: number
		completed: number
		skipped: number
		lastUpdated: string
	}
	isQueueMode: boolean
}

const STORE = 'kv'
export const MODERATION_QUEUE_KEY = 'moderation-queue:v1'

function isStringArray(value: unknown): value is string[] {
	return Array.isArray(value) && value.every((entry) => typeof entry === 'string')
}

function isPersistedStateCandidate(value: unknown): value is PersistedModerationQueueState {
	if (!value || typeof value !== 'object') return false

	const candidate = value as PersistedModerationQueueState
	if (candidate.version !== 1) return false
	if (typeof candidate.savedAt !== 'string') return false
	if (typeof candidate.isQueueMode !== 'boolean') return false

	const queue = candidate.currentQueue
	if (!queue || typeof queue !== 'object') return false
	if (!isStringArray(queue.items)) return false
	if (typeof queue.total !== 'number' || Number.isNaN(queue.total)) return false
	if (typeof queue.completed !== 'number' || Number.isNaN(queue.completed)) return false
	if (typeof queue.skipped !== 'number' || Number.isNaN(queue.skipped)) return false
	if (typeof queue.lastUpdated !== 'string') return false

	return true
}

export async function loadQueueState(): Promise<PersistedModerationQueueState | null> {
	if (!import.meta.client) return null

	try {
		const raw = await dbGet<unknown>(STORE, MODERATION_QUEUE_KEY)
		if (!isPersistedStateCandidate(raw)) return null
		return raw
	} catch (error) {
		console.debug('Failed to load moderation queue from IndexedDB:', error)
		return null
	}
}

export async function saveQueueState(state: PersistedModerationQueueState): Promise<void> {
	if (!import.meta.client) return

	try {
		await dbPut(STORE, MODERATION_QUEUE_KEY, state)
	} catch (error) {
		console.debug('Failed to save moderation queue to IndexedDB:', error)
	}
}

export async function clearQueueState(): Promise<void> {
	if (!import.meta.client) return

	try {
		await dbDelete(STORE, MODERATION_QUEUE_KEY)
	} catch (error) {
		console.debug('Failed to clear moderation queue from IndexedDB:', error)
	}
}
