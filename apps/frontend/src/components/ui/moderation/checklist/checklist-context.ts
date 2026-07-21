import type { IdentifiedNodeBuilder, NodeState } from '@modrinth/moderation'
import type { ComputedRef, InjectionKey, Ref } from 'vue'

export interface ActiveAction {
	node: IdentifiedNodeBuilder
	state: Record<string, NodeState>
	statePath: string[]
}

export interface LiveNode {
	isActive: boolean
	isVisible: boolean
	isFixActionable: boolean
	messageCount: number
	fixCount: number
	hasRequiredMissing: boolean
	activeActions: ActiveAction[]
}

export const NODE_META_KEY: InjectionKey<ComputedRef<Map<IdentifiedNodeBuilder, LiveNode>>> =
	Symbol('nodeMeta')
export const STATE_KEY: InjectionKey<Ref<Record<string, Record<string, NodeState>>>> =
	Symbol('checklistState')
