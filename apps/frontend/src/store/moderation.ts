import { useModerationQueue } from '~/composables/moderation-queue.ts'
import type {
	LockAcquireResponse,
	LockedByUser,
	LockStatusResponse,
	ModerationQueue,
	ModerationQueueService,
} from '~/services/moderation-queue.ts'

export type {
	LockAcquireResponse,
	LockedByUser,
	LockStatusResponse,
	ModerationQueue,
	ModerationQueueService,
}

export const useModerationStore = useModerationQueue
