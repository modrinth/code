import type {
	LockAcquireResponse,
	LockedByUser,
	LockStatusResponse,
	ModerationQueue,
	ModerationQueueService,
} from '~/services/moderation-queue.ts'
import { useModerationQueue } from '~/services/moderation-queue.ts'

export type {
	LockAcquireResponse,
	LockedByUser,
	LockStatusResponse,
	ModerationQueue,
	ModerationQueueService,
}

export const useModerationStore = useModerationQueue
