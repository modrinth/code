import type { Labrinth } from '@modrinth/api-client'
import type { FunctionalComponent, SVGAttributes } from 'vue'

import {
	expandVariables,
	flattenProjectV3Variables,
	flattenProjectVariables,
	flattenStaticVariables,
} from '../utils'

// ─── State ────────────────────────────────────────────────────────────────────

export type NodeState = boolean | string | number | Set<string> | NodeStateWithChildren | null | undefined

export interface NodeStateWithChildren {
	value?: boolean | string | number | Set<string> | null
	[childId: string]: NodeState
}

// ─── Context ──────────────────────────────────────────────────────────────────

export interface NodeContext {
	project: Labrinth.Projects.v3.Project
	projectV2: Labrinth.Projects.v2.Project
	state: Record<string, NodeState>
	globalState: Record<string, Record<string, NodeState>>
}

// ─── Function types ───────────────────────────────────────────────────────────

export type MessageFn = (ctx: NodeContext) => Promise<string>
export type ContentFn = (ctx: NodeContext) => string | Promise<string>
export type ShowFn = (ctx: NodeContext) => boolean
export type ChildrenFn = (ctx: NodeContext) => NodeBuilder[]

// ─── Enum-like types ──────────────────────────────────────────────────────────

export type ModerationStatus = 'approved' | 'rejected' | 'flagged'
export type ModerationSeverity = 'low' | 'medium' | 'high' | 'critical'

export type NodeType =
	| 'button'
	| 'toggle'
	| 'text'
	| 'markdown'
	| 'group'
	| 'select'
	| 'multi-select'
	| 'display'
	| 'stage'

// ─── Message helpers ──────────────────────────────────────────────────────────

const messageFiles = import.meta.glob('../data/messages/**/*.md', {
	query: '?raw',
	import: 'default',
})

export function mdEscape(text: string): string {
	return text.replace(/[\\*_`[~]/g, '\\$&')
}

const USER_CONTENT_KEYS = ['PROJECT_TITLE', 'PROJECT_SLUG', 'PROJECT_SUMMARY', 'PROJECT_TYPE', 'PROJECT_STATUS']

export function md(
	path: string,
	getVars?: (ctx: NodeContext) => Record<string, NodeState>,
): MessageFn {
	return async (ctx) => {
		const loader = messageFiles[`../data/messages/${path}.md`]
		if (!loader) return ''
		const raw = (await loader()) as string
		const vars: Record<string, string> = {
			...flattenStaticVariables(),
			...flattenProjectVariables(ctx.projectV2),
			...flattenProjectV3Variables(ctx.project),
		}
		for (const key of USER_CONTENT_KEYS) {
			if (key in vars) vars[key] = mdEscape(vars[key])
		}
		if (getVars) {
			for (const [k, v] of Object.entries(getVars(ctx))) {
				vars[k] = String(v ?? '')
			}
		}
		return expandVariables(raw, ctx.projectV2, ctx.project, vars)
	}
}
// ─── Fix builder ──────────────────────────────────────────────────────────────

type ProjectPatch = Partial<Labrinth.Projects.v3.Project>
type VersionPatch = Partial<Labrinth.Projects.v3.Version>

export class FixBuilder {
	_projectFn?: (patch: ProjectPatch) => void | false
	_versionsFn?: (patches: Map<string, VersionPatch>) => void | false

	project(fn: (patch: ProjectPatch) => void | false): this {
		this._projectFn = fn
		return this
	}

	versions(fn: (patches: Map<string, VersionPatch>) => void | false): this {
		this._versionsFn = fn
		return this
	}
}

export function fix(): FixBuilder {
	return new FixBuilder()
}

// ─── Action builder ───────────────────────────────────────────────────────────

export class ActionBuilder {
	_message?: MessageFn
	_weight?: number
	_suggestedStatus?: ModerationStatus
	_severity?: ModerationSeverity
	_fixes: FixBuilder[] = []

	message(fn: MessageFn): this {
		this._message = fn
		return this
	}

	weight(n: number): this {
		this._weight = n
		return this
	}

	suggestedStatus(s: ModerationStatus): this {
		this._suggestedStatus = s
		return this
	}

	severity(s: ModerationSeverity): this {
		this._severity = s
		return this
	}

	fix(f: FixBuilder): this {
		this._fixes.push(f)
		return this
	}
}

export function action(): ActionBuilder {
	return new ActionBuilder()
}

// ─── Node builders ────────────────────────────────────────────────────────────

export abstract class NodeBuilder {
	abstract readonly type: NodeType
	_shown?: ShowFn

	shown(fn: ShowFn): this {
		this._shown = fn
		return this
	}
}

export class DisplayNodeBuilder extends NodeBuilder {
	readonly type = 'display' as const
	_content: string | ContentFn

	constructor(content: string | ContentFn) {
		super()
		this._content = content
	}
}

export abstract class IdentifiedNodeBuilder extends NodeBuilder {
	readonly id: string | undefined
	readonly label: string
	_children: NodeBuilder[] = []
	_childrenFn?: ChildrenFn
	_action?: ActionBuilder
	_enabled?: ShowFn

	constructor(id: string | undefined, label: string) {
		super()
		this.id = id
		this.label = label
	}

	children(...nodes: NodeBuilder[]): this
	children(fn: ChildrenFn): this
	children(...args: NodeBuilder[] | [ChildrenFn]): this {
		if (args.length === 1 && typeof args[0] === 'function') {
			this._childrenFn = args[0] as ChildrenFn
		} else {
			this._children.push(...(args as NodeBuilder[]))
		}
		return this
	}

	action(a: ActionBuilder): this {
		this._action = a
		return this
	}

	enabled(fn: ShowFn): this {
		this._enabled = fn
		return this
	}
}

export abstract class ValueNodeBuilder extends IdentifiedNodeBuilder {
	_defaultValue?: NodeState
	_required?: boolean

	initial(v: NodeState): this {
		this._defaultValue = v
		return this
	}

	required(v = true): this {
		this._required = v
		return this
	}
}

export class BooleanNodeBuilder extends ValueNodeBuilder {
	readonly type: 'button' | 'toggle'

	constructor(id: string, nodeLabel: string, type: 'button' | 'toggle') {
		super(id, nodeLabel)
		this.type = type
	}
}

export class InputNodeBuilder extends ValueNodeBuilder {
	readonly type: 'text' | 'markdown'
	_placeholder?: string

	constructor(id: string, nodeLabel: string, type: 'text' | 'markdown') {
		super(id, nodeLabel)
		this.type = type
	}

	placeholder(p: string): this {
		this._placeholder = p
		return this
	}
}

export class SelectNodeBuilder extends ValueNodeBuilder {
	readonly type = 'select' as const

	constructor(id: string, nodeLabel: string) {
		super(id, nodeLabel)
	}
}

export class ChipsNodeBuilder extends ValueNodeBuilder {
	readonly type = 'multi-select' as const

	constructor(id: string, nodeLabel: string) {
		super(id, nodeLabel)
	}
}

export class GroupNodeBuilder extends IdentifiedNodeBuilder {
	readonly type = 'group' as const
	_layout?: 'flex' | 'column'

	constructor(id?: string) {
		super(id, '')
	}

	layout(l: 'flex' | 'column'): this {
		this._layout = l
		return this
	}
}

export class StageNodeBuilder extends IdentifiedNodeBuilder {
	readonly type = 'stage' as const
	readonly hint: string
	readonly guidanceUrl: string
	_icon?: FunctionalComponent<SVGAttributes>
	_navigate?: string

	constructor(id: string, title: string, hint: string, guidanceUrl: string) {
		super(id, title)
		this.hint = hint
		this.guidanceUrl = guidanceUrl
	}

	icon(i: FunctionalComponent<SVGAttributes>): this {
		this._icon = i
		return this
	}

	navigate(path: string): this {
		this._navigate = path
		return this
	}
}

// ─── Factory functions ────────────────────────────────────────────────────────

export const button = (id: string, nodeLabel: string) => new BooleanNodeBuilder(id, nodeLabel, 'button')
export const toggle = (id: string, nodeLabel: string) => new BooleanNodeBuilder(id, nodeLabel, 'toggle')
export const text = (id: string, nodeLabel: string) => new InputNodeBuilder(id, nodeLabel, 'text')
export const markdown = (id: string, nodeLabel: string) => new InputNodeBuilder(id, nodeLabel, 'markdown')
export const group = (id?: string) => new GroupNodeBuilder(id)
export const select = (id: string, nodeLabel: string) => new SelectNodeBuilder(id, nodeLabel)
export const chips = (id: string, nodeLabel: string) => new ChipsNodeBuilder(id, nodeLabel)
export const label = (content: string | ContentFn) => new DisplayNodeBuilder(content)
export const stage = (id: string, title: string, hint: string, guidanceUrl: string) =>
	new StageNodeBuilder(id, title, hint, guidanceUrl)
