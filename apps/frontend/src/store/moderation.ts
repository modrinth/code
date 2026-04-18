import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

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
	is_own_lock: boolean
	locked_by?: LockedByUser
	locked_at?: string
	expires_at?: string
	expired?: boolean
}

export interface LockAcquireResponse {
	success: boolean
	is_own_lock: boolean
	locked_by?: LockedByUser
	locked_at?: string
	expires_at?: string
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

export const useModerationStore = defineStore(
	'moderation',
	() => {
		const currentQueue = ref<ModerationQueue>(createEmptyQueue())
		const currentLock = ref<{ projectId: string; lockedAt: Date } | null>(null)
		const isQueueMode = ref(false)

		const queueLength = computed(() => currentQueue.value.items.length)
		const hasItems = computed(() => currentQueue.value.items.length > 0)
		const progress = computed(() => {
			if (currentQueue.value.total === 0) return 0
			return (currentQueue.value.completed + currentQueue.value.skipped) / currentQueue.value.total
		})

		function setQueue(projectIDs: string[]) {
			isQueueMode.value = true
			currentQueue.value = {
				items: [...projectIDs],
				total: projectIDs.length,
				completed: 0,
				skipped: 0,
				lastUpdated: new Date(),
			}
		}

		function setSingleProject(projectId: string) {
			isQueueMode.value = false
			currentQueue.value = {
				items: [projectId],
				total: 1,
				completed: 0,
				skipped: 0,
				lastUpdated: new Date(),
			}
		}

		function completeCurrentProject(
			projectId: string,
			status: 'completed' | 'skipped' = 'completed',
		) {
			if (status === 'completed') {
				currentQueue.value.completed++
			} else {
				currentQueue.value.skipped++
			}

			currentQueue.value.items = currentQueue.value.items.filter((id: string) => id !== projectId)
			currentQueue.value.lastUpdated = new Date()

			return currentQueue.value.items.length > 0
		}

		function getCurrentProjectId(): string | null {
			return currentQueue.value.items[0] || null
		}

		function resetQueue() {
			isQueueMode.value = false
			currentQueue.value = createEmptyQueue()
		}

		async function acquireLock(projectId: string): Promise<LockAcquireResponse> {
			try {
				const response = (await useBaseFetch(`moderation/lock/${projectId}`, {
					method: 'POST',
					internal: true,
				})) as LockAcquireResponse

				if (response.success) {
					currentLock.value = { projectId, lockedAt: new Date() }
				} else if (currentLock.value?.projectId === projectId) {
					// We were outbid or our lock expired — clear stale state
					currentLock.value = null
				}

				return response
			} catch (error) {
				console.error('Failed to acquire moderation lock:', error)
				return { success: false, is_own_lock: false }
			}
		}

		async function overrideLock(projectId: string): Promise<LockAcquireResponse> {
			try {
				const response = (await useBaseFetch(`moderation/lock/${projectId}/override`, {
					method: 'POST',
					internal: true,
				})) as LockAcquireResponse

				if (response.success) {
					currentLock.value = { projectId, lockedAt: new Date() }
				} else if (currentLock.value?.projectId === projectId) {
					currentLock.value = null
				}

				return response
			} catch (error) {
				console.error('Failed to override moderation lock:', error)
				return { success: false, is_own_lock: false }
			}
		}

		async function releaseLock(projectId: string): Promise<boolean> {
			try {
				const response = (await useBaseFetch(`moderation/lock/${projectId}`, {
					method: 'DELETE',
					internal: true,
				})) as { success: boolean }

				if (currentLock.value?.projectId === projectId) {
					currentLock.value = null
				}

				return response.success
			} catch {
				return false
			}
		}

		async function checkLock(projectId: string): Promise<LockStatusResponse> {
			try {
				const response = (await useBaseFetch(`moderation/lock/${projectId}`, {
					method: 'GET',
					internal: true,
				})) as LockStatusResponse
				return response
			} catch (error) {
				console.error('Failed to check moderation lock:', error)
				// Return unlocked status on error so moderation can proceed
				return { locked: false, is_own_lock: false }
			}
		}

		async function refreshLock(): Promise<LockAcquireResponse> {
			if (!currentLock.value) return { success: false, is_own_lock: false }

			try {
				const response = await acquireLock(currentLock.value.projectId)
				// acquireLock already clears currentLock on failure
				return response
			} catch (error) {
				console.error('Failed to refresh moderation lock:', error)
				currentLock.value = null
				return { success: false, is_own_lock: false }
			}
		}

		return {
			currentQueue,
			currentLock,
			isQueueMode,
			queueLength,
			hasItems,
			progress,
			setQueue,
			setSingleProject,
			completeCurrentProject,
			getCurrentProjectId,
			resetQueue,
			acquireLock,
			overrideLock,
			releaseLock,
			checkLock,
			refreshLock,
		}
	},
	{
		persist: {
			key: 'moderation-store',
			// Only persist queue state — currentLock is always revalidated on mount
			paths: ['currentQueue', 'isQueueMode'],
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
	},
)
