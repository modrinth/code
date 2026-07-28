import type { ActiveAction2, NodeState } from '@modrinth/moderation/src/types/node'
import type { InjectionKey, Ref } from 'vue'

export interface LiveNode {
	isActive: boolean
	isVisible: boolean
	isFixActionable: boolean
	messageCount: number
	fixCount: number
	hasRequiredMissing: boolean
	activeActions: ActiveAction2[]
}

export const STATE_KEY: InjectionKey<Ref<Record<string, Record<string, NodeState>>>> =
	Symbol('checklistState')
