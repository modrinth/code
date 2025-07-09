import type { RenderResult } from '../rendering/batch-skin-renderer'

interface StoredPreview {
  forwards: Blob
  backwards: Blob
  timestamp: number
}

export class SkinPreviewStorage {
  private dbName = 'skin-previews'
  private version = 1
  private db: IDBDatabase | null = null

  async init(): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(this.dbName, this.version)

      request.onerror = () => reject(request.error)
      request.onsuccess = () => {
        this.db = request.result
        resolve()
      }

      request.onupgradeneeded = () => {
        const db = request.result
        if (!db.objectStoreNames.contains('previews')) {
          db.createObjectStore('previews')
        }
      }
    })
  }

  async store(key: string, result: RenderResult): Promise<void> {
    if (!this.db) await this.init()

    const forwardsBlob = await fetch(result.forwards).then((r) => r.blob())
    const backwardsBlob = await fetch(result.backwards).then((r) => r.blob())

    const transaction = this.db!.transaction(['previews'], 'readwrite')
    const store = transaction.objectStore('previews')

    const storedPreview: StoredPreview = {
      forwards: forwardsBlob,
      backwards: backwardsBlob,
      timestamp: Date.now(),
    }

    return new Promise((resolve, reject) => {
      const request = store.put(storedPreview, key)

      request.onsuccess = () => resolve()
      request.onerror = () => reject(request.error)
    })
  }

  async retrieve(key: string): Promise<RenderResult | null> {
    if (!this.db) await this.init()

    const transaction = this.db!.transaction(['previews'], 'readonly')
    const store = transaction.objectStore('previews')

    return new Promise((resolve, reject) => {
      const request = store.get(key)

      request.onsuccess = () => {
        const result = request.result as StoredPreview | undefined

        if (!result) {
          resolve(null)
          return
        }

        const forwards = URL.createObjectURL(result.forwards)
        const backwards = URL.createObjectURL(result.backwards)
        resolve({ forwards, backwards })
      }
      request.onerror = () => reject(request.error)
    })
  }

  async cleanupInvalidKeys(validKeys: Set<string>): Promise<number> {
    if (!this.db) await this.init()

    const transaction = this.db!.transaction(['previews'], 'readwrite')
    const store = transaction.objectStore('previews')
    let deletedCount = 0

    return new Promise((resolve, reject) => {
      const request = store.openCursor()

      request.onsuccess = (event) => {
        const cursor = (event.target as IDBRequest<IDBCursorWithValue>).result

        if (cursor) {
          const key = cursor.primaryKey as string

          if (!validKeys.has(key)) {
            const deleteRequest = cursor.delete()
            deleteRequest.onsuccess = () => {
              deletedCount++
            }
            deleteRequest.onerror = () => {
              console.warn('Failed to delete invalid entry:', key)
            }
          }

          cursor.continue()
        } else {
          resolve(deletedCount)
        }
      }

      request.onerror = () => reject(request.error)
    })
  }
}

export const skinPreviewStorage = new SkinPreviewStorage()
