import type { BrowseInstallPlan, BrowseInstallProject, BrowseInstallQueue } from './plan'
import {
	getStoredServerAddonInstallQueue,
	withServerInstallQueueLock,
	withStoredServerInstallQueueFlushLock,
	writeStoredServerInstallQueue,
} from './queue-storage'

/**
 * Inputs for committing queued plans without re-running version matching.
 */
export interface FlushInstallQueueOptions<TProject extends BrowseInstallProject> {
	queue: BrowseInstallQueue<TProject>
	install: (plan: BrowseInstallPlan<TProject>) => void | Promise<void>
	lockKey?: string | null
	onError?: (error: unknown, plan: BrowseInstallPlan<TProject>) => void
	onProgress?: (
		completed: number,
		total: number,
		plan: BrowseInstallPlan<TProject>,
	) => void | Promise<void>
}

export interface FlushStoredServerAddonInstallQueueOptions<TProject extends BrowseInstallProject> {
	serverId: string
	worldId: string
	install: (plans: BrowseInstallPlan<TProject>[]) => void | Promise<void>
	onQueueChange?: (plans: Map<string, BrowseInstallPlan<TProject>>) => void
}

/**
 * Result of a queue flush. Failed plans are also written back to the queue.
 */
export interface FlushInstallQueueResult<TProject extends BrowseInstallProject> {
	ok: boolean
	successfulPlans: BrowseInstallPlan<TProject>[]
	failedPlans: Map<string, BrowseInstallPlan<TProject>>
}

export interface FlushStoredServerAddonInstallQueueResult<TProject extends BrowseInstallProject> {
	ok: boolean
	flushedPlans: BrowseInstallPlan<TProject>[]
	attemptedPlans: BrowseInstallPlan<TProject>[]
	error?: unknown
}

/**
 * Commits queued install plans exactly as stored.
 *
 * Successful plans are removed; failed plans remain in the queue for retry or user action.
 */
export async function flushInstallQueue<TProject extends BrowseInstallProject>({
	queue,
	install,
	lockKey,
	onError,
	onProgress,
}: FlushInstallQueueOptions<TProject>): Promise<FlushInstallQueueResult<TProject>> {
	return await withServerInstallQueueLock(lockKey, () =>
		flushInstallQueueUnlocked({ queue, install, onError, onProgress }),
	)
}

export async function flushStoredServerAddonInstallQueue<TProject extends BrowseInstallProject>({
	serverId,
	worldId,
	install,
	onQueueChange,
}: FlushStoredServerAddonInstallQueueOptions<TProject>): Promise<
	FlushStoredServerAddonInstallQueueResult<TProject>
> {
	let attemptedPlans: BrowseInstallPlan<TProject>[] = []

	try {
		const flushedPlans = await withStoredServerInstallQueueFlushLock(
			serverId,
			worldId,
			async () => {
				const plans = Array.from(
					getStoredServerAddonInstallQueue<TProject>(serverId, worldId).values(),
				)
				attemptedPlans = plans
				if (plans.length === 0) return []

				await install(plans)

				const remainingPlans = getStoredServerAddonInstallQueue<TProject>(serverId, worldId)
				for (const plan of plans) {
					remainingPlans.delete(plan.projectId)
				}
				writeStoredServerInstallQueue(serverId, worldId, remainingPlans)
				onQueueChange?.(remainingPlans)
				return plans
			},
		)

		return {
			ok: true,
			flushedPlans,
			attemptedPlans,
		}
	} catch (error) {
		return {
			ok: false,
			flushedPlans: [],
			attemptedPlans,
			error,
		}
	}
}

async function flushInstallQueueUnlocked<TProject extends BrowseInstallProject>({
	queue,
	install,
	onError,
	onProgress,
}: FlushInstallQueueOptions<TProject>): Promise<FlushInstallQueueResult<TProject>> {
	const queuedPlans = Array.from(queue.get().values())
	const failedPlans = new Map<string, BrowseInstallPlan<TProject>>()
	const successfulPlans: BrowseInstallPlan<TProject>[] = []
	let completed = 0

	for (const plan of queuedPlans) {
		try {
			await install(plan)
			successfulPlans.push(plan)

			const remainingPlans = new Map(queue.get())
			remainingPlans.delete(plan.projectId)
			queue.set(remainingPlans)
		} catch (error) {
			failedPlans.set(plan.projectId, plan)
			onError?.(error, plan)
		} finally {
			completed++
			await onProgress?.(completed, queuedPlans.length, plan)
		}
	}
	return {
		ok: failedPlans.size === 0,
		successfulPlans,
		failedPlans,
	}
}
