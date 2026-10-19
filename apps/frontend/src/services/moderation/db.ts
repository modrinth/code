const DB_NAME = 'modrinth-moderation'
const DB_VERSION = 3

function hasIndexedDb(): boolean {
	return typeof window !== 'undefined' && typeof indexedDB !== 'undefined'
}

function openDatabase(): Promise<IDBDatabase> {
	return new Promise((resolve, reject) => {
		const request = indexedDB.open(DB_NAME, DB_VERSION)

		request.onupgradeneeded = (event) => {
			const db = request.result
			if (event.oldVersion < 2 && db.objectStoreNames.contains('kv')) {
				db.deleteObjectStore('kv')
			}
			if (db.objectStoreNames.contains('checklist')) {
				db.deleteObjectStore('checklist')
			}
			if (!db.objectStoreNames.contains('queue')) {
				db.createObjectStore('queue')
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

export async function dbGet<T>(store: string, key: IDBValidKey): Promise<T | null> {
	if (!hasIndexedDb()) return null
	const db = await openDatabase()
	try {
		const tx = db.transaction(store, 'readonly')
		const result = await requestToPromise<T | undefined>(tx.objectStore(store).get(key))
		return result ?? null
	} finally {
		db.close()
	}
}

export async function dbPut<T>(store: string, key: IDBValidKey, value: T): Promise<void> {
	if (!hasIndexedDb()) return
	const db = await openDatabase()
	try {
		const tx = db.transaction(store, 'readwrite')
		tx.objectStore(store).put(value, key)
		await new Promise<void>((resolve, reject) => {
			tx.oncomplete = () => resolve()
			tx.onerror = () => reject(tx.error ?? new Error('IndexedDB transaction failed'))
		})
	} finally {
		db.close()
	}
}

export async function dbDelete(store: string, key: IDBValidKey): Promise<void> {
	if (!hasIndexedDb()) return
	const db = await openDatabase()
	try {
		const tx = db.transaction(store, 'readwrite')
		tx.objectStore(store).delete(key)
		await new Promise<void>((resolve, reject) => {
			tx.oncomplete = () => resolve()
			tx.onerror = () => reject(tx.error ?? new Error('IndexedDB transaction failed'))
		})
	} finally {
		db.close()
	}
}
