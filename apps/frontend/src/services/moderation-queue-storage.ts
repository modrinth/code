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

const DB_NAME = 'modrinth-moderation'
const DB_VERSION = 1
const STORE_NAME = 'kv'
export const MODERATION_QUEUE_KEY = 'moderation-queue:v1'

function hasIndexedDb(): boolean {
	return typeof window !== 'undefined' && typeof indexedDB !== 'undefined'
}

function hasLocalStorage(): boolean {
	return typeof window !== 'undefined' && typeof window.localStorage !== 'undefined'
}

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

async function loadFromIndexedDb(): Promise<PersistedModerationQueueState | null> {
	if (!hasIndexedDb()) return null

	const db = await openDatabase()
	try {
		const tx = db.transaction(STORE_NAME, 'readonly')
		const store = tx.objectStore(STORE_NAME)
		const raw = await requestToPromise(store.get(MODERATION_QUEUE_KEY))
		if (!isPersistedStateCandidate(raw)) return null

		return raw
	} finally {
		db.close()
	}
}

async function saveToIndexedDb(state: PersistedModerationQueueState): Promise<void> {
	if (!hasIndexedDb()) return

	const db = await openDatabase()
	try {
		const tx = db.transaction(STORE_NAME, 'readwrite')
		tx.objectStore(STORE_NAME).put(state, MODERATION_QUEUE_KEY)

		await new Promise<void>((resolve, reject) => {
			tx.oncomplete = () => resolve()
			tx.onerror = () => reject(tx.error ?? new Error('IndexedDB transaction failed'))
		})
	} finally {
		db.close()
	}
}

async function clearIndexedDb(): Promise<void> {
	if (!hasIndexedDb()) return

	const db = await openDatabase()
	try {
		const tx = db.transaction(STORE_NAME, 'readwrite')
		tx.objectStore(STORE_NAME).delete(MODERATION_QUEUE_KEY)

		await new Promise<void>((resolve, reject) => {
			tx.oncomplete = () => resolve()
			tx.onerror = () => reject(tx.error ?? new Error('IndexedDB transaction failed'))
		})
	} finally {
		db.close()
	}
}

function loadFromLocalStorage(): PersistedModerationQueueState | null {
	if (!hasLocalStorage()) return null

	const raw = window.localStorage.getItem(MODERATION_QUEUE_KEY)
	if (!raw) return null

	const parsed: unknown = JSON.parse(raw)
	return isPersistedStateCandidate(parsed) ? parsed : null
}

function saveToLocalStorage(state: PersistedModerationQueueState): void {
	if (!hasLocalStorage()) return
	window.localStorage.setItem(MODERATION_QUEUE_KEY, JSON.stringify(state))
}

function clearLocalStorage(): void {
	if (!hasLocalStorage()) return
	window.localStorage.removeItem(MODERATION_QUEUE_KEY)
}

export async function loadQueueState(): Promise<PersistedModerationQueueState | null> {
	if (!import.meta.client) return null

	try {
		const state = await loadFromIndexedDb()
		if (state) return state
	} catch (error) {
		console.debug('Failed to load moderation queue from IndexedDB:', error)
	}

	return loadFromLocalStorage()
}

export async function saveQueueState(state: PersistedModerationQueueState): Promise<void> {
	if (!import.meta.client) return

	if (hasIndexedDb()) {
		try {
			await saveToIndexedDb(state)
			return
		} catch (error) {
			console.debug('Failed to save moderation queue to IndexedDB, using localStorage fallback:', error)
		}
	}

	saveToLocalStorage(state)
}

export async function clearQueueState(): Promise<void> {
	if (!import.meta.client) return

	if (hasIndexedDb()) {
		try {
			await clearIndexedDb()
			return
		} catch (error) {
			console.debug('Failed to clear moderation queue from IndexedDB:', error)
		}
	}

	clearLocalStorage()
}
