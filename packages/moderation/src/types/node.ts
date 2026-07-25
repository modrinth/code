import type { Labrinth } from '@modrinth/api-client'
import { Checkbox, MarkdownEditor, StyledInput, Toggle } from '@modrinth/ui'
import type { Component, FunctionalComponent, InjectionKey, Ref, SVGAttributes } from 'vue'
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

export type ChildNode = NodeBuilder | (() => unknown) | string

export type ChildEntry =
	| NodeBuilder
	| string
	| (() => unknown)
	| null
	| Ref<NodeBuilder | null>
	| ((state?: Record<string, NodeState>) => NodeBuilder | NodeBuilder[] | null)

export type NodeType =
	| 'toggle'
	| 'check'
	| 'switch'
	| 'button'
	| 'text'
	| 'markdown'
	| 'group'
	| 'dropdown'
	| 'stage'
	| 'app-component'

// ─── Message helpers ──────────────────────────────────────────────────────────

const messageFiles = import.meta.glob('../data/messages/**/*.md', {
	query: '?raw',
	import: 'default',
})

export function mdOptional(
	path: string,
	getVars?: (state: Record<string, any>) => Record<string, any>,
): MessageFn {
	return makeMessageFn(async (state) => {
		const loader = messageFiles[`../data/messages/${path}.md`]
		if (!loader) return ''
		return loadMd(path, state, _project!.value, _projectV2!.value, getVars)
	})
}

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
	getVars?: (state: Record<string, any>) => Record<string, any>,
): Promise<string> {
	// Call getVars before any await so Vue's watchEffect tracks reactive reads inside it
	const extraVars = getVars ? getVars(state) : null
	const loader = messageFiles[`../data/messages/${path}.md`]
	if (!loader) {
		_onMissingMd?.(path)
		return ''
	}
	const raw = (await loader()) as string
	const vars: Record<string, string> = {
		...flattenStaticVariables(),
		...flattenProjectVariables(projectV2),
		...flattenProjectV3Variables(project),
	}
	for (const key of USER_CONTENT_KEYS) {
		if (key in vars) vars[key] = mdEscape(vars[key])
	}
	if (extraVars) {
		for (const [k, v] of Object.entries(extraVars)) {
			vars[k] = String(v ?? '')
		}
	}
	const expanded = expandVariables(raw, projectV2, project, vars)
	// Code spans render literally — markdown escapes inside them show as-is, so strip them
	return expanded.replace(/`[^`\n]*`/g, (match) => match.replace(/\\([\\*_`[~])/g, '$1'))
}

function makeMessageFn(fn: (state: Record<string, NodeState>) => Promise<string>): MessageFn {
	const rich = fn as MessageFn
	rich.concat = (...others) =>
		makeMessageFn(async (state) =>
			(await Promise.all([rich, ...others].map((f) => f(state)))).join(''),
		)
	return rich
}

let _project: Ref<Labrinth.Projects.v3.Project> | null = null
let _projectV2: Ref<Labrinth.Projects.v2.Project> | null = null
let _onMissingMd: ((path: string) => void) | null = null

export function setMissingMdHandler(handler: (path: string) => void) {
	_onMissingMd = handler
}

export function setMessageProject(
	project: Ref<Labrinth.Projects.v3.Project>,
	projectV2: Ref<Labrinth.Projects.v2.Project>,
) {
	_project = project
	_projectV2 = projectV2
}

export function md(
	path: string | ((state: Record<string, NodeState>) => string),
	getVars?: (state: Record<string, any>) => Record<string, any>,
): MessageFn {
	return makeMessageFn(async (state) => {
		const resolvedPath = typeof path === 'function' ? path(state) : path
		return loadMd(resolvedPath, state, _project!.value, _projectV2!.value, getVars)
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

// ─── Message segments ─────────────────────────────────────────────────────────

type GetVarsFn = (state: Record<string, any>) => Record<string, any>

export type MessageSegment =
	| { type: 'fn'; fn: ContentFn }
	| { type: 'auto'; getVars?: GetVarsFn }
	| { type: 'path'; path: string | (() => string); getVars?: GetVarsFn }
	| { type: 'collect'; fallback?: MessageSegment }

export function resolveRelativeMessagePath(
	messagePath: string | (() => string),
	statePath: string[],
): string {
	const name = typeof messagePath === 'function' ? messagePath() : messagePath
	if (name.startsWith('/')) return `checklist/messages${name}`
	const parts = [...statePath.slice(0, -1), ...name.split('/')]
	const normalized = parts.reduce<string[]>((acc, p) => {
		if (p === '..') acc.pop()
		else if (p) acc.push(p)
		return acc
	}, [])
	return `checklist/messages/${normalized.join('/')}`
}

export async function evalSegment(
	seg: MessageSegment,
	state: Record<string, NodeState>,
	statePath: string[],
): Promise<string> {
	if (seg.type === 'collect') return ''
	if (seg.type === 'fn') return String((await seg.fn(state)) ?? '')
	if (seg.type === 'auto')
		return loadMd(
			`checklist/messages/${statePath.join('/')}`,
			state,
			_project!.value,
			_projectV2!.value,
			seg.getVars,
		)
	return loadMd(
		resolveRelativeMessagePath(seg.path, statePath),
		state,
		_project!.value,
		_projectV2!.value,
		seg.getVars,
	)
}

// ─── Node builders ────────────────────────────────────────────────────────────

export abstract class NodeBuilder {
	abstract readonly type: NodeType
	_shown?: Reactive<boolean>
	_title?: Reactive<string>
	_icon?: FunctionalComponent<SVGAttributes>
	_tooltip?: Reactive<string> | ((state: Record<string, NodeState>) => string)

	shown(condition: Reactive<boolean>): this {
		this._shown = condition
		return this
	}

	title(t: Reactive<string>): this {
		this._title = t
		return this
	}

	icon(i: FunctionalComponent<SVGAttributes>): this {
		this._icon = markRaw(i)
		return this
	}

	tooltip(t: Reactive<string> | ((state: Record<string, NodeState>) => string)): this {
		this._tooltip = t
		return this
	}
}

export class ButtonNodeBuilder extends NodeBuilder {
	readonly type = 'button' as const
	readonly label?: string
	_onClick?: (state: Record<string, NodeState>) => void
	_enabled?: Reactive<boolean> | ((state: Record<string, NodeState>) => boolean)

	constructor(nodeLabel?: string) {
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
	_segments: MessageSegment[] = []
	_suggestedStatus?: ModerationStatus
	_severity?: ModerationSeverity
	_priority?: Priority
	_fixes: FixBuilder[] = []
	_applyFixes = false
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
			this._childrenFn = args[0] as ChildrenFn
		} else {
			this._children.push(...(args as ChildEntry[]))
		}
		return this
	}

	message(path?: string | (() => string), getVars?: GetVarsFn): this {
		const segment: MessageSegment =
			path === undefined
				? { type: 'auto', ...(getVars && { getVars }) }
				: { type: 'path', path, ...(getVars && { getVars }) }
		this._segments.push(segment)
		return this
	}

	rawMessage(content: string | ContentFn): this {
		const fn: ContentFn = typeof content === 'string' ? () => content : content
		this._segments.push({ type: 'fn', fn })
		return this
	}

	collect(path?: string | (() => string), getVars?: GetVarsFn): this {
		const fallback =
			path !== undefined || getVars !== undefined
				? path === undefined
					? { type: 'auto' as const, ...(getVars && { getVars }) }
					: { type: 'path' as const, path, ...(getVars && { getVars }) }
				: undefined
		this._segments.push({ type: 'collect', ...(fallback && { fallback }) })
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

	priority(p: Priority): this {
		this._priority = p
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

/**
 * A click-triggered, icon-capable status button (e.g. "flag this issue") — not a plain value
 * control, so unlike `check`/`toggleSwitch` (`BooleanComponentNodeBuilder`) it isn't built on
 * `ComponentNodeBuilder` and keeps its own dedicated `NodeRenderer` rendering.
 */
export class BooleanNodeBuilder extends ValueNodeBuilder {
	readonly type = 'toggle' as const
}

export type OverrideValue = { readonly __override: string }

export type OnChangeFn = (
	value: string,
	helpers: { override: (value: string) => OverrideValue },
) => OverrideValue | void

export interface ComponentNodePropsContext {
	onImageUpload?: (file: File) => Promise<string>
	/** For `_valueKind: 'set'` nodes — toggles one value in/out of the underlying `Set`. State
	 *  mutation only exists in `NodeRenderer` (state lives via Vue's provide/inject, not
	 *  something `node.ts` can reach), so this is how a `.props()` closure reaches it — map it to
	 *  whatever prop name the actual mounted component expects (e.g. `toggleLoader`). */
	toggleSetValue?: (value: string) => void
}

/**
 * Base for any node whose value is edited by mounting a real Vue component rather than
 * something `NodeRenderer` has its own hardcoded markup for. `NodeRenderer` renders every
 * instance of this the same generic way (`<component :is="node._component" .../>`), so a new
 * component-backed node type never needs a `NodeRenderer` change — just a subclass (or, for a
 * one-off, a direct construction) that fills in `_component` and how its value binds.
 */
export abstract class ComponentNodeBuilder extends IdentifiedNodeBuilder {
	_component?: Component
	/** v-model prop name; the paired event is `update:${_modelProp}`. Defaults to Vue's own convention. */
	_modelProp: string = 'modelValue'
	_componentProps?: (ctx: ComponentNodePropsContext) => Record<string, unknown>
	/** Extra props layered on top of `_componentProps`, e.g. `.props(() => ({ type: 'email' }))` —
	 *  the escape hatch into whatever the mounted component actually supports, without needing a
	 *  dedicated builder method for every prop it happens to have. */
	_extraProps?: (ctx: ComponentNodePropsContext) => Record<string, unknown>
	_showTooltip?: boolean
	/** Needs the ref-based setValue()/button-resync DOM-sync workaround — see `NodeRenderer`'s
	 *  `componentRefs`. Controlled-input-style components typically need this; components that
	 *  sync themselves off their own `modelValue` watcher (like `MarkdownEditor`) don't. */
	_imperativeSync?: boolean
	/** What shape this node's value is, so `NodeRenderer`/`isNodeActive`/`childrenScopePath` know
	 *  how to read/write/interpret it generically (`'string'`, `'boolean'`, or `'set'` for a
	 *  multi-value picker) without needing a dedicated case added per node type. */
	_valueKind: 'string' | 'boolean' | 'set' = 'string'

	props(fn: (ctx: ComponentNodePropsContext) => Record<string, unknown>): this {
		this._extraProps = fn
		return this
	}

	valueKind(kind: 'string' | 'boolean' | 'set'): this {
		this._valueKind = kind
		return this
	}

	modelProp(name: string): this {
		this._modelProp = name
		return this
	}
}

/** A component-backed node builder, plus a callable setter for any prop name that isn't
 *  already a real method/property on it. */
export type WithAutoProps<T> = T & { [key: string]: (value: unknown) => T }

/**
 * Lets a component-backed node builder be called like `.clearable(true)`/`.size('small')` for
 * any prop the mounted component actually accepts, without a dedicated method for each one —
 * same underlying mechanism as `.props()`, just spelled as the prop's own name. Falls through to
 * the real method/property untouched whenever one already exists with that name (e.g. `.icon()`
 * from `NodeBuilder`, or `.type` on `InputNodeBuilder`), so this only ever fills genuine gaps.
 */
function withAutoProps<T extends ComponentNodeBuilder>(builder: T): WithAutoProps<T> {
	return new Proxy(builder, {
		get(target, prop, receiver) {
			if (typeof prop === 'string' && !(prop in target)) {
				return (value: unknown) => {
					const prev = target._extraProps
					target._extraProps = (ctx) => ({ ...prev?.(ctx), [prop]: value })
					return receiver
				}
			}
			return Reflect.get(target, prop, receiver)
		},
	}) as WithAutoProps<T>
}

export class InputNodeBuilder extends ComponentNodeBuilder {
	readonly type: 'text' | 'markdown'
	_placeholder?: Reactive<string>
	_defaultValue?: NodeState | ((state: Record<string, NodeState>) => NodeState)
	_required?: boolean
	_onChange?: OnChangeFn

	constructor(id: string, type: 'text' | 'markdown') {
		super(id)
		this.type = type
		if (type === 'text') {
			this._component = markRaw(StyledInput)
			this._showTooltip = true
			this._imperativeSync = true
			this._componentProps = () => ({ class: 'min-w-40 flex-1', autocomplete: 'off' })
		} else {
			this._component = markRaw(MarkdownEditor)
			this._componentProps = (ctx) => ({
				maxHeight: 300,
				disabled: false,
				headingButtons: false,
				onImageUpload: ctx.onImageUpload,
			})
		}
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

	onChange(fn: OnChangeFn): this {
		this._onChange = fn
		return this
	}
}

/**
 * A plain boolean field — `check` renders as a `Checkbox`, `switch` (exposed as the
 * `toggleSwitch()` factory, since `switch` is a reserved word) as the `Toggle` slider. Distinct
 * from `toggle()`/`BooleanNodeBuilder`, which is a click-triggered, icon-capable status button
 * with its own color-coding logic — not a plain value control, so it stays on its own path.
 */
export class BooleanComponentNodeBuilder extends ComponentNodeBuilder {
	readonly type: 'check' | 'switch'
	label: string
	_defaultValue?: NodeState | ((state: Record<string, NodeState>) => NodeState)
	_required?: boolean

	constructor(id: string, label: string, type: 'check' | 'switch') {
		super(id)
		this.type = type
		this.label = label
		this._valueKind = 'boolean'
		if (type === 'check') {
			this._component = markRaw(Checkbox)
			this._componentProps = () => ({ label })
		} else {
			this._component = markRaw(Toggle)
		}
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

/**
 * A component-backed node whose actual Vue component can't be referenced from `node.ts` — it's
 * app-specific (e.g. lives in `apps/frontend`), not part of `@modrinth/ui`, which is all
 * `node.ts` can import. `NodeRenderer` resolves the real component itself from `_rendererKey`
 * instead of `_component` being set here. Construct directly for a one-off use; wrap in a
 * dedicated factory/subclass (mirroring `InputNodeBuilder`/`BooleanComponentNodeBuilder`) if it
 * ends up reused.
 */
export class AppComponentNodeBuilder extends ComponentNodeBuilder {
	readonly type = 'app-component' as const
	_rendererKey: string
	_defaultValue?: NodeState | ((state: Record<string, NodeState>) => NodeState)

	constructor(id: string, rendererKey: string) {
		super(id)
		this._rendererKey = rendererKey
	}

	initial(v: NodeState | ((state: Record<string, NodeState>) => NodeState)): this {
		this._defaultValue = v
		return this
	}
}

export class GroupNodeBuilder extends IdentifiedNodeBuilder {
	readonly type = 'group' as const
	_layout?: 'flex' | 'column'
	_required?: boolean
	_selectMode?: 'single' | 'multi'
	_selectId?: string
	_defaultValue?: NodeState | ((state: Record<string, NodeState>) => NodeState)

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

	/** Selected option(s) to start with before the moderator has touched this group. */
	initial(v: NodeState | ((state: Record<string, NodeState>) => NodeState)): this {
		this._defaultValue = v
		return this
	}
}

export class DropdownNodeBuilder extends IdentifiedNodeBuilder {
	readonly type = 'dropdown' as const
	_none?: string

	override children(fn: ChildrenFn): this
	override children(...nodes: IdentifiedNodeBuilder[]): this
	override children(...args: [ChildrenFn] | IdentifiedNodeBuilder[]): this {
		if (args.length === 1 && typeof args[0] === 'function' && args[0].length >= 1) {
			super.children(args[0] as ChildrenFn)
		} else {
			super.children(...(args as IdentifiedNodeBuilder[]))
		}
		return this
	}

	none(text: string): this {
		this._none = text
		return this
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
	// Boolean-valued component nodes (check/switch/any future one) nest state under their own
	// key the same way `toggle` does — other value kinds (string, set) don't have "revealed
	// children" today, so this generalizes instead of needing a case added per node type.
	if (node instanceof ComponentNodeBuilder) {
		return node._valueKind === 'boolean' ? node._statePath : null
	}
	switch (node.type) {
		case 'toggle':
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

function isBooleanStateActive(state: NodeState): boolean {
	if (typeof state === 'boolean') return state
	if (state && typeof state === 'object' && !(state instanceof Set)) {
		const v = (state as NodeStateWithChildren).value
		if (typeof v === 'boolean') return v
	}
	return false
}

/**
 * Whether a node counts as "active" given its resolved state. `state` is expected to already
 * reflect the node's default (see `withDefaults`) when nothing's been explicitly written —
 * this only interprets the value, it doesn't apply any defaulting of its own.
 */
export function isNodeActive(node: NodeBuilder, state: NodeState): boolean {
	// Component-backed nodes (text/markdown/check/switch/any future one) are dispatched by
	// their declared value shape instead of by type, so a new node type never needs a case
	// added here — only a genuinely new *value shape* would.
	if (node instanceof ComponentNodeBuilder) {
		if (node._valueKind === 'boolean') return isBooleanStateActive(state)
		if (node._valueKind === 'set') return state instanceof Set && state.size > 0
		return typeof state === 'string' && state !== ''
	}
	switch (node.type) {
		case 'toggle':
			return isBooleanStateActive(state)
		case 'group': {
			const g = node as GroupNodeBuilder
			if (g._selectMode === 'single') return typeof state === 'string' && state !== ''
			if (g._selectMode === 'multi') return state instanceof Set && state.size > 0
			return false
		}
		case 'dropdown':
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
): ChildNode[] {
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
	const result: ChildNode[] = []

	for (const entry of entries) {
		if (entry == null) continue
		if (entry instanceof NodeBuilder) {
			if (entry._shown !== undefined && !resolve(entry._shown)) continue
			if (stampStatic && scopePath) stampChildPaths([entry], scopePath)
			result.push(entry)
			continue
		}
		if (typeof entry === 'string') {
			result.push(entry)
			continue
		}
		if (typeof entry === 'function') {
			if (entry.length === 0) {
				result.push(entry as () => unknown)
				continue
			}
			const resolved = entry(state)
			if (resolved == null) continue
			if (Array.isArray(resolved)) {
				for (const r of resolved) {
					if (r == null) continue
					if (r._shown !== undefined && !resolve(r._shown)) continue
					if (scopePath) stampChildPaths([r], scopePath)
					result.push(r)
				}
				continue
			}
			if (resolved._shown !== undefined && !resolve(resolved._shown)) continue
			if (scopePath) stampChildPaths([resolved], scopePath)
			result.push(resolved)
			continue
		}
		// Ref<NodeBuilder | null>
		const resolved = toValue(entry as Ref<NodeBuilder | null>)
		if (resolved == null) continue
		if (resolved._shown !== undefined && !resolve(resolved._shown)) continue
		if (scopePath) stampChildPaths([resolved], scopePath)
		result.push(resolved)
	}

	return result
}

export function walkNodes(
	nodes: ChildNode[],
	stageState: Record<string, NodeState>,
	visitor: (
		node: IdentifiedNodeBuilder,
		nodeState: NodeState,
		localState: Record<string, NodeState>,
	) => void,
): void {
	// Consumer-facing state (fix/message segments) gets defaulting-aware reads; the
	// traversal below stays on raw `stageState` for structural decisions (isNodeActive,
	// resolveChildren) so defaults never change what gets walked, only what's read.
	const scopedState = withDefaults(stageState, nodes)

	for (const node of nodes) {
		if (!(node instanceof NodeBuilder)) continue
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

		if (node.type === 'button') continue

		const identified = node as IdentifiedNodeBuilder
		const stateKey =
			node.type === 'group'
				? ((node as GroupNodeBuilder)._selectId ?? identified.id!)
				: identified.id!
		// Defaulting-aware: isNodeActive and the visitor callback (fix/message evaluation) should
		// see a field's default when nothing's been written yet. Structural recursion below stays
		// on the raw value so descending into children never depends on what a default resolves to.
		const rawNodeState = stageState[stateKey]
		const nodeState = scopedState[stateKey]
		visitor(identified, nodeState, scopedState)

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
				visitor(childId, childState, scopedState)
				const nestedState = getBooleanChildState(rawChildState)
				walkNodes(resolveChildren(childId, nestedState), nestedState, visitor)
			}
		} else if (
			node.type === 'toggle' ||
			(node instanceof ComponentNodeBuilder && node._valueKind === 'boolean')
		) {
			const childState = getBooleanChildState(rawNodeState)
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
					const rawChildState = stageState[childId.id!]
					const childState: NodeState =
						rawChildState !== null &&
						rawChildState !== undefined &&
						typeof rawChildState === 'object' &&
						!(rawChildState instanceof Set) &&
						(rawChildState as NodeStateWithChildren).value === undefined
							? { ...(rawChildState as NodeStateWithChildren), value: true }
							: (rawChildState ?? true)
					visitor(childId, childState, scopedState)
					const nestedState = getBooleanChildState(rawChildState)
					walkNodes(resolveChildren(childId, nestedState), nestedState, visitor)
					break
				}
			}
		} else {
			walkNodes(children, stageState, visitor)
		}
	}
}

// ─── Self-defaulting state ────────────────────────────────────────────────────

export function resolveDefault(
	node: { _defaultValue?: NodeState | ((state: Record<string, NodeState>) => NodeState) },
	state: Record<string, NodeState>,
): NodeState {
	const d = node._defaultValue
	return typeof d === 'function' ? d(state) : d
}

const defaultingProxyCache = new WeakMap<object, object>()

/**
 * Id-less nodes (e.g. an unnamed `group()` used purely for layout/title) don't nest state
 * under their own key, so their children read/write directly into this same scope — flatten
 * through them so those grandchildren are still reachable for defaulting purposes. A select-mode
 * group without a structural id still owns a real key (its `_selectId`), same as `walkNodes`'
 * own stateKey resolution — only flatten when neither is present.
 */
function buildChildMap(
	children: ChildNode[],
	stateForFlattening: Record<string, NodeState>,
): Map<string, IdentifiedNodeBuilder> {
	const childMap = new Map<string, IdentifiedNodeBuilder>()
	function collect(nodes: ChildNode[]) {
		for (const child of nodes) {
			if (!(child instanceof IdentifiedNodeBuilder)) continue
			const selectId = child instanceof GroupNodeBuilder ? child._selectId : undefined
			const key = child.id ?? selectId
			if (key) {
				childMap.set(key, child)
			} else {
				collect(resolveChildren(child, stateForFlattening))
			}
		}
	}
	collect(children)
	return childMap
}

/**
 * A dropdown's or select-mode group's "children" are its enumerable options, not nested
 * sub-fields of a container — chaining into them as an empty scope (or a `withDefaults`-wrapped
 * nested object) would be wrong, and hands back a Proxy where callers expect the option id
 * itself. Only plain groups/toggles/checks actually nest state under their own key.
 */
function isOptionsContainer(child: IdentifiedNodeBuilder): boolean {
	return (
		child instanceof DropdownNodeBuilder ||
		(child instanceof GroupNodeBuilder && child._selectMode !== undefined)
	)
}

/**
 * A stand-in for a container that hasn't been written to yet (e.g. a group or toggle
 * nobody has touched), so reads can chain arbitrarily deep without checking existence
 * first. Reads resolve defaults the same way `withDefaults` does. A write cascades back
 * up through `writeSelf`, materializing every not-yet-real ancestor along the way as a
 * single patch on the nearest real object — nothing is written until something actually is.
 */
function emptyScope(
	children: ChildNode[],
	writeSelf: (patch: Record<string, NodeState>) => void,
): Record<string, NodeState> {
	const childMap = buildChildMap(children, {})

	const resolving = new Set<string>()
	const emptyCache = new Map<string, Record<string, NodeState>>()

	const proxy: Record<string, NodeState> = new Proxy(
		{},
		{
			get(_target, key) {
				if (typeof key !== 'string') return undefined
				const child = childMap.get(key)
				if (!child) return undefined

				if (!resolving.has(key)) {
					resolving.add(key)
					try {
						const def = resolveDefault(child, proxy)
						if (def !== undefined) return def
					} finally {
						resolving.delete(key)
					}
				}

				if (isOptionsContainer(child)) return undefined

				const cached = emptyCache.get(key)
				if (cached) return cached

				const grandchildren = resolveChildren(child, {})
				if (grandchildren.length === 0) return undefined

				const nested = emptyScope(grandchildren, (patch) => writeSelf({ [key]: patch }))
				emptyCache.set(key, nested)
				return nested
			},
			set(_target, key, value) {
				if (typeof key === 'string') writeSelf({ [key]: value as NodeState })
				return true
			},
			deleteProperty() {
				return true
			},
		},
	) as Record<string, NodeState>

	return proxy
}

/**
 * Wraps a state object so reading an unset key transparently returns that field's
 * `.initial()` default instead of `undefined`, without needing to store it, and so
 * reading into a container nobody has touched yet (a group/toggle with no state at all)
 * returns a safely-chainable empty scope instead of `undefined`. Writes pass through to
 * the real underlying object, materializing not-yet-real containers the first time
 * something is actually written into them. Nested objects are wrapped lazily on read,
 * scoped to whichever child's own children apply at that depth, so the wrap only needs
 * to happen once at the top of a scope for defaulting to work at any depth beneath it.
 */
export function withDefaults<T extends Record<string, NodeState>>(
	rawState: T,
	children: ChildNode[],
): T {
	if (rawState == null || typeof rawState !== 'object' || rawState instanceof Set) return rawState

	const cached = defaultingProxyCache.get(rawState)
	if (cached) return cached as T

	const childMap = buildChildMap(children, rawState)

	// Guards a default function that reads its own key from recursing into itself forever.
	const resolving = new Set<string>()
	const emptyCache = new Map<string, Record<string, NodeState>>()

	const proxy = new Proxy(rawState, {
		get(target, key, receiver) {
			if (typeof key !== 'string') return Reflect.get(target, key, receiver)
			const child = childMap.get(key)
			let value = Reflect.get(target, key, receiver)

			if (value === undefined && child !== undefined && !resolving.has(key)) {
				resolving.add(key)
				try {
					value = resolveDefault(child, proxy)
				} finally {
					resolving.delete(key)
				}
			}

			if (
				child !== undefined &&
				value !== null &&
				typeof value === 'object' &&
				!(value instanceof Set)
			) {
				const nested = value as Record<string, NodeState>
				return withDefaults(nested, resolveChildren(child, nested))
			}

			if (value === undefined && child !== undefined && !isOptionsContainer(child)) {
				const cachedEmpty = emptyCache.get(key)
				if (cachedEmpty) return cachedEmpty

				const grandchildren = resolveChildren(child, {})
				if (grandchildren.length > 0) {
					const nested = emptyScope(grandchildren, (patch) => {
						Reflect.set(target, key, patch, receiver)
					})
					emptyCache.set(key, nested)
					return nested
				}
			}

			return value
		},
		set(target, key, value, receiver) {
			return Reflect.set(target, key, value, receiver)
		},
		deleteProperty(target, key) {
			return Reflect.deleteProperty(target, key)
		},
	}) as T

	defaultingProxyCache.set(rawState, proxy)
	return proxy
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

export const toggle = (id: string, nodeLabel: string) => new BooleanNodeBuilder(id, nodeLabel)
export const check = (id: string, nodeLabel: string) =>
	withAutoProps(new BooleanComponentNodeBuilder(id, nodeLabel, 'check'))
export const toggleSwitch = (id: string, nodeLabel: string) =>
	withAutoProps(new BooleanComponentNodeBuilder(id, nodeLabel, 'switch'))
export const button = (nodeLabel?: string) => new ButtonNodeBuilder(nodeLabel)
export const text = (id: string) => withAutoProps(new InputNodeBuilder(id, 'text'))
export const markdown = (id: string) => withAutoProps(new InputNodeBuilder(id, 'markdown'))
export const group = (id?: string) => new GroupNodeBuilder(id)
export const dropdown = (id: string) => new DropdownNodeBuilder(id)
export const appComponent = (id: string, rendererKey: string) =>
	withAutoProps(new AppComponentNodeBuilder(id, rendererKey))
export const stage = (id: string, title: string) => new StageNodeBuilder(id, title)
