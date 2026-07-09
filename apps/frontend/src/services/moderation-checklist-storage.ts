import {
	type ActionState,
	deserializeActionStates,
	serializeActionStates,
} from '@modrinth/moderation'

import { dbDelete, dbGet, dbPut, dbScan } from './moderation-db.ts'

interface PersistedChecklistValue<T> {
	version: 1
	savedAt: string
	value: T
}

export interface ModerationChecklistGeneratedMessageState {
	generated: boolean
	message: string
}

const STORE = 'kv'
const CHECKLIST_OPEN_KEY_PREFIX = 'show-moderation-checklist-'
const STAGE_KEY_PREFIX = 'moderation-stage-'
const ACTION_STATES_KEY_PREFIX = 'moderation-actions-'
const TEXT_INPUTS_KEY_PREFIX = 'moderation-inputs-'
const GENERATED_MESSAGE_KEY_PREFIX = 'moderation-generated-message-'
const CHECKLIST_STATE_MAX_AGE_MS = 30 * 24 * 60 * 60 * 1000
const CHECKLIST_CLEANUP_INTERVAL_MS = 24 * 60 * 60 * 1000
const CHECKLIST_STATE_KEY_PREFIXES = [
	CHECKLIST_OPEN_KEY_PREFIX,
	STAGE_KEY_PREFIX,
	ACTION_STATES_KEY_PREFIX,
	TEXT_INPUTS_KEY_PREFIX,
	GENERATED_MESSAGE_KEY_PREFIX,
]
const saveChains = new Map<string, Promise<void>>()
let checklistCleanupPromise: Promise<void> | null = null
let checklistCleanupLastRunAt = 0

export function createEmptyGeneratedMessageState(): ModerationChecklistGeneratedMessageState {
	return {
		generated: false,
		message: '',
	}
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return !!value && typeof value === 'object'
}

function isPersistedValue<T>(
	value: unknown,
	isValue: (value: unknown) => value is T,
): value is PersistedChecklistValue<T> {
	if (!isRecord(value)) return false
	if (value.version !== 1) return false
	if (typeof value.savedAt !== 'string') return false
	return isValue(value.value)
}

function isBoolean(value: unknown): value is boolean {
	return typeof value === 'boolean'
}

function isNumber(value: unknown): value is number {
	return typeof value === 'number' && Number.isFinite(value)
}

function isString(value: unknown): value is string {
	return typeof value === 'string'
}

function isGeneratedMessageState(
	value: unknown,
): value is ModerationChecklistGeneratedMessageState {
	if (!isRecord(value)) return false
	return typeof value.generated === 'boolean' && typeof value.message === 'string'
}

function sanitizeStage(value: number): number {
	return Math.max(0, Math.trunc(value))
}

function sanitizeTextInputs(value: unknown): Record<string, string> | null {
	if (!isRecord(value)) return null

	const result: Record<string, string> = {}
	for (const [key, entry] of Object.entries(value)) {
		if (typeof entry === 'string') {
			result[key] = entry
		}
	}
	return result
}

function wrapValue<T>(value: T, savedAt = new Date().toISOString()): PersistedChecklistValue<T> {
	return { version: 1, savedAt, value }
}

function normalizeChecklistOpen(value: unknown): PersistedChecklistValue<boolean> | null {
	if (isPersistedValue(value, isBoolean)) return value
	if (isBoolean(value)) return wrapValue(value, '')
	return null
}

function normalizeStage(value: unknown): PersistedChecklistValue<number> | null {
	if (isPersistedValue(value, isNumber)) {
		return { ...value, value: sanitizeStage(value.value) }
	}
	if (isNumber(value)) return wrapValue(sanitizeStage(value), '')
	return null
}

function normalizeActionStates(
	value: unknown,
): PersistedChecklistValue<Record<string, ActionState>> | null {
	if (isRecord(value) && value.version === 1 && typeof value.savedAt === 'string') {
		if (isString(value.value)) {
			return {
				version: 1,
				savedAt: value.savedAt,
				value: deserializeActionStates(value.value),
			}
		}

		if (isRecord(value.value)) {
			return {
				version: 1,
				savedAt: value.savedAt,
				value: deserializeActionStates(JSON.stringify(value.value)),
			}
		}
	}

	if (isString(value)) return wrapValue(deserializeActionStates(value), '')
	if (isRecord(value)) return wrapValue(deserializeActionStates(JSON.stringify(value)), '')
	return null
}

function normalizeTextInputs(
	value: unknown,
): PersistedChecklistValue<Record<string, string>> | null {
	if (isRecord(value) && value.version === 1 && typeof value.savedAt === 'string') {
		const textInputs = sanitizeTextInputs(value.value)
		if (textInputs) {
			return { version: 1, savedAt: value.savedAt, value: textInputs }
		}
	}

	const textInputs = sanitizeTextInputs(value)
	return textInputs ? wrapValue(textInputs, '') : null
}

function normalizeGeneratedMessage(
	value: unknown,
): PersistedChecklistValue<ModerationChecklistGeneratedMessageState> | null {
	if (isPersistedValue(value, isGeneratedMessageState)) return value
	if (isGeneratedMessageState(value)) return wrapValue(value, '')
	return null
}

function savedAtTime<T>(state: PersistedChecklistValue<T>): number {
	const time = Date.parse(state.savedAt)
	return Number.isNaN(time) ? 0 : time
}

function isChecklistStateKey(key: string): boolean {
	return CHECKLIST_STATE_KEY_PREFIXES.some((prefix) => key.startsWith(prefix))
}

function isStaleState<T>(
	state: PersistedChecklistValue<T>,
	now = Date.now(),
	maxAgeMs = CHECKLIST_STATE_MAX_AGE_MS,
): boolean {
	const savedAt = savedAtTime(state)
	if (savedAt === 0) return false
	return now - savedAt > maxAgeMs
}

function isStaleRawState(value: unknown, now = Date.now()): boolean {
	if (!isRecord(value)) return false
	if (value.version !== 1 || typeof value.savedAt !== 'string') return false

	const savedAt = Date.parse(value.savedAt)
	if (Number.isNaN(savedAt)) return false
	return now - savedAt > CHECKLIST_STATE_MAX_AGE_MS
}

async function cleanupIndexedDb(now = Date.now()): Promise<void> {
	const entries = await dbScan<unknown>(STORE)
	const staleKeys = entries
		.filter(
			({ key, value }) =>
				typeof key === 'string' && isChecklistStateKey(key) && isStaleRawState(value, now),
		)
		.map(({ key }) => key as string)
	await Promise.all(staleKeys.map((key) => dbDelete(STORE, key)))
}

function scheduleStaleChecklistCleanup(): void {
	if (!import.meta.client || checklistCleanupPromise) return

	const now = Date.now()
	if (now - checklistCleanupLastRunAt < CHECKLIST_CLEANUP_INTERVAL_MS) return

	checklistCleanupLastRunAt = now

	checklistCleanupPromise = cleanupIndexedDb(now)
		.catch((error) => {
			console.debug('Failed to cleanup stale moderation checklist state from IndexedDB:', error)
		})
		.finally(() => {
			checklistCleanupPromise = null
		})
}

async function saveInOrder<T>(key: string, value: PersistedChecklistValue<T>): Promise<void> {
	const run = () => dbPut(STORE, key, value)
	const result = (saveChains.get(key) ?? Promise.resolve()).then(run, run)
	saveChains.set(
		key,
		result.then(
			() => undefined,
			() => undefined,
		),
	)
	return result
}

async function deleteInOrder(key: string): Promise<void> {
	const run = () => dbDelete(STORE, key)
	const result = (saveChains.get(key) ?? Promise.resolve()).then(run, run)
	saveChains.set(
		key,
		result.then(
			() => undefined,
			() => undefined,
		),
	)
	return result
}

async function loadState<T>(
	key: string,
	normalize: (value: unknown) => PersistedChecklistValue<T> | null,
	touch = true,
): Promise<T | null> {
	if (!import.meta.client) return null

	scheduleStaleChecklistCleanup()

	let state: PersistedChecklistValue<T> | null = null
	try {
		const raw = await dbGet<unknown>(STORE, key)
		state = raw !== null ? normalize(raw) : null
	} catch (error) {
		console.debug('Failed to load moderation checklist state from IndexedDB:', error)
	}

	if (!state) return null

	if (isStaleState(state)) {
		await clearState(key)
		return null
	}

	if (touch) {
		void saveState(key, state.value)
	}

	return state.value
}

async function saveState<T>(key: string, value: T): Promise<void> {
	if (!import.meta.client) return

	scheduleStaleChecklistCleanup()

	try {
		await saveInOrder(key, wrapValue(value))
	} catch (error) {
		console.debug('Failed to save moderation checklist state to IndexedDB:', error)
	}
}

async function clearState(key: string): Promise<void> {
	if (!import.meta.client) return

	try {
		await deleteInOrder(key)
	} catch (error) {
		console.debug('Failed to clear moderation checklist state from IndexedDB:', error)
	}
}

export async function loadChecklistOpenState(projectId: string): Promise<boolean | null> {
	return loadState(`${CHECKLIST_OPEN_KEY_PREFIX}${projectId}`, normalizeChecklistOpen, false)
}

export async function saveChecklistOpenState(projectId: string, open: boolean): Promise<void> {
	await saveState(`${CHECKLIST_OPEN_KEY_PREFIX}${projectId}`, open)
}

export async function loadChecklistStage(projectSlug: string): Promise<number | null> {
	return loadState(`${STAGE_KEY_PREFIX}${projectSlug}`, normalizeStage)
}

export async function saveChecklistStage(projectSlug: string, stage: number): Promise<void> {
	await saveState(`${STAGE_KEY_PREFIX}${projectSlug}`, sanitizeStage(stage))
}

export async function loadChecklistActionStates(
	projectSlug: string,
): Promise<Record<string, ActionState>> {
	const actionStates =
		(await loadState(`${ACTION_STATES_KEY_PREFIX}${projectSlug}`, normalizeActionStates, false)) ??
		{}
	if (Object.keys(actionStates).length > 0) {
		void saveChecklistActionStates(projectSlug, actionStates)
	}
	return actionStates
}

export async function saveChecklistActionStates(
	projectSlug: string,
	actionStates: Record<string, ActionState>,
): Promise<void> {
	await saveState(`${ACTION_STATES_KEY_PREFIX}${projectSlug}`, serializeActionStates(actionStates))
}

export async function loadChecklistTextInputs(
	projectSlug: string,
): Promise<Record<string, string>> {
	return (await loadState(`${TEXT_INPUTS_KEY_PREFIX}${projectSlug}`, normalizeTextInputs)) ?? {}
}

export async function saveChecklistTextInputs(
	projectSlug: string,
	textInputs: Record<string, string>,
): Promise<void> {
	await saveState(`${TEXT_INPUTS_KEY_PREFIX}${projectSlug}`, textInputs)
}

export async function clearChecklistProgressState(projectSlug: string): Promise<void> {
	await Promise.all([
		clearState(`${STAGE_KEY_PREFIX}${projectSlug}`),
		clearState(`${ACTION_STATES_KEY_PREFIX}${projectSlug}`),
		clearState(`${TEXT_INPUTS_KEY_PREFIX}${projectSlug}`),
	])
}

export async function loadGeneratedMessageState(
	projectSlug: string,
): Promise<ModerationChecklistGeneratedMessageState> {
	return (
		(await loadState(`${GENERATED_MESSAGE_KEY_PREFIX}${projectSlug}`, normalizeGeneratedMessage)) ??
		createEmptyGeneratedMessageState()
	)
}

export async function saveGeneratedMessageState(
	projectSlug: string,
	state: ModerationChecklistGeneratedMessageState,
): Promise<void> {
	await saveState(`${GENERATED_MESSAGE_KEY_PREFIX}${projectSlug}`, state)
}

export async function clearGeneratedMessageState(projectSlug: string): Promise<void> {
	await clearState(`${GENERATED_MESSAGE_KEY_PREFIX}${projectSlug}`)
}
