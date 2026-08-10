import type { ActiveAction } from '@modrinth/moderation/src/types/node'

export interface LiveNode {
	isActive: boolean
	isVisible: boolean
	isFixActionable: boolean
	messageCount: number
	fixCount: number
	hasRequiredMissing: boolean
	activeActions: ActiveAction[]
}
