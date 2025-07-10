interface StoredHead {
  blob: Blob
  timestamp: number
}

export class HeadStorage {
  private dbName = 'head-storage'
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
        if (!db.objectStoreNames.contains('heads')) {
          db.createObjectStore('heads')
        }
      }
    })
  }

  async store(key: string, blob: Blob): Promise<void> {
    if (!this.db) await this.init()

    const transaction = this.db!.transaction(['heads'], 'readwrite')
    const store = transaction.objectStore('heads')

    const storedHead: StoredHead = {
      blob,
      timestamp: Date.now(),
    }

    return new Promise((resolve, reject) => {
      const request = store.put(storedHead, key)

      request.onsuccess = () => resolve()
      request.onerror = () => reject(request.error)
    })
  }

  async retrieve(key: string): Promise<string | null> {
    if (!this.db) await this.init()

    const transaction = this.db!.transaction(['heads'], 'readonly')
    const store = transaction.objectStore('heads')

    return new Promise((resolve, reject) => {
      const request = store.get(key)

      request.onsuccess = () => {
        const result = request.result as StoredHead | undefined

        if (!result) {
          resolve(null)
          return
        }

        const url = URL.createObjectURL(result.blob)
        resolve(url)
      }
      request.onerror = () => reject(request.error)
    })
  }

  async batchRetrieve(keys: string[]): Promise<Record<string, Blob | null>> {
    if (!this.db) await this.init()

    const transaction = this.db!.transaction(['heads'], 'readonly')
    const store = transaction.objectStore('heads')
    const results: Record<string, Blob | null> = {}

    return new Promise((resolve, _reject) => {
      let completedRequests = 0

      if (keys.length === 0) {
        resolve(results)
        return
      }

      for (const key of keys) {
        const request = store.get(key)

        request.onsuccess = () => {
          const result = request.result as StoredHead | undefined

          if (result) {
            results[key] = result.blob
          } else {
            results[key] = null
          }

          completedRequests++
          if (completedRequests === keys.length) {
            resolve(results)
          }
        }

        request.onerror = () => {
          results[key] = null
          completedRequests++
          if (completedRequests === keys.length) {
            resolve(results)
          }
        }
      }
    })
  }

  async cleanupInvalidKeys(validKeys: Set<string>): Promise<number> {
    if (!this.db) await this.init()

    const transaction = this.db!.transaction(['heads'], 'readwrite')
    const store = transaction.objectStore('heads')
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
              console.warn('Failed to delete invalid head entry:', key)
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

  async debugCalculateStorage(): Promise<void> {
    if (!this.db) await this.init()

    const transaction = this.db!.transaction(['heads'], 'readonly')
    const store = transaction.objectStore('heads')

    let totalSize = 0
    let count = 0
    const entries: Array<{ key: string; size: number }> = []

    return new Promise((resolve, reject) => {
      const request = store.openCursor()

      request.onsuccess = (event) => {
        const cursor = (event.target as IDBRequest<IDBCursorWithValue>).result

        if (cursor) {
          const key = cursor.primaryKey as string
          const value = cursor.value as StoredHead

          const entrySize = value.blob.size
          totalSize += entrySize
          count++

          entries.push({
            key,
            size: entrySize,
          })

          cursor.continue()
        } else {
          console.group('🗄️ Head Storage Debug Info')
          console.log(`Total entries: ${count}`)
          console.log(`Total size: ${(totalSize / 1024 / 1024).toFixed(2)} MB`)
          console.log(
            `Average size per entry: ${count > 0 ? (totalSize / count / 1024).toFixed(2) : 0} KB`,
          )

          if (entries.length > 0) {
            const sortedEntries = entries.sort((a, b) => b.size - a.size)
            console.log(
              'Largest entry:',
              sortedEntries[0].key,
              '(' + (sortedEntries[0].size / 1024).toFixed(2) + ' KB)',
            )
            console.log(
              'Smallest entry:',
              sortedEntries[sortedEntries.length - 1].key,
              '(' + (sortedEntries[sortedEntries.length - 1].size / 1024).toFixed(2) + ' KB)',
            )
          }

          console.groupEnd()
          resolve()
        }
      }

      request.onerror = () => reject(request.error)
    })
  }

  async clearAll(): Promise<void> {
    if (!this.db) await this.init()

    const transaction = this.db!.transaction(['heads'], 'readwrite')
    const store = transaction.objectStore('heads')

    return new Promise((resolve, reject) => {
      const request = store.clear()

      request.onsuccess = () => resolve()
      request.onerror = () => reject(request.error)
    })
  }
}

export const headStorage = new HeadStorage()
