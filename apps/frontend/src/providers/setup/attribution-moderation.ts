import { attributionQuickReplies } from '@modrinth/moderation'
import { provideAttributionModeration } from '@modrinth/ui'

export function setupAttributionModerationProvider() {
	provideAttributionModeration({ attributionQuickReplies })
}
