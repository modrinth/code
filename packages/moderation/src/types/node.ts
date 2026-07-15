import type { Labrinth } from '@modrinth/api-client'
import { injectProjectPageContext } from '@modrinth/ui'
import type { FunctionalComponent, InjectionKey, Ref, SVGAttributes } from 'vue'
import { markRaw, toValue } from 'vue'

import {
	expandVariables,
	flattenProjectV3Variables,
	flattenProjectVariables,
	flattenStaticVariables,
} from '../utils'
import type { ModerationSeverity, ModerationStatus } from './actions'
import type { Priority } from './priority.ts'

// ─── State ────────────────────────────────────────────────────────────────────

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

// ─── Function types ───────────────────────────────────────────────────────────

export type MessageFn = ((state: Record<string, NodeState>) => Promise<string>) & {
	concat(...others: MessageFn[]): MessageFn
}
export type ContentFn = (state: Record<string, NodeState>) => string | Promise<string>
export type ChildrenFn = (state: Record<string, NodeState>) => ChildEntry[]

export type Reactive<T> = T | Ref<T>

export function resolve<T>(value: Reactive<T>): T {
	return toValue(value as T | Ref<T>)
}

export type ChildEntry =
	| NodeBuilder
	| null
	| Ref<NodeBuilder | null>
	| ((state?: Record<string, NodeState>) => NodeBuilder | null)

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

const USER_CONTENT_KEYS = [
	'PROJECT_TITLE',
	'PROJECT_SLUG',
	'PROJECT_SUMMARY',
	'PROJECT_TYPE',
	'PROJECT_STATUS',
]

export async function loadMd(
	path: string,
	state: Record<string, NodeState>,
	project: Labrinth.Projects.v3.Project,
	projectV2: Labrinth.Projects.v2.Project,
	getVars?: (state: Record<string, NodeState>) => Record<string, NodeState>,
): Promise<string> {
	const loader = messageFiles[`../data/messages/${path}.md`]
	if (!loader) return ''
	const raw = (await loader()) as string
	const vars: Record<string, string> = {
		...flattenStaticVariables(),
		...flattenProjectVariables(projectV2),
		...flattenProjectV3Variables(project),
	}
	for (const key of USER_CONTENT_KEYS) {
		if (key in vars) vars[key] = mdEscape(vars[key])
	}
	if (getVars) {
		for (const [k, v] of Object.entries(getVars(state))) {
			vars[k] = String(v ?? '')
		}
	}
	return expandVariables(raw, projectV2, project, vars)
}

function makeMessageFn(fn: (state: Record<string, NodeState>) => Promise<string>): MessageFn {
	const rich = fn as MessageFn
	rich.concat = (...others) =>
		makeMessageFn(async (state) =>
			(await Promise.all([rich, ...others].map((f) => f(state)))).join(''),
		)
	return rich
}

export function md(
	path: string | ((state: Record<string, NodeState>) => string),
	getVars?: (state: Record<string, NodeState>) => Record<string, NodeState>,
): MessageFn {
	const { projectV3: project, projectV2 } = injectProjectPageContext()
	return makeMessageFn(async (state) => {
		const resolvedPath = typeof path === 'function' ? path(state) : path
		return loadMd(resolvedPath, state, project.value, projectV2.value, getVars)
	})
}
// ─── Fix builder ──────────────────────────────────────────────────────────────

export function createTrackedPatch<T extends object>(
	source: T,
): { proxy: T; changes: () => Partial<T> } {
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
		changes: () =>
			Object.fromEntries([...written].map((k) => [k, data[k as keyof T]])) as Partial<T>,
	}
}

export class FixBuilder {
	_projectFn?: (
		patch: Labrinth.Projects.v3.EditProjectRequest,
		state: Record<string, NodeState>,
	) => void
	_versionFn?: (
		patch: Labrinth.Versions.v3.ModifyVersionRequest,
		state: Record<string, NodeState>,
	) => void

	project(
		fn: (patch: Labrinth.Projects.v3.EditProjectRequest, state: Record<string, NodeState>) => void,
	): this {
		this._projectFn = fn
		return this
	}

	version(
		fn: (
			patch: Labrinth.Versions.v3.ModifyVersionRequest,
			state: Record<string, NodeState>,
		) => void,
	): this {
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
	_autoMessage = false
	_autoMessageVars?: (state: Record<string, NodeState>) => Record<string, NodeState>
	_priority?: Priority
	_suggestedStatus?: ModerationStatus
	_severity?: ModerationSeverity
	_fixes: FixBuilder[] = []
	_applyFixes = false

	message(
		fn?: MessageFn | ((state: Record<string, NodeState>) => Record<string, NodeState>),
	): this {
		if (!fn) {
			this._autoMessage = true
		} else if (fn.constructor.name === 'AsyncFunction') {
			this._message = fn as MessageFn
		} else {
			this._autoMessage = true
			this._autoMessageVars = fn as (state: Record<string, NodeState>) => Record<string, NodeState>
		}
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
	_title?: Reactive<string>

	shown(condition: Reactive<boolean>): this {
		this._shown = condition
		return this
	}

	title(t: Reactive<string>): this {
		this._title = t
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
	_onClick?: (state: Record<string, NodeState>) => void
	_enabled?: Reactive<boolean> | ((state: Record<string, NodeState>) => boolean)

	constructor(nodeLabel: string) {
		super()
		this.label = nodeLabel
	}

	onClick(fn: (state: Record<string, NodeState>) => void): this {
		this._onClick = fn
		return this
	}

	enabled(condition: Reactive<boolean> | ((state: Record<string, NodeState>) => boolean)): this {
		this._enabled = condition
		return this
	}
}

export abstract class IdentifiedNodeBuilder extends NodeBuilder {
	readonly id: string | undefined
	_children: ChildEntry[] = []
	_childrenFn?: ChildrenFn
	_computingChildren = false
	_action?: ActionBuilder
	_enabled?: Reactive<boolean> | ((state: Record<string, NodeState>) => boolean)
	_statePath?: string[]

	constructor(id: string | undefined) {
		super()
		this.id = id
	}

	children(fn: ChildrenFn): this
	children(...entries: ChildEntry[]): this
	children(...args: [ChildrenFn] | ChildEntry[]): this {
		if (args.length === 1 && typeof args[0] === 'function' && args[0].length >= 1) {
			// Single function with ≥1 required param → ChildrenFn (e.g. (state) => [...])
			this._childrenFn = args[0] as ChildrenFn
		} else {
			this._children.push(...(args as ChildEntry[]))
		}
		return this
	}

	action(a: ActionBuilder): this {
		this._action = a
		return this
	}

	enabled(condition: Reactive<boolean> | ((state: Record<string, NodeState>) => boolean)): this {
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
	_defaultValue?: NodeState | ((state: Record<string, NodeState>) => NodeState)
	_required?: boolean

	initial(v: NodeState | ((state: Record<string, NodeState>) => NodeState)): this {
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

export class InputNodeBuilder extends IdentifiedNodeBuilder {
	readonly type: 'text' | 'markdown'
	_placeholder?: Reactive<string>
	_defaultValue?: NodeState | ((state: Record<string, NodeState>) => NodeState)
	_required?: boolean

	constructor(id: string, type: 'text' | 'markdown') {
		super(id)
		this.type = type
	}

	placeholder(p: Reactive<string>): this {
		this._placeholder = p
		return this
	}

	initial(v: NodeState | ((state: Record<string, NodeState>) => NodeState)): this {
		this._defaultValue = v
		return this
	}

	required(v = true): this {
		this._required = v
		return this
	}
}

export class GroupNodeBuilder extends IdentifiedNodeBuilder {
	readonly type = 'group' as const
	_layout?: 'flex' | 'column'
	_required?: boolean
	_selectMode?: 'single' | 'multi'
	_selectId?: string

	layout(l: 'flex' | 'column'): this {
		this._layout = l
		return this
	}

	required(v = true): this {
		this._required = v
		return this
	}

	singleSelect(id?: string): this {
		this._selectMode = 'single'
		if (id !== undefined) this._selectId = id
		return this
	}

	multiSelect(id?: string): this {
		this._selectMode = 'multi'
		if (id !== undefined) this._selectId = id
		return this
	}
}

export class DropdownNodeBuilder extends IdentifiedNodeBuilder {
	readonly type = 'dropdown' as const
	_none?: string

	override children(fn: ChildrenFn): this
	override children(...nodes: OptionNodeBuilder[]): this
	override children(...args: [ChildrenFn] | OptionNodeBuilder[]): this {
		if (args.length === 1 && typeof args[0] === 'function' && args[0].length >= 1) {
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
		this._icon = markRaw(i)
		return this
	}

	navigate(path: string): this {
		this._navigate = path
		return this
	}

	override children(fn: ChildrenFn): this
	override children(...entries: ChildEntry[]): this
	override children(...args: [ChildrenFn] | ChildEntry[]): this {
		if (args.length === 1 && typeof args[0] === 'function' && args[0].length >= 1) {
			super.children(args[0] as ChildrenFn)
			if (!this._statePath) this._statePath = [this.id!]
			return this
		}
		const entries = args as ChildEntry[]
		super.children(...entries)
		if (!this._statePath) this._statePath = [this.id!]
		stampChildPaths(entries, [this.id!])
		return this
	}
}

export const STAGES_KEY: InjectionKey<Ref<StageNodeBuilder[]>> = Symbol('checklistStages')
export const GLOBAL_STATE_KEY: InjectionKey<Ref<Record<string, Record<string, NodeState>>>> =
	Symbol('checklistGlobalState')

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

function stampChildPaths(entries: ChildEntry[], scopePath: string[]): void {
	for (const entry of entries) {
		// Skip reactive entries (refs, functions, nulls) — they get stamped on resolution
		if (!(entry instanceof NodeBuilder)) continue
		if (!(entry instanceof IdentifiedNodeBuilder)) continue
		const selectId = entry instanceof GroupNodeBuilder ? entry._selectId : undefined
		if (!entry.id && !selectId) {
			// Truly transparent: no structural id and no select id
			stampChildPaths(entry._children, scopePath)
			continue
		}
		if (!entry._statePath) {
			// Structural id contributes to path first, then select id
			const pathComponents = entry.id
				? selectId
					? [...scopePath, entry.id, selectId]
					: [...scopePath, entry.id]
				: [...scopePath, selectId!]
			entry._statePath = pathComponents
		}
		const childScope = childrenScopePath(entry)
		if (childScope) stampChildPaths(entry._children, childScope)
	}
}

export function isNodeActive(node: NodeBuilder, state: NodeState): boolean {
	switch (node.type) {
		case 'toggle':
		case 'check':
		case 'option': {
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
		case 'dropdown':
			return typeof state === 'string' && state !== ''
		case 'text':
		case 'markdown':
			return typeof state === 'string' && state !== ''
		default:
			return false
	}
}

export function getBooleanChildState(nodeState: NodeState): Record<string, NodeState> {
	if (nodeState && typeof nodeState === 'object' && !(nodeState instanceof Set)) {
		return nodeState as Record<string, NodeState>
	}
	return {}
}

export function resolveChildren(
	node: IdentifiedNodeBuilder,
	state: Record<string, NodeState>,
): NodeBuilder[] {
	let entries: ChildEntry[]
	let stampStatic = false

	if (node._childrenFn) {
		if (node._computingChildren) return []
		node._computingChildren = true
		try {
			entries = node._childrenFn(state)
			stampStatic = true // childrenFn results haven't been pre-stamped
		} finally {
			node._computingChildren = false
		}
	} else {
		entries = node._children
	}

	const scopePath = childrenScopePath(node)
	const result: NodeBuilder[] = []

	for (const entry of entries) {
		if (entry == null) continue
		if (entry instanceof NodeBuilder) {
			if (entry._shown !== undefined && !resolve(entry._shown)) continue
			if (stampStatic && scopePath) stampChildPaths([entry], scopePath)
			result.push(entry)
			continue
		}
		const resolved =
			typeof entry === 'function' ? entry(state) : toValue(entry as Ref<NodeBuilder | null>)
		if (resolved == null) continue
		if (resolved._shown !== undefined && !resolve(resolved._shown)) continue
		if (scopePath) stampChildPaths([resolved], scopePath)
		result.push(resolved)
	}

	return result
}

export function walkNodes(
	nodes: NodeBuilder[],
	stageState: Record<string, NodeState>,
	visitor: (
		node: IdentifiedNodeBuilder,
		nodeState: NodeState,
		localState: Record<string, NodeState>,
	) => void,
): void {
	for (const node of nodes) {
		if (node._shown !== undefined && !resolve(node._shown)) continue

		if (node.type === 'stage') {
			const identified = node as IdentifiedNodeBuilder
			if (identified.id) {
				const raw = stageState[identified.id]
				const childState =
					raw && typeof raw === 'object' && !(raw instanceof Set)
						? (raw as Record<string, NodeState>)
						: {}
				walkNodes(resolveChildren(identified, childState), childState, visitor)
			} else {
				walkNodes(resolveChildren(identified, stageState), stageState, visitor)
			}
			continue
		}

		if (node.type === 'group') {
			const g = node as GroupNodeBuilder
			if (!g._selectMode) {
				// Plain container: traverse children in group's own state scope
				if (g.id) {
					const raw = stageState[g.id]
					const childState =
						raw && typeof raw === 'object' && !(raw instanceof Set)
							? (raw as Record<string, NodeState>)
							: {}
					walkNodes(resolveChildren(g, childState), childState, visitor)
				} else {
					walkNodes(resolveChildren(g, stageState), stageState, visitor)
				}
				continue
			}
			// Fall through to value-node path for groups with selectMode
		}

		if (node.type === 'display' || node.type === 'button') continue

		const identified = node as IdentifiedNodeBuilder
		const stateKey =
			node.type === 'group'
				? ((node as GroupNodeBuilder)._selectId ?? identified.id!)
				: identified.id!
		const nodeState = stageState[stateKey]
		visitor(identified, nodeState, stageState)

		const active = isNodeActive(node, nodeState)
		const children = resolveChildren(identified, stageState)
		if (children.length === 0 || !active) continue

		if (node.type === 'group' && (node as GroupNodeBuilder)._selectMode === 'multi') {
			const selected = nodeState instanceof Set ? nodeState : new Set<string>()
			for (const child of children) {
				const childId = child as IdentifiedNodeBuilder
				if (!selected.has(childId.id!) || (child._shown !== undefined && !resolve(child._shown)))
					continue
				const rawChildState = stageState[childId.id!]
				// Option children are active by virtue of being in the selected Set, not by their own
				// boolean state. If they have child state but no explicit value, inject value: true
				// so isNodeActive returns true even after child state has been written to their path.
				const childState: NodeState =
					rawChildState !== null &&
					rawChildState !== undefined &&
					typeof rawChildState === 'object' &&
					!(rawChildState instanceof Set) &&
					(rawChildState as NodeStateWithChildren).value === undefined
						? { ...(rawChildState as NodeStateWithChildren), value: true }
						: (rawChildState ?? true)
				visitor(childId, childState, stageState)
				walkNodes(resolveChildren(childId, stageState), stageState, visitor)
			}
		} else if (node.type === 'toggle' || node.type === 'check') {
			const childState = getBooleanChildState(nodeState)
			walkNodes(children, childState, visitor)
		} else if (
			(node.type === 'group' && (node as GroupNodeBuilder)._selectMode === 'single') ||
			node.type === 'dropdown'
		) {
			const selectedId = typeof nodeState === 'string' ? nodeState : undefined
			if (selectedId) {
				for (const child of children) {
					const childId = child as IdentifiedNodeBuilder
					if (childId.id !== selectedId || (child._shown !== undefined && !resolve(child._shown)))
						continue
					visitor(childId, stageState[childId.id!], stageState)
					walkNodes(resolveChildren(childId, stageState), stageState, visitor)
					break
				}
			}
		} else {
			walkNodes(children, stageState, visitor)
		}
	}
}

// ─── Factory functions ────────────────────────────────────────────────────────

export type StageFn = (
	project: Labrinth.Projects.v3.Project,
	projectV2: Labrinth.Projects.v2.Project,
) => StageNodeBuilder

export function stageFn(factory: StageFn): StageFn {
	const cache = new WeakMap<object, StageNodeBuilder>()
	return (project, projectV2) => {
		if (!cache.has(project)) cache.set(project, factory(project, projectV2))
		return cache.get(project)!
	}
}

export const toggle = (id: string, nodeLabel: string) =>
	new BooleanNodeBuilder(id, nodeLabel, 'toggle')
export const check = (id: string, nodeLabel: string) =>
	new BooleanNodeBuilder(id, nodeLabel, 'check')
export const button = (nodeLabel: string) => new ButtonNodeBuilder(nodeLabel)
export const text = (id: string) => new InputNodeBuilder(id, 'text')
export const markdown = (id: string) => new InputNodeBuilder(id, 'markdown')
export const group = (id?: string) => new GroupNodeBuilder(id)
export const dropdown = (id: string) => new DropdownNodeBuilder(id)
export const option = (id: string, nodeLabel: string) => new OptionNodeBuilder(id, nodeLabel)
export const label = (content: string | ContentFn) => new DisplayNodeBuilder(content)
export const stage = (id: string, title: string) => new StageNodeBuilder(id, title)
