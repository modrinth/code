import type { Ref } from 'vue'

import type { Configurable, Visible } from './capabilities'
import type { NodeState } from './state'

export type AnyNode = Visible

export type ChildNode = AnyNode | (() => unknown) | string

export type ChildEntry =
	| AnyNode
	| string
	| (() => unknown)
	| null
	| Ref<AnyNode | null>
	| ((state?: Record<string, NodeState>) => AnyNode | AnyNode[] | null)

export type ChildrenFn = (state: Record<string, NodeState>) => ChildEntry[]

export interface HasChildren {
	_children: ChildEntry[]
	_childrenFn?: ChildrenFn
	children(this: this, fn: ChildrenFn): this
	children(this: this, ...entries: ChildEntry[]): this
}

export function withChildren<T extends object>(node: T): T & HasChildren {
	return Object.assign(node, {
		_children: [] as ChildEntry[],
		children(this: any, ...args: [ChildrenFn] | ChildEntry[]) {
			if (args.length === 1 && typeof args[0] === 'function' && args[0].length >= 1) {
				this._childrenFn = args[0] as ChildrenFn
			} else {
				this._children.push(...(args as ChildEntry[]))
			}
			return this
		},
	})
}

export function withAutoProps<T extends Configurable>(
	node: T,
): T & { [key: string]: (value: unknown) => T } {
	return new Proxy(node, {
		get(target, prop, receiver) {
			if (typeof prop === 'string' && !prop.startsWith('_') && !(prop in target)) {
				return (value: unknown) => {
					const prev = target._extraProps
					target._extraProps = (ctx) => ({ ...prev?.(ctx), [prop]: value })
					return receiver
				}
			}
			return Reflect.get(target, prop, receiver)
		},
	}) as T & { [key: string]: (value: unknown) => T }
}
