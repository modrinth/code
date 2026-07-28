import type { InjectionKey, Ref } from 'vue'

import type { NodeMeta } from './node-meta'

export interface ChecklistMetaContext {
	metaMap: Map<object, NodeMeta>
	attentionMap: Map<object, boolean>
}

export const CHECKLIST_META_KEY: InjectionKey<Ref<ChecklistMetaContext>> = Symbol('checklistMeta')
