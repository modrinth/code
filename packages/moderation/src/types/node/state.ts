import type { Ref } from 'vue'
import { toValue } from 'vue'

export type ModerationStatus = 'approved' | 'rejected' | 'flagged'

export type NodeState =
	| boolean
	| string
	| number
	| Set<string>
	| NodeStateWithChildren
	| null
	| undefined

export interface NodeStateWithChildren {
	value?: NodeState
	[childId: string]: NodeState
}

export type Reactive<T> = T | Ref<T>

export function resolve<T>(value: Reactive<T>): T {
	return toValue(value as T | Ref<T>)
}

export type GetVarsFn = (state: Record<string, any>) => Record<string, any>
export type ContentFn = (state: Record<string, NodeState>) => string | Promise<string>

export type MessageSegment =
	| { type: 'fn'; fn: ContentFn }
	| { type: 'auto'; getVars?: GetVarsFn }
	| { type: 'path'; path: string | (() => string); getVars?: GetVarsFn }
	| { type: 'collect'; fallback?: MessageSegment }
