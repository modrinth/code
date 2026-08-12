import type { AbstractModrinthClient, Labrinth } from '@modrinth/api-client'
import { injectModrinthClient } from '@modrinth/ui'
import { computed, proxyRefs, ref } from 'vue'

import {
	loadQueueState,
	type PersistedModerationQueueState,
	saveQueueState,
} from './queue-storage.ts'

export interface ModerationQueue {
	items: string[]
	skipped: string[]
	total: number
	completed: string[]
	lastUpdated: Date
}

export type LockedByUser = Labrinth.Moderation.Internal.LockedByUser
export type LockStatusResponse = Labrinth.Moderation.Internal.LockStatusResponse
export type LockAcquireResponse = Labrinth.Moderation.Internal.LockAcquireResponse

export interface ModerationQueueService {
	currentQueue: ModerationQueue
	currentLock: { projectId: string; lockedAt: Date } | null
	isQueueMode: boolean
	hydrated: boolean
	ready: Promise<void>

	queueLength: number
	hasItems: boolean
	hasSkipped: boolean
	progress: number

	setQueue(projectIds: string[]): Promise<void>
	completeProject(projectId: string): Promise<boolean>
	deferProject(projectId: string): Promise<boolean>
	excludeProject(projectId: string): Promise<boolean>
	startSkippedReview(): Promise<void>
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
	skipped: [],
	total: 0,
	completed: [],
	lastUpdated: new Date(),
}

function createEmptyQueue(): ModerationQueue {
	return { ...EMPTY_QUEUE, lastUpdated: new Date(), items: [], skipped: [], completed: [] }
}

function sanitizeQueue(raw: PersistedModerationQueueState['currentQueue']): ModerationQueue {
	const lastUpdated = new Date(raw.lastUpdated)
	const items = raw.items.filter((id): id is string => typeof id === 'string')
	const skipped = (raw.skipped ?? []).filter((id): id is string => typeof id === 'string')
	const completed = (raw.completed ?? []).filter((id): id is string => typeof id === 'string')
	const minimumTotal = items.length + completed.length
	const total = Number.isFinite(raw.total)
		? Math.max(Math.trunc(raw.total), minimumTotal)
		: minimumTotal

	return {
		items,
		skipped,
		total,
		completed,
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
			skipped: [...queue.skipped],
			total: queue.total,
			completed: [...queue.completed],
			lastUpdated: queue.lastUpdated.toISOString(),
		},
		isQueueMode,
	}
}

function createModerationQueueState(client: AbstractModrinthClient = injectModrinthClient()) {
	const currentQueue = ref(createEmptyQueue())
	const currentLock = ref<{ projectId: string; lockedAt: Date } | null>(null)
	const isQueueMode = ref(false)
	const hydrated = ref(false)

	const queueLength = computed(() => currentQueue.value.items.length)
	const hasItems = computed(() => currentQueue.value.items.length > 0)
	const hasSkipped = computed(() => currentQueue.value.skipped.length > 0)
	const progress = computed(() => {
		if (currentQueue.value.total === 0) return 0
		return (currentQueue.value.total - currentQueue.value.items.length) / currentQueue.value.total
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

	function setQueueState(items: string[]) {
		isQueueMode.value = true
		currentQueue.value = {
			items: [...items],
			skipped: [],
			total: items.length,
			completed: [],
			lastUpdated: new Date(),
		}
	}

	async function setQueue(projectIds: string[]): Promise<void> {
		await withMutation(() => {
			setQueueState(projectIds)
		})
	}

	async function completeProject(projectId: string): Promise<boolean> {
		return withMutation(() => {
			if (!currentQueue.value.items.includes(projectId)) {
				return currentQueue.value.items.length > 0
			}

			currentQueue.value.completed = [...currentQueue.value.completed, projectId]
			currentQueue.value.items = currentQueue.value.items.filter((id) => id !== projectId)
			currentQueue.value.lastUpdated = new Date()

			return currentQueue.value.items.length > 0
		})
	}

	async function excludeProject(projectId: string): Promise<boolean> {
		return withMutation(() => {
			if (!currentQueue.value.items.includes(projectId)) {
				return currentQueue.value.items.length > 0
			}

			currentQueue.value.items = currentQueue.value.items.filter((id) => id !== projectId)
			currentQueue.value.lastUpdated = new Date()

			return currentQueue.value.items.length > 0
		})
	}

	async function deferProject(projectId: string): Promise<boolean> {
		return withMutation(() => {
			if (!currentQueue.value.items.includes(projectId)) {
				return currentQueue.value.items.length > 0
			}

			currentQueue.value.items = currentQueue.value.items.filter((id) => id !== projectId)
			currentQueue.value.skipped = [...currentQueue.value.skipped, projectId]
			currentQueue.value.lastUpdated = new Date()

			return currentQueue.value.items.length > 0
		})
	}

	async function startSkippedReview(): Promise<void> {
		await withMutation(() => {
			setQueueState(currentQueue.value.skipped)
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
			const response = await client.labrinth.moderation_internal.acquireLock(projectId)

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
			const response = await client.labrinth.moderation_internal.overrideLock(projectId)

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
			const response = await client.labrinth.moderation_internal.releaseLock(projectId)

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
			const response = await client.labrinth.moderation_internal.checkLock(projectId)
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
		hasSkipped,
		progress,

		setQueue,
		completeProject,
		deferProject,
		excludeProject,
		startSkippedReview,
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

const moderationQueueServices = new WeakMap<object, ModerationQueueService>()

export function useModerationQueue(): ModerationQueueService {
	const nuxtApp = useNuxtApp()
	const existingService = moderationQueueServices.get(nuxtApp)
	if (existingService) return existingService

	const service = createModerationQueueService()
	moderationQueueServices.set(nuxtApp, service)
	return service
}
