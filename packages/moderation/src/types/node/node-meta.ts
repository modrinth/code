import type { AnyNode, ChildNode, HasChildren } from './builder'
import type { FixBuilder } from './fix'
import { getBooleanChildState, hasCap, hasChildrenCap, hasIdCap, hasOptionsCap, hasValueCap, isNodeActive, isShown, resolveChildren, walkNodes } from './resolve'
import type { NodeState } from './state'

export interface NodeMeta {
	hasRequiredMissing: boolean
	isFixActionable: boolean
}

export function computeNodeMeta(
	children: ChildNode[],
	stageState: Record<string, NodeState>,
	isFixActionable: (fixes: FixBuilder[], state: Record<string, NodeState>) => boolean,
): Map<object, NodeMeta> {
	const map = new Map<object, NodeMeta>()

	walkNodes(children, stageState, (node, nodeState, localState) => {
		const active = isNodeActive(node, nodeState, localState)

		const required = hasCap(node, '_required') && node._required === true
		const hasRequiredMissing = required && !active

		let fixActionable = false
		if (active && hasCap(node, '_fixes')) {
			const fixes = node._fixes as FixBuilder[]
			if (fixes.length > 0) {
				const actionState = hasValueCap(node) ? getBooleanChildState(nodeState) : localState
				fixActionable = isFixActionable(fixes, actionState)
			}
		}

		map.set(node, { hasRequiredMissing, isFixActionable: fixActionable })
	})

	return map
}

export function computeAttentionMap(
	nodes: ChildNode[],
	stageState: Record<string, NodeState>,
	metaMap: Map<object, NodeMeta>,
	attention: Map<object, boolean> = new Map(),
): Map<object, boolean> {
	for (const node of nodes) {
		if (typeof node !== 'object' || node === null) continue
		if (!isShown(node as AnyNode)) continue

		if (hasChildrenCap(node) && !hasValueCap(node)) {
			const childState = childScopeFor(node, stageState)
			computeAttentionMap(resolveChildren(node, childState), childState, metaMap, attention)
			const childrenNeedAttention = someChildNeedsAttention(node, childState, attention)
			const ownRequired = hasCap(node, '_required') && node._required === true
			const unsatisfied = ownRequired && !hasActiveChild(node, childState)
			attention.set(node, unsatisfied || childrenNeedAttention)
			continue
		}

		if (hasValueCap(node) && hasIdCap(node)) {
			const rawNodeState = stageState[node.id]
			const active = isNodeActive(node, rawNodeState, stageState)
			let childrenNeedAttention = false

			if (active) {
				if (hasOptionsCap(node)) {
					const selectedValue = typeof rawNodeState === 'string' ? rawNodeState : undefined
					const selected = node._options.find((o) => o.value === selectedValue)
					if (selected && isShown(selected) && hasChildrenCap(selected)) {
						computeAttentionMap(resolveChildren(selected, stageState), stageState, metaMap, attention)
						childrenNeedAttention = someChildNeedsAttention(selected, stageState, attention)
					}
				} else if (hasChildrenCap(node)) {
					const childState = getBooleanChildState(rawNodeState)
					computeAttentionMap(resolveChildren(node, childState), childState, metaMap, attention)
					childrenNeedAttention = someChildNeedsAttention(node, childState, attention)
				}
			}

			const ownMissing = metaMap.get(node)?.hasRequiredMissing ?? false
			attention.set(node, ownMissing || childrenNeedAttention)
		}
	}
	return attention
}

function childScopeFor(node: HasChildren & { id?: string }, state: Record<string, NodeState>): Record<string, NodeState> {
	if (!hasIdCap(node)) return state
	const raw = state[node.id]
	return raw && typeof raw === 'object' && !(raw instanceof Set) ? (raw as Record<string, NodeState>) : {}
}

function hasActiveChild(node: HasChildren, childState: Record<string, NodeState>): boolean {
	return resolveChildren(node, childState).some((child) => {
		if (typeof child !== 'object' || child === null) return false
		const raw = hasIdCap(child) ? childState[child.id] : undefined
		return isNodeActive(child, raw, childState)
	})
}

function someChildNeedsAttention(
	node: HasChildren,
	childState: Record<string, NodeState>,
	attention: Map<object, boolean>,
): boolean {
	return resolveChildren(node, childState).some(
		(child) => typeof child === 'object' && child !== null && attention.get(child) === true,
	)
}
