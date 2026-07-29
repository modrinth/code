import type { Ref } from 'vue'
import { toValue } from 'vue'

import type { AnyNode, ChildEntry, ChildNode, HasChildren } from './builder'
import type { HasValue, Identified } from './capabilities'
import type { NodeState, NodeStateWithChildren } from './state'
import { resolve } from './state'

export function hasCap<K extends string>(node: object, key: K): node is Record<K, unknown> {
	return key in node
}

export function hasValueCap(node: object): node is HasValue {
	return hasCap(node, '_getValue') && hasCap(node, '_isActive')
}

export function hasChildrenCap(node: object): node is HasChildren {
	return hasCap(node, '_children')
}

export function hasOptionsCap(node: object): node is { _options: OptionLike[] } {
	return hasCap(node, '_options')
}

export function hasIdCap(node: object): node is Identified {
	return hasCap(node, 'id')
}

interface OptionLike extends AnyNode, Partial<HasChildren> {
	value: string
	_segments?: unknown[]
}

export function isShown(node: AnyNode): boolean {
	return node._shown === undefined || resolve(node._shown)
}

export function getEffectiveValue<V>(
	node: HasValue<V>,
	rawState: NodeState,
	contextState: Record<string, NodeState> = {},
): V {
	if (rawState !== undefined) return node._getValue(rawState)
	if (node._defaultValue !== undefined) {
		const def =
			typeof node._defaultValue === 'function'
				? (node._defaultValue as (state: Record<string, NodeState>) => NodeState)(contextState)
				: (node._defaultValue as unknown as NodeState)
		return node._getValue(def)
	}
	return node._getValue(rawState)
}

export function isNodeActive(
	node: object,
	rawState: NodeState,
	contextState: Record<string, NodeState> = {},
): boolean {
	if (hasValueCap(node)) return node._isActive(getEffectiveValue(node, rawState, contextState))
	return false
}

export function getBooleanChildState(nodeState: NodeState): Record<string, NodeState> {
	if (nodeState && typeof nodeState === 'object' && !(nodeState instanceof Set)) {
		return nodeState as Record<string, NodeState>
	}
	return {}
}

export function resolveChildren(
	node: HasChildren,
	state: Record<string, NodeState>,
): ChildNode[] {
	const entries: ChildEntry[] = node._childrenFn ? node._childrenFn(state) : node._children

	const result: ChildNode[] = []
	for (const entry of entries) {
		if (entry == null) continue
		if (typeof entry === 'string') {
			result.push(entry)
			continue
		}
		if (typeof entry === 'function') {
			if (entry.length === 0) {
				result.push(entry as () => unknown)
				continue
			}
			const fn = entry as (state?: Record<string, NodeState>) => AnyNode | AnyNode[] | null
			const resolved = fn(state)
			if (resolved == null) continue
			if (Array.isArray(resolved)) {
				for (const r of resolved) {
					if (r != null && isShown(r)) result.push(r)
				}
				continue
			}
			if (isShown(resolved)) result.push(resolved)
			continue
		}
		const resolved = toValue(entry as AnyNode | Ref<AnyNode | null>)
		if (resolved != null && isShown(resolved)) result.push(resolved)
	}
	return result
}

function buildChildMap(
	children: ChildNode[],
	stateForFlattening: Record<string, NodeState>,
): Map<string, object> {
	const map = new Map<string, object>()
	function collect(nodes: ChildNode[]) {
		for (const node of nodes) {
			if (typeof node !== 'object' || node === null) continue
			if (hasIdCap(node)) {
				map.set(node.id, node)
				continue
			}
			if (hasChildrenCap(node)) collect(resolveChildren(node, stateForFlattening))
		}
	}
	collect(children)
	return map
}

function optionChildrenFor(node: object, state: Record<string, NodeState>): ChildNode[] {
	if (!hasOptionsCap(node) || !hasIdCap(node)) return []
	const raw = state[node.id]
	const selectedValue = typeof raw === 'string' ? raw : undefined
	const selected = node._options.find((o) => o.value === selectedValue)
	if (!selected || !hasChildrenCap(selected)) return []
	return resolveChildren(selected, state)
}

export function withDefaults<T extends Record<string, NodeState>>(rawState: T, children: ChildNode[]): T {
	if (rawState == null || typeof rawState !== 'object' || rawState instanceof Set) return rawState

	const childMap = buildChildMap(children, rawState)
	for (const [, node] of childMap) {
		for (const c of optionChildrenFor(node, rawState)) {
			if (typeof c === 'object' && c !== null && hasIdCap(c)) childMap.set(c.id, c)
		}
	}
	const resolving = new Set<string>()

	const proxy = new Proxy(rawState, {
		get(target, key, receiver) {
			if (typeof key !== 'string') return Reflect.get(target, key, receiver)
			const child = childMap.get(key)
			let value: NodeState = Reflect.get(target, key, receiver)

			if (value === undefined && child && hasValueCap(child) && !resolving.has(key)) {
				resolving.add(key)
				try {
					const effective = getEffectiveValue(child, undefined, proxy)
					const grandchildren = hasChildrenCap(child) ? resolveChildren(child, {}) : []
					if (grandchildren.length > 0) {
						return withDefaults({ value: effective } as Record<string, NodeState>, grandchildren)
					}
					value = child._setValue(undefined, effective)
				} finally {
					resolving.delete(key)
				}
			}

			if (
				child &&
				hasChildrenCap(child) &&
				value !== null &&
				typeof value === 'object' &&
				!(value instanceof Set)
			) {
				const nested = value as Record<string, NodeState>
				return withDefaults(nested, resolveChildren(child, nested))
			}

			if (value === undefined && child && hasChildrenCap(child)) {
				const grandchildren = resolveChildren(child, {})
				if (grandchildren.length > 0) return emptyScope(grandchildren)
			}

			return value
		},
		set(target, key, value, receiver) {
			return Reflect.set(target, key, value, receiver)
		},
	}) as T

	return proxy
}

export function resolveActionState(
	node: object,
	nodeState: NodeState,
	localState: Record<string, NodeState>,
): Record<string, NodeState> {
	if (!hasValueCap(node)) return localState
	const childState = getBooleanChildState(nodeState)
	const base = hasChildrenCap(node)
		? withDefaults(childState, resolveChildren(node, childState))
		: childState
	return { ...base, value: getEffectiveValue(node, nodeState, localState) as NodeState }
}

function emptyScope(children: ChildNode[]): Record<string, NodeState> {
	const childMap = buildChildMap(children, {})
	const resolving = new Set<string>()
	const cache = new Map<string, Record<string, NodeState>>()

	const proxy: Record<string, NodeState> = new Proxy(
		{},
		{
			get(_target, key) {
				if (typeof key !== 'string') return undefined
				const child = childMap.get(key)
				if (!child) return undefined

				if (hasValueCap(child) && !resolving.has(key)) {
					resolving.add(key)
					try {
						const def = getEffectiveValue(child, undefined, proxy)
						if (def !== undefined) return child._setValue(undefined, def)
					} finally {
						resolving.delete(key)
					}
				}

				if (!hasChildrenCap(child)) return undefined

				const cached = cache.get(key)
				if (cached) return cached

				const grandchildren = resolveChildren(child, {})
				if (grandchildren.length === 0) return undefined

				const nested = emptyScope(grandchildren)
				cache.set(key, nested)
				return nested
			},
		},
	) as Record<string, NodeState>

	return proxy
}

export type Visitor = (
	node: object,
	nodeState: NodeState,
	localState: Record<string, NodeState>,
	path: string[],
) => void

export function walkNodes(
	nodes: ChildNode[],
	stageState: Record<string, NodeState>,
	visitor: Visitor,
	path: string[] = [],
): void {
	for (const node of nodes) {
		if (typeof node !== 'object' || node === null) continue
		if (!isShown(node as AnyNode)) continue

		if (hasChildrenCap(node) && !hasValueCap(node)) {
			if (hasIdCap(node)) {
				const raw = stageState[node.id]
				const childState =
					raw && typeof raw === 'object' && !(raw instanceof Set)
						? (raw as Record<string, NodeState>)
						: {}
				walkNodes(resolveChildren(node, childState), childState, visitor, [...path, node.id])
			} else {
				walkNodes(resolveChildren(node, stageState), stageState, visitor, path)
			}
			continue
		}

		if (hasValueCap(node) && hasIdCap(node)) {
			const rawNodeState = stageState[node.id]
			const nodePath = [...path, node.id]
			visitor(node, rawNodeState, stageState, nodePath)

			const active = isNodeActive(node, rawNodeState, stageState)
			if (!active) continue

			if (hasOptionsCap(node)) {
				const selectedValue = typeof rawNodeState === 'string' ? rawNodeState : undefined
				const selected = node._options.find((o) => o.value === selectedValue)
				if (selected && isShown(selected)) {
					const selectedPath = [...path, selected.value]
					visitor(selected, undefined, stageState, selectedPath)
					if (hasChildrenCap(selected)) {
						walkNodes(resolveChildren(selected, stageState), stageState, visitor, selectedPath)
					}
				}
				continue
			}

			if (hasChildrenCap(node)) {
				const childState = getBooleanChildState(rawNodeState)
				walkNodes(resolveChildren(node, childState), childState, visitor, nodePath)
			}
		}
	}
}
