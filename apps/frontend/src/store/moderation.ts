import { createPinia, defineStore } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'

export interface ModerationQueue {
	items: string[]
	total: number
	completed: number
	skipped: number
	lastUpdated: Date
}

export interface LockedByUser {
	id: string
	username: string
	avatar_url?: string
}

export interface LockStatusResponse {
	locked: boolean
	locked_by?: LockedByUser
	locked_at?: string
	expired?: boolean
}

export interface LockAcquireResponse {
	success: boolean
	locked_by?: LockedByUser
	locked_at?: string
	expired?: boolean
}

const EMPTY_QUEUE: Partial<ModerationQueue> = {
	items: [],

	// TODO: Consider some form of displaying this in the checklist, maybe at the end
	total: 0,
	completed: 0,
	skipped: 0,
}

function createEmptyQueue(): ModerationQueue {
	return { ...EMPTY_QUEUE, lastUpdated: new Date() } as ModerationQueue
}

const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)

export const useModerationStore = defineStore('moderation', {
	state: () => ({
		currentQueue: createEmptyQueue(),
		currentLock: null as { projectId: string; lockedAt: Date } | null,
		isQueueMode: false,
	}),

	getters: {
		queueLength: (state) => state.currentQueue.items.length,
		hasItems: (state) => state.currentQueue.items.length > 0,
		progress: (state) => {
			if (state.currentQueue.total === 0) return 0
			return (state.currentQueue.completed + state.currentQueue.skipped) / state.currentQueue.total
		},
	},

	actions: {
		setQueue(projectIDs: string[]) {
			this.isQueueMode = true
			this.currentQueue = {
				items: [...projectIDs],
				total: projectIDs.length,
				completed: 0,
				skipped: 0,
				lastUpdated: new Date(),
			}
		},

		setSingleProject(projectId: string) {
			this.isQueueMode = false
			this.currentQueue = {
				items: [projectId],
				total: 1,
				completed: 0,
				skipped: 0,
				lastUpdated: new Date(),
			}
		},

		completeCurrentProject(projectId: string, status: 'completed' | 'skipped' = 'completed') {
			if (status === 'completed') {
				this.currentQueue.completed++
			} else {
				this.currentQueue.skipped++
			}

			this.currentQueue.items = this.currentQueue.items.filter((id: string) => id !== projectId)
			this.currentQueue.lastUpdated = new Date()

			return this.currentQueue.items.length > 0
		},

		getCurrentProjectId(): string | null {
			return this.currentQueue.items[0] || null
		},

		resetQueue() {
			this.isQueueMode = false
			this.currentQueue = createEmptyQueue()
		},

		async acquireLock(projectId: string): Promise<LockAcquireResponse> {
			try {
				const response = (await useBaseFetch(`moderation/lock/${projectId}`, {
					method: 'POST',
					internal: true,
				})) as LockAcquireResponse

				if (response.success) {
					this.currentLock = { projectId, lockedAt: new Date() }
				}

				return response
			} catch (error) {
				console.error('Failed to acquire moderation lock:', error)
				// Return a failed response so the UI can handle it gracefully
				return { success: false }
			}
		},

		async releaseLock(projectId: string): Promise<boolean> {
			try {
				const response = (await useBaseFetch(`moderation/lock/${projectId}`, {
					method: 'DELETE',
					internal: true,
				})) as { success: boolean }

				if (this.currentLock?.projectId === projectId) {
					this.currentLock = null
				}

				return response.success
			} catch {
				return false
			}
		},

		async checkLock(projectId: string): Promise<LockStatusResponse> {
			try {
				const response = (await useBaseFetch(`moderation/lock/${projectId}`, {
					method: 'GET',
					internal: true,
				})) as LockStatusResponse
				return response
			} catch (error) {
				console.error('Failed to check moderation lock:', error)
				// Return unlocked status on error so moderation can proceed
				return { locked: false }
			}
		},

		async refreshLock(): Promise<boolean> {
			if (!this.currentLock) return false

			try {
				const response = await this.acquireLock(this.currentLock.projectId)
				return response.success
			} catch (error) {
				console.error('Failed to refresh moderation lock:', error)
				return false
			}
		},
	},

	persist: {
		key: 'moderation-store',
		serializer: {
			serialize: JSON.stringify,
			deserialize: (value: string) => {
				const parsed = JSON.parse(value)
				if (parsed.currentQueue?.lastUpdated) {
					parsed.currentQueue.lastUpdated = new Date(parsed.currentQueue.lastUpdated)
				}
				return parsed
			},
		},
	},
})
