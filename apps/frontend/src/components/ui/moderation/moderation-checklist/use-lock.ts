import type { AbstractWebNotificationManager } from '@modrinth/ui'
import { ref } from 'vue'

import type { LockAcquireResponse, ModerationQueueService } from '~/services/moderation/queue.ts'

interface ChecklistLockOptions {
	projectId: string
	queue: ModerationQueueService
	addNotification: typeof AbstractWebNotificationManager.prototype.addNotification
	refreshPrefetchQueue: () => void
}

interface ChecklistLockStatus {
	locked: boolean
	lockedBy?: { id: string; username: string; avatar_url?: string }
	lockedAt?: Date
	expiresAt?: Date
	expired?: boolean
	isOwnLock: boolean
}

export function useChecklistLock({
	projectId,
	queue,
	addNotification,
	refreshPrefetchQueue,
}: ChecklistLockOptions) {
	const status = ref<ChecklistLockStatus | null>(null)
	const error = ref(false)
	const timeRemaining = ref<string | null>(null)
	let heartbeat: ReturnType<typeof setInterval> | null = null
	let countdown: ReturnType<typeof setInterval> | null = null

	function clearCountdown() {
		if (countdown) {
			clearInterval(countdown)
			countdown = null
		}
		timeRemaining.value = null
	}

	function updateCountdown() {
		if (!status.value?.lockedAt || status.value.isOwnLock) {
			timeRemaining.value = null
			return
		}

		const lockedAt = new Date(status.value.lockedAt)
		const expiresAt = status.value.expiresAt
			? new Date(status.value.expiresAt)
			: new Date(lockedAt.getTime() + 15 * 60 * 1000)
		const remainingMs = expiresAt.getTime() - Date.now()

		if (remainingMs <= 0) {
			timeRemaining.value = null
			status.value.expired = true
			clearCountdown()
			return
		}

		const minutes = Math.floor(remainingMs / 60000)
		const seconds = Math.floor((remainingMs % 60000) / 1000)
		timeRemaining.value = `${minutes}:${seconds.toString().padStart(2, '0')}`
	}

	function startCountdown() {
		clearCountdown()
		updateCountdown()
		countdown = setInterval(updateCountdown, 1000)
	}

	function setLockedBy(result: LockAcquireResponse) {
		status.value = {
			locked: result.locked_by != null,
			lockedBy: result.locked_by,
			lockedAt: result.locked_at ? new Date(result.locked_at) : undefined,
			expiresAt: result.expires_at ? new Date(result.expires_at) : undefined,
			expired: result.expired,
			isOwnLock: false,
		}
		error.value = false
		if (result.locked_by) startCountdown()
		else clearCountdown()
	}

	function handleLost(result: LockAcquireResponse) {
		if (heartbeat) {
			clearInterval(heartbeat)
			heartbeat = null
		}
		setLockedBy(result)

		if (result.locked_by) {
			addNotification({
				title: 'Lock taken over',
				text: `@${result.locked_by.username} is now moderating this project.`,
				type: 'warning',
			})
		} else {
			addNotification({
				title: 'Moderation lock lost',
				text: 'Your lock on this project has expired. Acquire the lock again to continue.',
				type: 'warning',
			})
		}
	}

	function startHeartbeat() {
		if (heartbeat) clearInterval(heartbeat)
		heartbeat = setInterval(
			async () => {
				const result = await queue.refreshLock()
				if (!result.success) handleLost(result)
			},
			5 * 60 * 1000,
		)
	}

	function handleAcquired() {
		status.value = { locked: false, isOwnLock: true }
		error.value = false
		clearCountdown()
		startHeartbeat()
		refreshPrefetchQueue()
	}

	function handleUnavailable() {
		error.value = true
		status.value = { locked: false, isOwnLock: false }
		clearCountdown()
		addNotification({
			title: 'Lock unavailable',
			text: 'Could not acquire moderation lock. Others may also be moderating this project.',
			type: 'warning',
		})
	}

	async function acquire() {
		const result = await queue.acquireLock(projectId)
		if (result.success) handleAcquired()
		else if (result.locked_by) setLockedBy(result)
		else handleUnavailable()
	}

	async function override() {
		const result = await queue.overrideLock(projectId)
		if (result.success) {
			addNotification({
				title: 'Moderation lock overridden',
				text: 'You are now moderating this project.',
				type: 'success',
			})
			handleAcquired()
		} else if (result.locked_by) {
			setLockedBy(result)
		} else {
			handleUnavailable()
		}
	}

	async function handleVisibilityChange() {
		if (document.visibilityState !== 'visible' || !status.value?.isOwnLock) return
		const result = await queue.refreshLock()
		if (!result.success) {
			handleLost(result)
			return
		}
		refreshPrefetchQueue()
	}

	function stop() {
		if (heartbeat) {
			clearInterval(heartbeat)
			heartbeat = null
		}
		clearCountdown()
	}

	return { acquire, error, handleVisibilityChange, override, status, stop, timeRemaining }
}
