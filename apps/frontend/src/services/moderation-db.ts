const DB_NAME = 'modrinth-moderation'
const DB_VERSION = 1

function hasIndexedDb(): boolean {
	return typeof window !== 'undefined' && typeof indexedDB !== 'undefined'
}

function openDatabase(): Promise<IDBDatabase> {
	return new Promise((resolve, reject) => {
		const request = indexedDB.open(DB_NAME, DB_VERSION)

		request.onupgradeneeded = () => {
			const db = request.result
			if (!db.objectStoreNames.contains('kv')) {
				db.createObjectStore('kv')
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

export async function dbScan<T>(store: string): Promise<{ key: IDBValidKey; value: T }[]> {
	if (!hasIndexedDb()) return []
	const db = await openDatabase()
	try {
		const tx = db.transaction(store, 'readonly')
		const s = tx.objectStore(store)
		const [keys, values] = await Promise.all([
			requestToPromise(s.getAllKeys()),
			requestToPromise(s.getAll()),
		])
		return keys.map((key, i) => ({ key, value: values[i] as T }))
	} finally {
		db.close()
	}
}
