import { createModerationQueueService } from '~/services/moderation-queue.ts'

export default defineNuxtPlugin(() => {
	const moderationQueue = createModerationQueueService()

	return {
		provide: { moderationQueue },
	}
})
