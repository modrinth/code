import { computed, proxyRefs, ref } from 'vue'

import {
	loadQueueState,
	type PersistedModerationQueueState,
	saveQueueState,
} from './moderation-queue-storage.ts'

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

export interface ModerationQueueService {
	currentQueue: ModerationQueue
	currentLock: { projectId: string; lockedAt: Date } | null
	isQueueMode: boolean
	hydrated: boolean
	ready: Promise<void>

	queueLength: number
	hasItems: boolean
	progress: number

	setQueue(projectIds: string[]): Promise<void>
	setSingleProject(projectId: string): Promise<void>
	completeCurrentProject(projectId: string, status?: 'completed' | 'skipped'): Promise<boolean>
	getCurrentProjectId(): string | null
	resetQueue(): Promise<void>

	acquireLock(projectId: string): Promise<LockAcquireResponse>
	overrideLock(projectId: string): Promise<LockAcquireResponse>
	releaseLock(projectId: string): Promise<boolean>
	checkLock(projectId: string): Promise<LockStatusResponse>
	refreshLock(): Promise<LockAcquireResponse>
}

const EMPTY_QUEUE: ModerationQueue = {
	items: [],
	total: 0,
	completed: 0,
	skipped: 0,
	lastUpdated: new Date(),
}

function createEmptyQueue(): ModerationQueue {
	return { ...EMPTY_QUEUE, lastUpdated: new Date(), items: [] }
}

function sanitizeQueue(raw: PersistedModerationQueueState['currentQueue']): ModerationQueue {
	const lastUpdated = new Date(raw.lastUpdated)
	const items = raw.items.filter((id): id is string => typeof id === 'string')
	const completed = Number.isFinite(raw.completed) ? Math.max(Math.trunc(raw.completed), 0) : 0
	const skipped = Number.isFinite(raw.skipped) ? Math.max(Math.trunc(raw.skipped), 0) : 0
	const minimumTotal = items.length + completed + skipped
	const total = Number.isFinite(raw.total) ? Math.max(Math.trunc(raw.total), minimumTotal) : minimumTotal

	return {
		items,
		total,
		completed,
		skipped,
		lastUpdated: Number.isNaN(lastUpdated.getTime()) ? new Date() : lastUpdated,
	}
}

function persistedPayload(
	queue: ModerationQueue,
	isQueueMode: boolean,
): PersistedModerationQueueState {
	return {
		version: 1,
		savedAt: new Date().toISOString(),
		currentQueue: {
			items: [...queue.items],
			total: queue.total,
			completed: queue.completed,
			skipped: queue.skipped,
			lastUpdated: queue.lastUpdated.toISOString(),
		},
		isQueueMode,
	}
}

function createModerationQueueState() {
	const currentQueue = ref(createEmptyQueue())
	const currentLock = ref<{ projectId: string; lockedAt: Date } | null>(null)
	const isQueueMode = ref(false)
	const hydrated = ref(false)

	const queueLength = computed(() => currentQueue.value.items.length)
	const hasItems = computed(() => currentQueue.value.items.length > 0)
	const progress = computed(() => {
		if (currentQueue.value.total === 0) return 0
		return (currentQueue.value.completed + currentQueue.value.skipped) / currentQueue.value.total
	})
	let mutationChain = Promise.resolve()

	const ready = (async () => {
		if (import.meta.server) {
			hydrated.value = true
			return
		}

		try {
			const persisted = await loadQueueState()
			if (persisted?.currentQueue) {
				currentQueue.value = sanitizeQueue(persisted.currentQueue)
				isQueueMode.value = persisted.isQueueMode
			}
		} catch {
			currentQueue.value = createEmptyQueue()
			isQueueMode.value = false
		} finally {
			hydrated.value = true
		}
	})()

	async function persist(): Promise<void> {
		if (import.meta.server) return
		await saveQueueState(persistedPayload(currentQueue.value, isQueueMode.value))
	}

	async function withMutation<T>(callback: () => T): Promise<T> {
		const run = async () => {
			await ready
			const value = callback()
			await persist()
			return value
		}

		const result = mutationChain.then(run, run)
		mutationChain = result.then(
			() => undefined,
			() => undefined,
		)
		return result
	}

	function setQueueState(items: string[], mode: boolean) {
		isQueueMode.value = mode
		currentQueue.value = {
			items: [...items],
			total: items.length,
			completed: 0,
			skipped: 0,
			lastUpdated: new Date(),
		}
	}

	async function setQueue(projectIds: string[]): Promise<void> {
		await withMutation(() => {
			setQueueState(projectIds, true)
		})
	}

	async function setSingleProject(projectId: string): Promise<void> {
		await withMutation(() => {
			setQueueState([projectId], false)
		})
	}

	async function completeCurrentProject(
		projectId: string,
		status: 'completed' | 'skipped' = 'completed',
	): Promise<boolean> {
		return withMutation(() => {
			if (!currentQueue.value.items.includes(projectId)) {
				return currentQueue.value.items.length > 0
			}

			if (status === 'completed') {
				currentQueue.value.completed++
			} else {
				currentQueue.value.skipped++
			}

			currentQueue.value.items = currentQueue.value.items.filter((id) => id !== projectId)
			currentQueue.value.lastUpdated = new Date()

			return currentQueue.value.items.length > 0
		})
	}

	function getCurrentProjectId(): string | null {
		return currentQueue.value.items[0] || null
	}

	async function resetQueue(): Promise<void> {
		await withMutation(() => {
			isQueueMode.value = false
			currentQueue.value = createEmptyQueue()
		})
	}

	async function acquireLock(projectId: string): Promise<LockAcquireResponse> {
		await ready

		try {
			const response = (await useBaseFetch(`moderation/lock/${projectId}`, {
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
			console.error('Failed to acquire moderation lock:', error)
			return { success: false, is_own_lock: false }
		}
	}

	async function overrideLock(projectId: string): Promise<LockAcquireResponse> {
		await ready

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
		await ready

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
		await ready

		try {
			const response = (await useBaseFetch(`moderation/lock/${projectId}`, {
				method: 'GET',
				internal: true,
			})) as LockStatusResponse
			return response
		} catch (error) {
			console.error('Failed to check moderation lock:', error)
			return { locked: false, is_own_lock: false }
		}
	}

	async function refreshLock(): Promise<LockAcquireResponse> {
		await ready

		if (!currentLock.value) return { success: false, is_own_lock: false }

		try {
			const response = await acquireLock(currentLock.value.projectId)
			return response
		} catch (error) {
			console.error('Failed to refresh moderation lock:', error)
			currentLock.value = null
			return { success: false, is_own_lock: false }
		}
	}

	return proxyRefs({
		currentQueue,
		currentLock,
		isQueueMode,
		hydrated,
		ready,

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
	}) as ModerationQueueService
}

export const createModerationQueueService = createModerationQueueState
