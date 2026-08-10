import type {
	AnyNode,
	ChildNode,
	Configurable,
	Enableable,
	HasValue,
	Identified,
	NodeMeta,
	NodePropsContext,
	NodeState,
	Renderable,
	Tweakable,
	Writer,
} from '@modrinth/moderation/src/types/node'

export type RenderableValueNode = AnyNode &
	HasValue &
	Identified &
	Partial<Enableable> &
	Renderable &
	Partial<Configurable> &
	Partial<Tweakable>

export interface ChecklistMeta {
	metaMap: Map<object, NodeMeta>
	attentionMap: Map<object, boolean>
	tooltipHtml: Map<object, string>
}

export interface RendererPropsContext extends NodePropsContext {
	nodeFacts: { needsAttention: boolean; fixActionable: boolean }
}

export interface NodeRendererProps {
	nodes: ChildNode[]
	state: Record<string, NodeState>
	write: Writer
	meta: ChecklistMeta
	onImageUpload?: (file: File) => Promise<string>
	flex?: boolean
	titleDepth?: number
	globalState?: Record<string, Record<string, NodeState>>
}
