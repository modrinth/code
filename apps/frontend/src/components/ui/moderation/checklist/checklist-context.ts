import type { ActionBuilder, IdentifiedNodeBuilder, NodeContext } from '@modrinth/moderation'
import type { ComputedRef, InjectionKey } from 'vue'

export interface ActiveAction {
	action: ActionBuilder
	ctx: NodeContext
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

export const NODE_META_KEY: InjectionKey<ComputedRef<Map<IdentifiedNodeBuilder, LiveNode>>> = Symbol('nodeMeta')
