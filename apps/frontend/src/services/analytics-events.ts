import type { Labrinth } from '@modrinth/api-client'

// Temporary browser-only mock. Another agent will replace this with the backend analytics events route once it is ready.

type AnalyticsEvent = Labrinth.Analytics.v3.AnalyticsEvent
type AnalyticsEventId = Labrinth.Analytics.v3.AnalyticsEventId
type AnalyticsEventMetricKind = Labrinth.Analytics.v3.AnalyticsEventMetricKind
type AnalyticsEventUpsert = Labrinth.Analytics.v3.AnalyticsEventUpsert

interface PersistedAnalyticsEventsState {
	version: 1
	savedAt: string
	events: AnalyticsEvent[]
}

const DB_NAME = 'modrinth-analytics'
const DB_VERSION = 1
const STORE_NAME = 'kv'
const ANALYTICS_EVENTS_KEY = 'analytics-events:v1'
const metricKinds = new Set<AnalyticsEventMetricKind>(['view', 'downloads', 'revenue', 'playtime'])

export const analyticsEventsQueryKey = ['analytics-events'] as const

const initialAnalyticsEvents: AnalyticsEvent[] = [
	{
		id: 1,
		title: 'Analytics outage',
		announcement_url: null,
		for_metric_kind: null,
		starts: '2026-04-25T00:00:00.000Z',
		ends: '2026-04-27T00:00:00.000Z',
	},
	{
		id: 2,
		title: 'Ad revenue over reported, resulting in a potential spike.',
		announcement_url: 'https://modrinth.com/news',
		for_metric_kind: ['revenue'],
		starts: '2026-05-04T00:00:00.000Z',
		ends: '2026-05-04T00:00:00.000Z',
	},
	{
		id: 3,
		title: 'China CDN ingest outage',
		announcement_url: 'https://modrinth.com/news',
		for_metric_kind: ['downloads'],
		starts: '2026-05-01T00:00:00.000Z',
		ends: '2026-05-07T00:00:00.000Z',
	},
	{
		id: 4,
		title: 'Modrinth App release',
		announcement_url: 'https://modrinth.com/news',
		for_metric_kind: null,
		starts: '2023-08-07T00:00:00.000Z',
		ends: '2023-08-07T00:00:00.000Z',
	},
]

function hasIndexedDb(): boolean {
	return typeof window !== 'undefined' && typeof indexedDB !== 'undefined'
}

function getLocalStorage(): Storage | null {
	if (typeof window === 'undefined') return null

	try {
		return window.localStorage
	} catch {
		return null
	}
}

function openDatabase(): Promise<IDBDatabase> {
	return new Promise((resolve, reject) => {
		const request = indexedDB.open(DB_NAME, DB_VERSION)

		request.onupgradeneeded = () => {
			const db = request.result
			if (!db.objectStoreNames.contains(STORE_NAME)) {
				db.createObjectStore(STORE_NAME)
			}
		}

		request.onsuccess = () => resolve(request.result)
		request.onerror = () => reject(request.error ?? new Error('Failed to open IndexedDB'))
		request.onblocked = () => reject(new Error('IndexedDB open request blocked'))
	})
}

function requestToPromise<T>(request: IDBRequest<T>): Promise<T> {
	return new Promise((resolve, reject) => {
		request.onsuccess = () => resolve(request.result)
		request.onerror = () => reject(request.error ?? new Error('IndexedDB request failed'))
	})
}

function cloneEvents(events: AnalyticsEvent[]): AnalyticsEvent[] {
	return events.map((event) => ({
		...event,
		for_metric_kind: event.for_metric_kind ? [...event.for_metric_kind] : null,
	}))
}

function wrapEvents(events: AnalyticsEvent[]): PersistedAnalyticsEventsState {
	return {
		version: 1,
		savedAt: new Date().toISOString(),
		events: cloneEvents(events),
	}
}

function isMetricKind(value: unknown): value is AnalyticsEventMetricKind {
	return typeof value === 'string' && metricKinds.has(value as AnalyticsEventMetricKind)
}

function isMetricKindArray(value: unknown): value is AnalyticsEventMetricKind[] {
	return Array.isArray(value) && value.every(isMetricKind)
}

function isAnalyticsEvent(value: unknown): value is AnalyticsEvent {
	if (!value || typeof value !== 'object') return false

	const candidate = value as AnalyticsEvent
	if (typeof candidate.id !== 'number' || !Number.isFinite(candidate.id)) return false
	if (typeof candidate.title !== 'string') return false
	if (typeof candidate.starts !== 'string') return false
	if (typeof candidate.ends !== 'string') return false
	if (candidate.announcement_url !== null && typeof candidate.announcement_url !== 'string') {
		return false
	}
	if (candidate.for_metric_kind !== null && !isMetricKindArray(candidate.for_metric_kind)) {
		return false
	}

	return true
}

function isPersistedAnalyticsEventsState(value: unknown): value is PersistedAnalyticsEventsState {
	if (!value || typeof value !== 'object') return false

	const candidate = value as PersistedAnalyticsEventsState
	if (candidate.version !== 1) return false
	if (typeof candidate.savedAt !== 'string') return false
	if (!Array.isArray(candidate.events)) return false

	return candidate.events.every(isAnalyticsEvent)
}

async function loadFromIndexedDb(): Promise<PersistedAnalyticsEventsState | null> {
	if (!hasIndexedDb()) return null

	const db = await openDatabase()
	try {
		const tx = db.transaction(STORE_NAME, 'readonly')
		const store = tx.objectStore(STORE_NAME)
		const raw = await requestToPromise(store.get(ANALYTICS_EVENTS_KEY))
		if (!isPersistedAnalyticsEventsState(raw)) return null

		return raw
	} finally {
		db.close()
	}
}

async function saveToIndexedDb(state: PersistedAnalyticsEventsState): Promise<void> {
	if (!hasIndexedDb()) return

	const db = await openDatabase()
	try {
		const tx = db.transaction(STORE_NAME, 'readwrite')
		tx.objectStore(STORE_NAME).put(state, ANALYTICS_EVENTS_KEY)

		await new Promise<void>((resolve, reject) => {
			tx.oncomplete = () => resolve()
			tx.onerror = () => reject(tx.error ?? new Error('IndexedDB transaction failed'))
		})
	} finally {
		db.close()
	}
}

function loadFromLocalStorage(): PersistedAnalyticsEventsState | null {
	const storage = getLocalStorage()
	if (!storage) return null

	const raw = storage.getItem(ANALYTICS_EVENTS_KEY)
	if (!raw) return null

	try {
		const parsed: unknown = JSON.parse(raw)
		if (isPersistedAnalyticsEventsState(parsed)) return parsed
	} catch (error) {
		console.debug('Failed to parse analytics events from localStorage:', error)
	}

	safeClearLocalStorage()
	return null
}

function saveToLocalStorage(state: PersistedAnalyticsEventsState): void {
	const storage = getLocalStorage()
	if (!storage) return
	storage.setItem(ANALYTICS_EVENTS_KEY, JSON.stringify(state))
}

function clearLocalStorage(): void {
	const storage = getLocalStorage()
	if (!storage) return
	storage.removeItem(ANALYTICS_EVENTS_KEY)
}

function safeClearLocalStorage(): void {
	try {
		clearLocalStorage()
	} catch (error) {
		console.debug('Failed to clear analytics events from localStorage:', error)
	}
}

function safeSaveLocalStorage(state: PersistedAnalyticsEventsState): void {
	try {
		saveToLocalStorage(state)
	} catch (error) {
		console.debug('Failed to save analytics events to localStorage:', error)
	}
}

async function loadAnalyticsEventsState(): Promise<PersistedAnalyticsEventsState | null> {
	if (!import.meta.client) return null

	try {
		const indexedDbState = await loadFromIndexedDb()
		if (indexedDbState) return indexedDbState
	} catch (error) {
		console.debug('Failed to load analytics events from IndexedDB:', error)
	}

	try {
		return loadFromLocalStorage()
	} catch (error) {
		console.debug('Failed to load analytics events from localStorage:', error)
	}

	return null
}

async function saveAnalyticsEventsState(state: PersistedAnalyticsEventsState): Promise<void> {
	if (!import.meta.client) return

	if (hasIndexedDb()) {
		try {
			await saveToIndexedDb(state)
			safeSaveLocalStorage(state)
			return
		} catch (error) {
			console.debug(
				'Failed to save analytics events to IndexedDB, using localStorage fallback:',
				error,
			)
		}
	}

	safeSaveLocalStorage(state)
}

async function loadOrSeedAnalyticsEvents(): Promise<AnalyticsEvent[]> {
	const state = await loadAnalyticsEventsState()
	if (state) return cloneEvents(state.events)

	const seededEvents = cloneEvents(initialAnalyticsEvents)
	await saveAnalyticsEventsState(wrapEvents(seededEvents))
	return seededEvents
}

export async function getAnalyticsEvents(): Promise<AnalyticsEvent[]> {
	if (!import.meta.client) return []

	return loadOrSeedAnalyticsEvents()
}

export async function createAnalyticsEvent(data: AnalyticsEventUpsert): Promise<AnalyticsEvent> {
	const events = await loadOrSeedAnalyticsEvents()
	const nextId = events.reduce((maxId, event) => Math.max(maxId, event.id), 0) + 1
	const event: AnalyticsEvent = {
		...data,
		id: nextId,
		for_metric_kind: data.for_metric_kind ? [...data.for_metric_kind] : null,
	}

	await saveAnalyticsEventsState(wrapEvents([...events, event]))
	return { ...event, for_metric_kind: event.for_metric_kind ? [...event.for_metric_kind] : null }
}

export async function editAnalyticsEvent(
	id: AnalyticsEventId,
	data: AnalyticsEventUpsert,
): Promise<AnalyticsEvent> {
	const events = await loadOrSeedAnalyticsEvents()
	const eventIndex = events.findIndex((event) => event.id === id)
	if (eventIndex === -1) {
		throw new Error('Analytics event not found')
	}

	const event: AnalyticsEvent = {
		...data,
		id,
		for_metric_kind: data.for_metric_kind ? [...data.for_metric_kind] : null,
	}
	const nextEvents = [...events]
	nextEvents[eventIndex] = event

	await saveAnalyticsEventsState(wrapEvents(nextEvents))
	return { ...event, for_metric_kind: event.for_metric_kind ? [...event.for_metric_kind] : null }
}

export async function deleteAnalyticsEvent(id: AnalyticsEventId): Promise<void> {
	const events = await loadOrSeedAnalyticsEvents()
	const nextEvents = events.filter((event) => event.id !== id)
	if (nextEvents.length === events.length) {
		throw new Error('Analytics event not found')
	}

	await saveAnalyticsEventsState(wrapEvents(nextEvents))
}
