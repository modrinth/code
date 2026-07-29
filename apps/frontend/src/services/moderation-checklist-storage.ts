import type { NodeState } from '@modrinth/moderation'

import { dbDelete, dbGet, dbPut, dbScan } from './moderation-db.ts'

export interface PersistedChecklistState {
	savedAt: string
	open?: boolean
	reviewAnyway?: boolean
	stage?: string
	message?: string
	state?: Record<string, Record<string, NodeState>>
}

const STORE = 'checklist'
const CHECKLIST_STATE_MAX_AGE_MS = 30 * 24 * 60 * 60 * 1000
const CHECKLIST_CLEANUP_INTERVAL_MS = 24 * 60 * 60 * 1000
const saveChain = new Map<string, Promise<void>>()
let checklistCleanupPromise: Promise<void> | null = null
let checklistCleanupLastRunAt = 0

function isPersistedChecklistState(value: unknown): value is PersistedChecklistState {
	if (!value || typeof value !== 'object') return false
	const v = value as PersistedChecklistState
	if (typeof v.savedAt !== 'string') return false
	if (v.stage !== undefined && typeof v.stage !== 'string') return false
	if (v.message !== undefined && typeof v.message !== 'string') return false
	if (v.state !== undefined && typeof v.state !== 'object') return false
	return true
}

function isStale(savedAt: string, now = Date.now()): boolean {
	const time = Date.parse(savedAt)
	return !Number.isNaN(time) && now - time > CHECKLIST_STATE_MAX_AGE_MS
}

async function cleanupStaleStates(now = Date.now()): Promise<void> {
	const entries = await dbScan<unknown>(STORE)
	const staleKeys = entries
		.filter(({ value }) => {
			if (!isPersistedChecklistState(value)) return false
			return isStale(value.savedAt, now)
		})
		.map(({ key }) => key)
	await Promise.all(staleKeys.map((key) => dbDelete(STORE, key)))
}

function scheduleStaleChecklistCleanup(): void {
	if (!import.meta.client || checklistCleanupPromise) return

	const now = Date.now()
	if (now - checklistCleanupLastRunAt < CHECKLIST_CLEANUP_INTERVAL_MS) return

	checklistCleanupLastRunAt = now
	checklistCleanupPromise = cleanupStaleStates(now)
		.catch((error) => {
			console.debug('Failed to cleanup stale moderation checklist states from IndexedDB:', error)
		})
		.finally(() => {
			checklistCleanupPromise = null
		})
}

async function enqueueOp(projectId: string, op: () => Promise<void>): Promise<void> {
	const result = (saveChain.get(projectId) ?? Promise.resolve()).then(op, op)
	saveChain.set(
		projectId,
		result.then(
			() => undefined,
			() => undefined,
		),
	)
	return result
}

export async function loadChecklistState(
	projectId: string,
): Promise<PersistedChecklistState | null> {
	if (!import.meta.client) return null
	scheduleStaleChecklistCleanup()

	try {
		const raw = await dbGet<unknown>(STORE, projectId)
		if (!isPersistedChecklistState(raw)) return null
		if (isStale(raw.savedAt)) {
			await clearChecklistState(projectId)
			return null
		}
		return raw
	} catch (error) {
		console.debug('Failed to load checklist state from IndexedDB:', error)
		return null
	}
}

export async function saveChecklistState(
	projectId: string,
	state: Omit<PersistedChecklistState, 'savedAt'>,
): Promise<void> {
	if (!import.meta.client) return
	scheduleStaleChecklistCleanup()

	const record: PersistedChecklistState = { ...state, savedAt: new Date().toISOString() }
	try {
		await enqueueOp(projectId, () => dbPut(STORE, projectId, record))
	} catch (error) {
		console.debug('Failed to save checklist state to IndexedDB:', error)
	}
}

export async function clearChecklistState(projectId: string): Promise<void> {
	if (!import.meta.client) return

	try {
		await enqueueOp(projectId, () => dbDelete(STORE, projectId))
	} catch (error) {
		console.debug('Failed to clear checklist state from IndexedDB:', error)
	}
}
