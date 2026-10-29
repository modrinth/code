import type { CopyrightProjectArchive } from './types'

async function openDatabase(): Promise<IDBDatabase> {
	if (typeof indexedDB === 'undefined') throw new Error('IndexedDB is unavailable in this browser.')
	return new Promise((resolve, reject) => {
		const request = indexedDB.open('modrinth-copyright-archives', 1)
		request.onupgradeneeded = () => request.result.createObjectStore('archives')
		request.onerror = () => reject(request.error ?? new Error('Could not open local storage.'))
		request.onblocked = () =>
			reject(new Error('Local storage is blocked. Close other archive tabs and retry.'))
		request.onsuccess = () => resolve(request.result)
	})
}

export async function loadArchives(): Promise<CopyrightProjectArchive[]> {
	const db = await openDatabase()
	try {
		return await new Promise<CopyrightProjectArchive[]>((resolve, reject) => {
			const request = db.transaction('archives', 'readonly').objectStore('archives').get('saved')
			request.onsuccess = () => resolve(request.result ?? [])
			request.onerror = () => reject(request.error ?? new Error('Could not load archives.'))
		})
	} finally {
		db.close()
	}
}

export async function saveArchives(
	archives: CopyrightProjectArchive[],
	original: CopyrightProjectArchive[],
): Promise<CopyrightProjectArchive[]> {
	const originalById = new Map(original.map((archive) => [archive.id, archive]))
	const archivesById = new Map(archives.map((archive) => [archive.id, archive]))
	const changedIds = [...new Set([...originalById.keys(), ...archivesById.keys()])].filter(
		(id) => JSON.stringify(originalById.get(id)) !== JSON.stringify(archivesById.get(id)),
	)
	const db = await openDatabase()
	try {
		return await new Promise<CopyrightProjectArchive[]>((resolve, reject) => {
			const transaction = db.transaction('archives', 'readwrite')
			const store = transaction.objectStore('archives')
			let merged: CopyrightProjectArchive[] = []
			let conflict: Error | undefined
			transaction.oncomplete = () => resolve(merged)
			transaction.onerror = () => reject(transaction.error ?? new Error('Could not save archives.'))
			transaction.onabort = () =>
				reject(conflict ?? transaction.error ?? new Error('Saving archives was aborted.'))
			const request = store.get('saved')
			request.onsuccess = () => {
				const latest = (request.result ?? []) as CopyrightProjectArchive[]
				const latestById = new Map(latest.map((archive) => [archive.id, archive]))
				for (const id of changedIds) {
					if (JSON.stringify(latestById.get(id)) !== JSON.stringify(originalById.get(id))) {
						conflict = new Error(
							'An archive you changed was updated or deleted in another tab. Copy your edits, reload the page, and try again.',
						)
						transaction.abort()
						return
					}
					const archive = archivesById.get(id)
					if (archive) latestById.set(id, archive)
					else latestById.delete(id)
				}
				merged = [...latestById.values()]
				store.put(merged, 'saved')
			}
		})
	} finally {
		db.close()
	}
}
