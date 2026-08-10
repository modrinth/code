import { withAutoProps, withChildren } from './builder'
import type { Configurable, NodePropsContext } from './capabilities'
import {
	withEditable,
	withEnabled,
	withExtraProps,
	withFix,
	withIcon,
	withId,
	withLayout,
	withMessaging,
	withNoneLabel,
	withOnClick,
	withPriority,
	withRequired,
	withRenderer,
	withSelectable,
	withShown,
	withStageMeta,
	withStateOrigin,
	withTitle,
	withTooltip,
	withTweak,
	withValue,
} from './capabilities'
import { pipe } from './pipe'
import type { NodeState, NodeStateWithChildren } from './state'

function getBooleanValue(raw: NodeState): boolean {
	if (typeof raw === 'boolean') return raw
	if (raw && typeof raw === 'object' && !(raw instanceof Set)) {
		const v = (raw as NodeStateWithChildren).value
		if (typeof v === 'boolean') return v
	}
	return false
}

function setBooleanValue(raw: NodeState, next: boolean, isDefault?: boolean): NodeState {
	if (raw && typeof raw === 'object' && !(raw instanceof Set)) {
		const { value: _v, ...children } = raw as NodeStateWithChildren & Record<string, NodeState>
		if (Object.keys(children).length > 0) return isDefault ? children : { ...children, value: next }
	}
	return isDefault ? undefined : next
}

export const booleanValue = {
	_getValue: getBooleanValue,
	_setValue: setBooleanValue,
	_isActive: (v: boolean) => v === true,
}

export function toggle(id: string, label: string) {
	return pipe(
		{ label } as { label: string },
		(n) => withId(n, id),
		withChildren,
		withShown,
		withIcon,
		withTooltip,
		withTitle,
		withMessaging,
		withPriority,
		withRequired,
		withFix,
		withEnabled,
		(n) => withValue(n, booleanValue),
		(n) => withRenderer(n, { renderer: { type: 'action' } }),
	)
}

export function button(label?: string) {
	return pipe(
		{ label } as { label?: string },
		withShown,
		withIcon,
		withTooltip,
		withEnabled,
		withOnClick,
	)
}

export function check(id: string, label: string) {
	return withAutoProps(
		pipe(
			{ label } as { label: string },
			(n) => withId(n, id),
			withChildren,
			withShown,
			withTooltip,
			withTitle,
			withMessaging,
			withPriority,
			withRequired,
			withFix,
			withEnabled,
			(n) => withValue(n, booleanValue),
			(n) => withRenderer(n, { renderer: { type: 'checkbox' } }),
			withExtraProps,
		),
	)
}

export function toggleSwitch(id: string, label: string) {
	return withAutoProps(
		pipe(
			{ label } as { label: string },
			(n) => withId(n, id),
			withChildren,
			withShown,
			withTooltip,
			withTitle,
			withMessaging,
			withPriority,
			withRequired,
			withFix,
			withEnabled,
			(n) => withValue(n, booleanValue),
			(n) => withRenderer(n, { renderer: { type: 'toggle' } }),
			withExtraProps,
		),
	)
}

export function group(id?: string) {
	const base = pipe(
		{} as Record<string, never>,
		withChildren,
		withShown,
		withTitle,
		withRequired,
		withSelectable,
		withLayout,
	)
	return id === undefined ? base : withId(base, id)
}

export type GroupNode = ReturnType<typeof group>

export function externalGroup(path: string[]) {
	return pipe({} as Record<string, never>, withChildren, withShown, (n) =>
		withStateOrigin(n).stateOrigin(path),
	)
}

const optionValue = {
	_getValue: () => true,
	_setValue: () => undefined,
	_isActive: () => true,
}

export function option(value: string, label: string) {
	return pipe(
		{ value, label } as { value: string; label: string },
		withChildren,
		withShown,
		withMessaging,
		withPriority,
		(n) => withValue(n, optionValue),
	)
}

type OptionNode = ReturnType<typeof option>

interface HasOptions {
	_options: OptionNode[]
	options(this: this, ...opts: OptionNode[]): this
}

function withOptions<T extends object>(node: T): T & HasOptions {
	return Object.assign(node, {
		_options: [] as OptionNode[],
		options(this: HasOptions, ...opts: OptionNode[]) {
			this._options = opts
			return this
		},
	})
}

function getStringValue(raw: NodeState): string {
	if (typeof raw === 'string') return raw
	if (raw && typeof raw === 'object' && !(raw instanceof Set)) {
		const v = (raw as NodeStateWithChildren).value
		if (typeof v === 'string') return v
	}
	return ''
}

function setStringValue(raw: NodeState, next: string, isDefault?: boolean): NodeState {
	if (raw && typeof raw === 'object' && !(raw instanceof Set)) {
		const { value: _v, ...children } = raw as NodeStateWithChildren & Record<string, NodeState>
		if (Object.keys(children).length > 0) return isDefault ? children : { ...children, value: next }
	}
	return isDefault ? undefined : next
}

const dropdownValue = {
	_getValue: getStringValue,
	_setValue: setStringValue,
	_isActive: (v: string) => v !== '',
}

export function dropdown(id: string) {
	return pipe(
		{} as Record<string, never>,
		(n) => withId(n, id),
		withShown,
		withTitle,
		withRequired,
		(n) => withValue(n, dropdownValue),
		withNoneLabel,
		withOptions,
		(n) => withRenderer(n, { renderer: { type: 'dropdown' } }),
	)
}

export function stage(id: string, title: string) {
	return pipe(
		{ label: title } as { label: string },
		(n) => withId(n, id),
		withChildren,
		withShown,
		withIcon,
		withStageMeta,
		withMessaging,
		withPriority,
	)
}

export type StageNode = ReturnType<typeof stage>

const stringValue = dropdownValue

export function text(id: string) {
	return withAutoProps(
		pipe(
			{} as Record<string, never>,
			(n) => withId(n, id),
			withChildren,
			withShown,
			withTooltip,
			withTitle,
			withMessaging,
			withPriority,
			withRequired,
			withFix,
			withEnabled,
			withEditable,
			(n) => Object.assign(n, { _showTooltip: true, _imperativeSync: true }),
			(n) => withValue(n, stringValue),
			withTweak,
			(n) => withRenderer(n, { renderer: { type: 'text' } }),
			withExtraProps,
		),
	)
}

export function markdown(id: string) {
	return withAutoProps(
		pipe(
			{} as Record<string, never>,
			(n) => withId(n, id),
			withChildren,
			withShown,
			withTooltip,
			withTitle,
			withMessaging,
			withPriority,
			withRequired,
			withFix,
			withEnabled,
			withEditable,
			(n) => withValue(n, stringValue),
			(n) => withRenderer(n, { renderer: { type: 'markdown' } }),
			withExtraProps,
		),
	)
}

function getSetValue(raw: NodeState): string[] {
	return raw instanceof Set ? Array.from(raw) : []
}

function setSetValue(_raw: NodeState, next: string[], isDefault?: boolean): NodeState {
	return isDefault || next.length === 0 ? undefined : new Set(next)
}

const setValue = {
	_getValue: getSetValue,
	_setValue: setSetValue,
	_isActive: (v: string[]) => v.length > 0,
}

const stringValueBehavior = {
	_getValue: getStringValue,
	_setValue: setStringValue,
	_isActive: (v: string) => v !== '',
}

export function appComponent(id: string, rendererKey: string) {
	const node = pipe(
		{} as Record<string, never>,
		(n) => withId(n, id),
		withChildren,
		withShown,
		withTooltip,
		withTitle,
		withMessaging,
		withPriority,
		withRequired,
		withFix,
		withEnabled,
		(n) => withValue(n, stringValueBehavior),
		(n) => withRenderer(n, { renderer: { type: 'custom', key: rendererKey } }),
		withExtraProps,
	)
	return Object.assign(node, {
		valueKind(this: Configurable, kind: 'boolean' | 'string' | 'set') {
			Object.assign(
				this,
				kind === 'boolean' ? booleanValue : kind === 'set' ? setValue : stringValueBehavior,
			)
			return this
		},
		props(this: Configurable, fn: (ctx: NodePropsContext) => Record<string, unknown>) {
			this._extraProps = fn
			return this
		},
	})
}
