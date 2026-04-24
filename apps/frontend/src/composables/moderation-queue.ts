import type { ModerationQueueService } from '~/services/moderation-queue.ts'

export function useModerationQueue(): ModerationQueueService {
	return useNuxtApp().$moderationQueue as ModerationQueueService
}
