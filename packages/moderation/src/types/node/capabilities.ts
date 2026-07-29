import type { Component, FunctionalComponent, SVGAttributes } from 'vue'

import type { FixBuilder } from './fix'
import { Priority } from '../priority.ts'
import type {
	GetVarsFn,
	MessageSegment,
	ModerationStatus,
	NodeState,
	Reactive,
} from './state'

export interface Visible {
	_shown: Reactive<boolean> | undefined
	shown(this: this, condition: Reactive<boolean>): this
}

export function withShown<T extends object>(node: T): T & Visible {
	return Object.assign(node, {
		_shown: undefined as Reactive<boolean> | undefined,
		shown(this: any, condition: Reactive<boolean>) {
			this._shown = condition
			return this
		},
	})
}

export interface Iconable {
	_icon: FunctionalComponent<SVGAttributes> | undefined
	icon(this: this, icon: FunctionalComponent<SVGAttributes> | undefined): this
}

export function withIcon<T extends object>(node: T): T & Iconable {
	return Object.assign(node, {
		_icon: undefined as FunctionalComponent<SVGAttributes> | undefined,
		icon(this: any, icon: FunctionalComponent<SVGAttributes> | undefined) {
			this._icon = icon
			return this
		},
	})
}

export interface Tooltippable {
	_tooltip: Reactive<string> | ((state: Record<string, NodeState>) => string) | undefined
	tooltip(
		this: this,
		tooltip: Reactive<string> | ((state: Record<string, NodeState>) => string) | undefined,
	): this
}

export function withTooltip<T extends object>(node: T): T & Tooltippable {
	return Object.assign(node, {
		_tooltip: undefined as Tooltippable['_tooltip'],
		tooltip(this: any, tooltip: Tooltippable['_tooltip']) {
			this._tooltip = tooltip
			return this
		},
	})
}

export interface Titled {
	_title: Reactive<string> | undefined
	title(this: this, title: Reactive<string>): this
}

export function withTitle<T extends object>(node: T): T & Titled {
	return Object.assign(node, {
		_title: undefined as Reactive<string> | undefined,
		title(this: any, title: Reactive<string>) {
			this._title = title
			return this
		},
	})
}

export interface Identified {
	id: string
	_statePath?: string[]
	statePath(this: this, path: string[]): this
}

export function withId<T extends object>(node: T, id: string): T & Identified {
	return Object.assign(node, {
		id,
		statePath(this: any, path: string[]) {
			this._statePath = path
			return this
		},
	})
}

export interface Prioritizable {
	_priority: Priority
	priority(this: this, p: Priority): this
}

export function withPriority<T extends object>(node: T): T & Prioritizable {
	return Object.assign(node, {
		_priority: new Priority(),
		priority(this: any, p: Priority) {
			this._priority = p
			return this
		},
	})
}

export interface Messageable {
	_segments: MessageSegment[]
	_suggestedStatus?: ModerationStatus
	suggestedStatus(this: this, status: ModerationStatus): this
	message(this: this, path?: string | (() => string) | GetVarsFn, getVars?: GetVarsFn): this
	rawMessage(
		this: this,
		content: string | ((state: Record<string, NodeState>) => string | Promise<string>),
	): this
	collect(this: this, path?: string | (() => string) | GetVarsFn, getVars?: GetVarsFn): this
}

function splitPathAndVars(
	pathOrGetVars: string | (() => string) | GetVarsFn | undefined,
	getVars: GetVarsFn | undefined,
): { path: string | (() => string) | undefined; getVars: GetVarsFn | undefined } {
	if (typeof pathOrGetVars === 'function' && pathOrGetVars.length >= 1) {
		return { path: undefined, getVars: pathOrGetVars as GetVarsFn }
	}
	return { path: pathOrGetVars as string | (() => string) | undefined, getVars }
}

export function withMessaging<T extends object>(node: T): T & Messageable {
	return Object.assign(node, {
		_segments: [] as MessageSegment[],
		suggestedStatus(this: any, status: ModerationStatus) {
			this._suggestedStatus = status
			return this
		},
		message(this: any, pathOrGetVars?: string | (() => string) | GetVarsFn, getVarsArg?: GetVarsFn) {
			const { path, getVars } = splitPathAndVars(pathOrGetVars, getVarsArg)
			const segment: MessageSegment =
				path === undefined
					? { type: 'auto', ...(getVars && { getVars }) }
					: { type: 'path', path, ...(getVars && { getVars }) }
			this._segments.push(segment)
			return this
		},
		rawMessage(
			this: any,
			content: string | ((state: Record<string, NodeState>) => string | Promise<string>),
		) {
			const fn = typeof content === 'string' ? () => content : content
			this._segments.push({ type: 'fn', fn })
			return this
		},
		collect(this: any, pathOrGetVars?: string | (() => string) | GetVarsFn, getVarsArg?: GetVarsFn) {
			const { path, getVars } = splitPathAndVars(pathOrGetVars, getVarsArg)
			const fallback: MessageSegment | undefined =
				path !== undefined || getVars !== undefined
					? path === undefined
						? { type: 'auto', ...(getVars && { getVars }) }
						: { type: 'path', path, ...(getVars && { getVars }) }
					: undefined
			this._segments.push({ type: 'collect', ...(fallback && { fallback }) })
			return this
		},
	})
}

export interface Requireable {
	_required: boolean | undefined
	required(this: this, v?: boolean): this
}

export function withRequired<T extends object>(node: T): T & Requireable {
	return Object.assign(node, {
		_required: undefined as boolean | undefined,
		required(this: any, v = true) {
			this._required = v
			return this
		},
	})
}

export interface Fixable {
	_fixes: FixBuilder[]
	_applyFixes?: boolean
	fix(this: this, f: FixBuilder): this
	applyFixes(this: this): this
}

export function withFix<T extends object>(node: T): T & Fixable {
	return Object.assign(node, {
		_fixes: [] as FixBuilder[],
		fix(this: any, f: FixBuilder) {
			this._fixes.push(f)
			return this
		},
		applyFixes(this: any) {
			this._applyFixes = true
			return this
		},
	})
}

export interface Enableable {
	_enabled: Reactive<boolean> | ((state: Record<string, NodeState>) => boolean) | undefined
	enabled(this: this, condition: Enableable['_enabled']): this
}

export function withEnabled<T extends object>(node: T): T & Enableable {
	return Object.assign(node, {
		_enabled: undefined as Enableable['_enabled'],
		enabled(this: any, condition: Enableable['_enabled']) {
			this._enabled = condition
			return this
		},
	})
}

export interface HasValue<V = unknown> {
	_defaultValue?: V | ((state: Record<string, NodeState>) => V)
	_getValue: (raw: NodeState) => V
	_setValue: (raw: NodeState, next: V, isDefault?: boolean) => NodeState
	_isActive: (value: V) => boolean
	initial(this: this, v: V | ((state: Record<string, NodeState>) => V)): this
}

export function withValue<T extends object, V>(
	node: T,
	behavior: Pick<HasValue<V>, '_getValue' | '_setValue' | '_isActive'>,
): T & HasValue<V> {
	return Object.assign(node, {
		...behavior,
		initial(this: any, v: V | ((state: Record<string, NodeState>) => V)) {
			this._defaultValue = v
			return this
		},
	})
}

export interface Clickable {
	_onClick: ((state: Record<string, NodeState>, write: (id: string, value: NodeState) => void) => void) | undefined
	onClick(this: this, fn: Clickable['_onClick']): this
}

export function withOnClick<T extends object>(node: T): T & Clickable {
	return Object.assign(node, {
		_onClick: undefined as Clickable['_onClick'],
		onClick(this: any, fn: Clickable['_onClick']) {
			this._onClick = fn
			return this
		},
	})
}

export interface ComponentNodePropsContext {
	onImageUpload?: (file: File) => Promise<string>
	toggleSetValue?: (value: string) => void
	nodeFacts?: { needsAttention: boolean; fixActionable: boolean }
	tooltip?: Record<string, unknown>
}

export interface Renderable {
	_component: Component | undefined
	_rendererKey: string | undefined
	_componentProps?: (ctx: ComponentNodePropsContext) => Record<string, unknown>
	_modelProp: string
}

export function withComponent<T extends object>(
	node: T,
	opts: {
		component?: Component
		rendererKey?: string
		modelProp?: string
		componentProps?: Renderable['_componentProps']
	},
): T & Renderable {
	return Object.assign(node, {
		_component: opts.component,
		_rendererKey: opts.rendererKey,
		_componentProps: opts.componentProps,
		_modelProp: opts.modelProp ?? 'modelValue',
	})
}

export interface Configurable {
	_extraProps: ((ctx: ComponentNodePropsContext) => Record<string, unknown>) | undefined
}

export function withExtraProps<T extends object>(node: T): T & Configurable {
	return Object.assign(node, {
		_extraProps: undefined as Configurable['_extraProps'],
	})
}

export interface HasNoneLabel {
	_none: string | undefined
	none(this: this, text: string): this
}

export function withNoneLabel<T extends object>(node: T): T & HasNoneLabel {
	return Object.assign(node, {
		_none: undefined as string | undefined,
		none(this: any, text: string) {
			this._none = text
			return this
		},
	})
}

export interface Layoutable {
	_layout: 'flex' | 'column' | undefined
	layout(this: this, value: 'flex' | 'column'): this
}

export function withLayout<T extends object>(node: T): T & Layoutable {
	return Object.assign(node, {
		_layout: undefined as 'flex' | 'column' | undefined,
		layout(this: any, value: 'flex' | 'column') {
			this._layout = value
			return this
		},
	})
}

export interface Selectable {
	_exclusive: boolean | undefined
	singleSelect(this: this): this
}

export function withSelectable<T extends object>(node: T): T & Selectable {
	return Object.assign(node, {
		_exclusive: undefined as boolean | undefined,
		singleSelect(this: any) {
			this._exclusive = true
			return this
		},
	})
}

export interface StageMeta {
	_hint: string | undefined
	_guidanceUrl: string | undefined
	_navigate: string | undefined
	_shownSticky: boolean | undefined
	hint(this: this, hint: string): this
	guidance(this: this, url: string): this
	// A stage-relative path (e.g. '/versions'). Omit entirely to mean the
	// project's own base page, rather than passing '/' as a magic string.
	navigate(this: this, path?: string): this
	sticky(this: this): this
}

export function withStageMeta<T extends object>(node: T): T & StageMeta {
	return Object.assign(node, {
		_hint: undefined as string | undefined,
		_guidanceUrl: undefined as string | undefined,
		_navigate: undefined as string | undefined,
		_shownSticky: undefined as boolean | undefined,
		hint(this: any, hint: string) {
			this._hint = hint
			return this
		},
		guidance(this: any, url: string) {
			this._guidanceUrl = url
			return this
		},
		navigate(this: any, path?: string) {
			this._navigate = path ?? ''
			return this
		},
		sticky(this: any) {
			this._shownSticky = true
			return this
		},
	})
}

export type OverrideValue = { readonly __override: string }

export type OnChangeFn = (
	value: string,
	helpers: { override: (value: string) => OverrideValue },
) => OverrideValue | void

export interface Editable {
	_placeholder: Reactive<string> | undefined
	_onChange: OnChangeFn | undefined
	_showTooltip: boolean | undefined
	_imperativeSync: boolean | undefined
	placeholder(this: this, p: Reactive<string>): this
	onChange(this: this, fn: OnChangeFn): this
}

export function withEditable<T extends object>(node: T): T & Editable {
	return Object.assign(node, {
		_placeholder: undefined as Reactive<string> | undefined,
		_onChange: undefined as OnChangeFn | undefined,
		_showTooltip: undefined as boolean | undefined,
		_imperativeSync: undefined as boolean | undefined,
		placeholder(this: any, p: Reactive<string>) {
			this._placeholder = p
			return this
		},
		onChange(this: any, fn: OnChangeFn) {
			this._onChange = fn
			return this
		},
	})
}

export interface StateOrigin {
	_stateOrigin: string[] | undefined
	stateOrigin(this: this, path: string[]): this
}

export function withStateOrigin<T extends object>(node: T): T & StateOrigin {
	return Object.assign(node, {
		_stateOrigin: undefined as string[] | undefined,
		stateOrigin(this: any, path: string[]) {
			this._stateOrigin = path
			return this
		},
	})
}
