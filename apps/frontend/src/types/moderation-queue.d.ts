import type { ModerationQueueService } from '~/services/moderation-queue.ts'

export {}

declare module '#app' {
	interface NuxtApp {
		$moderationQueue: ModerationQueueService
	}
}

declare module 'vue' {
	interface ComponentCustomProperties {
		$moderationQueue: ModerationQueueService
	}
}
