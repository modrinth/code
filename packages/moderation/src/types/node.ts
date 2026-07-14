import type { Labrinth } from '@modrinth/api-client'
import type { FunctionalComponent, SVGAttributes } from 'vue'

import {
	expandVariables,
	flattenProjectV3Variables,
	flattenProjectVariables,
	flattenStaticVariables,
} from '../utils'
import type { ModerationSeverity, ModerationStatus } from './actions'
import type { Priority } from './priority.ts'

// ─── State ────────────────────────────────────────────────────────────────────

export type NodeState = boolean | string | number | Set<string> | NodeStateWithChildren | null | undefined

export interface NodeStateWithChildren {
	value?: NodeState
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

export type Reactive<T> = T | ((ctx: NodeContext) => T)

export function resolve<T>(value: Reactive<T>, ctx: NodeContext): T {
	return typeof value === 'function' ? (value as (ctx: NodeContext) => T)(ctx) : value
}

export type NodeType =
	| 'toggle'
	| 'check'
	| 'button'
	| 'text'
	| 'markdown'
	| 'group'
	| 'dropdown'
	| 'option'
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
	path: string | ((ctx: NodeContext) => string),
	getVars?: (ctx: NodeContext) => Record<string, NodeState>,
): MessageFn {
	return async (ctx) => {
		const resolvedPath = typeof path === 'function' ? path(ctx) : path
		const loader = messageFiles[`../data/messages/${resolvedPath}.md`]
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

export function createTrackedPatch<T extends object>(source: T): { proxy: T; changes: () => Partial<T> } {
	const written = new Set<string | symbol>()
	const data = { ...source }
	const proxy = new Proxy(data, {
		set(target, key, value) {
			if (value !== (source as Record<string | symbol, unknown>)[key]) {
				written.add(key)
			} else {
				written.delete(key)
			}
			;(target as Record<string | symbol, unknown>)[key] = value
			return true
		},
	})
	return {
		proxy,
		changes: () => Object.fromEntries([...written].map((k) => [k, data[k as keyof T]])) as Partial<T>,
	}
}

export class FixBuilder {
	_projectFn?: (patch: Labrinth.Projects.v3.EditProjectRequest, ctx: NodeContext) => void
	_versionFn?: (patch: Labrinth.Versions.v3.ModifyVersionRequest, ctx: NodeContext) => void

	project(fn: (patch: Labrinth.Projects.v3.EditProjectRequest, ctx: NodeContext) => void): this {
		this._projectFn = fn
		return this
	}

	version(fn: (patch: Labrinth.Versions.v3.ModifyVersionRequest, ctx: NodeContext) => void): this {
		this._versionFn = fn
		return this
	}
}

export function fix(): FixBuilder {
	return new FixBuilder()
}

// ─── Action builder ───────────────────────────────────────────────────────────

export class ActionBuilder {
	_message?: MessageFn
	_priority?: Priority
	_suggestedStatus?: ModerationStatus
	_severity?: ModerationSeverity
	_fixes: FixBuilder[] = []
	_applyFixes = false

	message(fn: MessageFn): this {
		this._message = fn
		return this
	}

	priority(p: Priority): this {
		this._priority = p
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

	applyFixes(): this {
		this._applyFixes = true
		return this
	}
}

export function action(): ActionBuilder {
	return new ActionBuilder()
}

// ─── Node builders ────────────────────────────────────────────────────────────

export abstract class NodeBuilder {
	abstract readonly type: NodeType
	_shown?: Reactive<boolean>

	shown(condition: Reactive<boolean>): this {
		this._shown = condition
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

export class ButtonNodeBuilder extends NodeBuilder {
	readonly type = 'button' as const
	readonly label: string
	_onClick?: (ctx: NodeContext) => void
	_enabled?: Reactive<boolean>

	constructor(nodeLabel: string) {
		super()
		this.label = nodeLabel
	}

	onClick(fn: (ctx: NodeContext) => void): this {
		this._onClick = fn
		return this
	}

	enabled(condition: Reactive<boolean>): this {
		this._enabled = condition
		return this
	}
}

export abstract class IdentifiedNodeBuilder extends NodeBuilder {
	readonly id: string | undefined
	_children: NodeBuilder[] = []
	_childrenFn?: ChildrenFn
	_computingChildren = false
	_action?: ActionBuilder
	_enabled?: Reactive<boolean>
	_statePath?: string[]

	constructor(id: string | undefined) {
		super()
		this.id = id
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

	enabled(condition: Reactive<boolean>): this {
		this._enabled = condition
		return this
	}

	statePath(path: string[]): this {
		this._statePath = path
		return this
	}
}

export abstract class LabeledNodeBuilder extends IdentifiedNodeBuilder {
	readonly label: string

	constructor(id: string, label: string) {
		super(id)
		this.label = label
	}
}

export abstract class ValueNodeBuilder extends LabeledNodeBuilder {
	_defaultValue?: NodeState | ((ctx: NodeContext) => NodeState)
	_required?: boolean

	initial(v: NodeState | ((ctx: NodeContext) => NodeState)): this {
		this._defaultValue = v
		return this
	}

	required(v = true): this {
		this._required = v
		return this
	}
}

export class BooleanNodeBuilder extends ValueNodeBuilder {
	readonly type: 'toggle' | 'check'

	constructor(id: string, nodeLabel: string, type: 'toggle' | 'check') {
		super(id, nodeLabel)
		this.type = type
	}
}

export class InputNodeBuilder extends ValueNodeBuilder {
	readonly type: 'text' | 'markdown'
	_placeholder?: Reactive<string>

	constructor(id: string, nodeLabel: string, type: 'text' | 'markdown') {
		super(id, nodeLabel)
		this.type = type
	}

	placeholder(p: Reactive<string>): this {
		this._placeholder = p
		return this
	}
}

export class GroupNodeBuilder extends IdentifiedNodeBuilder {
	readonly type = 'group' as const
	_layout?: 'flex' | 'column'
	_title?: Reactive<string>
	_required?: boolean
	_selectMode?: 'single' | 'multi'

	constructor(id?: string) {
		super(id)
	}

	layout(l: 'flex' | 'column'): this {
		this._layout = l
		return this
	}

	title(t: Reactive<string>): this {
		this._title = t
		return this
	}

	required(v = true): this {
		this._required = v
		return this
	}

	singleSelect(): this {
		this._selectMode = 'single'
		return this
	}

	multiSelect(): this {
		this._selectMode = 'multi'
		return this
	}
}

export class DropdownNodeBuilder extends IdentifiedNodeBuilder {
	readonly type = 'dropdown' as const
	declare _children: OptionNodeBuilder[]
	_none?: string

	constructor(id: string) {
		super(id)
	}

	override children(...nodes: OptionNodeBuilder[]): this
	override children(fn: ChildrenFn): this
	override children(...args: OptionNodeBuilder[] | [ChildrenFn]): this {
		if (args.length === 1 && typeof args[0] === 'function') {
			super.children(args[0] as ChildrenFn)
		} else {
			super.children(...(args as OptionNodeBuilder[]))
		}
		return this
	}

	none(text: string): this {
		this._none = text
		return this
	}
}

export class OptionNodeBuilder extends LabeledNodeBuilder {
	readonly type = 'option' as const

	constructor(id: string, nodeLabel: string) {
		super(id, nodeLabel)
	}
}

export class StageNodeBuilder extends LabeledNodeBuilder {
	readonly type = 'stage' as const
	_hint?: string
	_guidanceUrl?: string
	_icon?: FunctionalComponent<SVGAttributes>
	_navigate?: string

	hint(h: string): this {
		this._hint = h
		return this
	}

	guidance(url: string): this {
		this._guidanceUrl = url
		return this
	}

	icon(i: FunctionalComponent<SVGAttributes>): this {
		this._icon = i
		return this
	}

	navigate(path: string): this {
		this._navigate = path
		return this
	}

	override children(...nodes: NodeBuilder[]): this
	override children(fn: ChildrenFn): this
	override children(...args: NodeBuilder[] | [ChildrenFn]): this {
		if (typeof args[0] === 'function') {
			super.children(args[0] as ChildrenFn)
			if (!this._statePath) this._statePath = [this.id!]
			return this
		}
		const nodes = args as NodeBuilder[]
		super.children(...nodes)
		if (!this._statePath) this._statePath = [this.id!]
		stampChildPaths(nodes, [this.id!])
		return this
	}
}

// ─── Node traversal ───────────────────────────────────────────────────────────

function childrenScopePath(node: IdentifiedNodeBuilder): string[] | null {
	if (!node._statePath) return null
	switch (node.type) {
		case 'toggle':
		case 'check':
		case 'option':
		case 'stage':
			return node._statePath
		case 'group': {
			const g = node as GroupNodeBuilder
			if (g._selectMode) {
				// Option ids become namespaces at the same level as the group
				return node._statePath.slice(0, -1)
			}
			return node._statePath
		}
		case 'dropdown':
			// Option ids become namespaces at the same level as the dropdown
			return node._statePath.slice(0, -1)
		default:
			return null
	}
}

function stampChildPaths(nodes: NodeBuilder[], scopePath: string[]): void {
	for (const node of nodes) {
		if (!(node instanceof IdentifiedNodeBuilder)) continue
		if (!node.id) {
			stampChildPaths(node._children, scopePath)
			continue
		}
		if (!node._statePath) {
			node._statePath = [...scopePath, node.id]
		}
		const childScope = childrenScopePath(node)
		if (childScope) stampChildPaths(node._children, childScope)
	}
}

export function isNodeActive(node: NodeBuilder, state: NodeState): boolean {
	switch (node.type) {
		case 'toggle':
		case 'check': {
			if (typeof state === 'boolean') return state
			if (state && typeof state === 'object' && !(state instanceof Set)) {
				const v = (state as NodeStateWithChildren).value
				if (typeof v === 'boolean') return v
			}
			return (node as BooleanNodeBuilder)._defaultValue === true
		}
		case 'group': {
			const g = node as GroupNodeBuilder
			if (g._selectMode === 'single') return typeof state === 'string' && state !== ''
			if (g._selectMode === 'multi') return state instanceof Set && state.size > 0
			return false
		}
		case 'dropdown': return typeof state === 'string' && state !== ''
		case 'text':
		case 'markdown': return typeof state === 'string' && state !== ''
		default: return false
	}
}

export function getBooleanChildState(nodeState: NodeState): Record<string, NodeState> {
	if (nodeState && typeof nodeState === 'object' && !(nodeState instanceof Set)) {
		return nodeState as Record<string, NodeState>
	}
	return {}
}

export function resolveChildren(node: IdentifiedNodeBuilder, ctx: NodeContext): NodeBuilder[] {
	if (node._childrenFn) {
		if (node._computingChildren) return []
		node._computingChildren = true
		try {
			const result = node._childrenFn(ctx)
			const scopePath = childrenScopePath(node)
			if (scopePath) stampChildPaths(result, scopePath)
			return result
		} finally {
			node._computingChildren = false
		}
	}
	return node._children
}

export function walkNodes(
	nodes: NodeBuilder[],
	stageState: Record<string, NodeState>,
	ctx: NodeContext,
	visitor: (node: IdentifiedNodeBuilder, state: NodeState, ctx: NodeContext) => void,
): void {
	for (const node of nodes) {
		if (node._shown !== undefined && !resolve(node._shown, ctx)) continue

		if (node.type === 'stage') {
			const identified = node as IdentifiedNodeBuilder
			if (identified.id) {
				const raw = stageState[identified.id]
				const childState = (raw && typeof raw === 'object' && !(raw instanceof Set))
					? (raw as Record<string, NodeState>)
					: {}
				walkNodes(resolveChildren(identified, ctx), childState, { ...ctx, state: childState }, visitor)
			} else {
				walkNodes(resolveChildren(identified, ctx), stageState, ctx, visitor)
			}
			continue
		}

		if (node.type === 'group') {
			const g = node as GroupNodeBuilder
			if (!g._selectMode) {
				// Plain container: traverse children in group's own state scope
				if (g.id) {
					const raw = stageState[g.id]
					const childState = (raw && typeof raw === 'object' && !(raw instanceof Set))
						? (raw as Record<string, NodeState>)
						: {}
					walkNodes(resolveChildren(g, ctx), childState, { ...ctx, state: childState }, visitor)
				} else {
					walkNodes(resolveChildren(g, ctx), stageState, ctx, visitor)
				}
				continue
			}
			// Fall through to value-node path for groups with selectMode
		}

		if (node.type === 'display' || node.type === 'button') continue

		const identified = node as IdentifiedNodeBuilder
		const nodeState = stageState[identified.id!]
		visitor(identified, nodeState, ctx)

		const active = isNodeActive(node, nodeState)
		const children = resolveChildren(identified, ctx)
		if (children.length === 0 || !active) continue

		if (node.type === 'group' && (node as GroupNodeBuilder)._selectMode === 'multi') {
			const selected = nodeState instanceof Set ? nodeState : new Set<string>()
			for (const child of children) {
				const childId = child as IdentifiedNodeBuilder
				if (!selected.has(childId.id!) || (child._shown !== undefined && !resolve(child._shown, ctx))) continue
				const rawChildState = stageState[childId.id!]
				// Option children are active by virtue of being in the selected Set, not by their own
				// boolean state. If they have child state but no explicit value, inject value: true
				// so isNodeActive returns true even after child state has been written to their path.
				const childState: NodeState = (rawChildState !== null && rawChildState !== undefined && typeof rawChildState === 'object' && !(rawChildState instanceof Set) && (rawChildState as NodeStateWithChildren).value === undefined)
					? { ...(rawChildState as NodeStateWithChildren), value: true }
					: (rawChildState ?? true)
				visitor(childId, childState, ctx)
				walkNodes(resolveChildren(childId, ctx), stageState, ctx, visitor)
			}
		} else if (node.type === 'toggle' || node.type === 'check') {
			const childState = getBooleanChildState(nodeState)
			walkNodes(children, childState, { ...ctx, state: childState }, visitor)
		} else if (
			(node.type === 'group' && (node as GroupNodeBuilder)._selectMode === 'single') ||
			node.type === 'dropdown'
		) {
			const selectedId = typeof nodeState === 'string' ? nodeState : undefined
			if (selectedId) {
				for (const child of children) {
					const childId = child as IdentifiedNodeBuilder
					if (childId.id !== selectedId || (child._shown !== undefined && !resolve(child._shown, ctx))) continue
					visitor(childId, stageState[childId.id!], ctx)
					walkNodes(resolveChildren(childId, ctx), stageState, ctx, visitor)
					break
				}
			}
		} else {
			walkNodes(children, stageState, ctx, visitor)
		}
	}
}

// ─── Factory functions ────────────────────────────────────────────────────────

export type StageFn = (project: Labrinth.Projects.v3.Project, projectV2: Labrinth.Projects.v2.Project) => StageNodeBuilder

export function stageFn(factory: StageFn): StageFn {
	const cache = new WeakMap<object, StageNodeBuilder>()
	return (project, projectV2) => {
		if (!cache.has(project)) cache.set(project, factory(project, projectV2))
		return cache.get(project)!
	}
}

export const toggle = (id: string, nodeLabel: string) => new BooleanNodeBuilder(id, nodeLabel, 'toggle')
export const check = (id: string, nodeLabel: string) => new BooleanNodeBuilder(id, nodeLabel, 'check')
export const button = (nodeLabel: string) => new ButtonNodeBuilder(nodeLabel)
export const text = (id: string, nodeLabel: string) => new InputNodeBuilder(id, nodeLabel, 'text')
export const markdown = (id: string, nodeLabel: string) => new InputNodeBuilder(id, nodeLabel, 'markdown')
export const group = (id?: string) => new GroupNodeBuilder(id)
export const dropdown = (id: string) => new DropdownNodeBuilder(id)
export const option = (id: string, nodeLabel: string) => new OptionNodeBuilder(id, nodeLabel)
export const label = (content: string | ContentFn) => new DisplayNodeBuilder(content)
export const stage = (id: string, title: string) => new StageNodeBuilder(id, title)
