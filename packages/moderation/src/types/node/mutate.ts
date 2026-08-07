import type { HasValue, Identified } from './capabilities'
import { getEffectiveValue } from './resolve'
import type { NodeState } from './state'

function valuesEqual(a: unknown, b: unknown): boolean {
	if (Array.isArray(a) && Array.isArray(b)) {
		if (a.length !== b.length) return false
		const setB = new Set(b)
		return a.every((v) => setB.has(v))
	}
	return a === b
}

export type Writer = (id: string, value: NodeState) => void

export function childWriter(
	parentRead: Record<string, NodeState>,
	parentWrite: Writer,
	containerId: string,
): Writer {
	return (childId, value) => {
		const existing = parentRead[containerId]
		const container: Record<string, NodeState> =
			existing && typeof existing === 'object' && !(existing instanceof Set)
				? { ...(existing as Record<string, NodeState>) }
				: existing !== undefined
					? { value: existing }
					: {}
		if (value === undefined) Reflect.deleteProperty(container, childId)
		else container[childId] = value
		parentWrite(containerId, Object.keys(container).length === 0 ? undefined : container)
	}
}

export function writeNodeValue<V>(
	node: HasValue<V> & Identified,
	read: Record<string, NodeState>,
	write: Writer,
	next: V,
	contextState: Record<string, NodeState> = read,
): void {
	const isDefault = valuesEqual(next, getEffectiveValue(node, undefined, contextState))
	write(node.id, node._setValue(read[node.id], next, isDefault))
}

export function originScope(
	root: Record<string, NodeState>,
	path: string[],
): { state: Record<string, NodeState>; write: Writer } {
	let state = root
	let write: Writer = (id, value) => {
		if (value === undefined) Reflect.deleteProperty(root, id)
		else root[id] = value
	}
	for (const segment of path) {
		write = childWriter(state, write, segment)
		const raw = state[segment]
		state =
			raw && typeof raw === 'object' && !(raw instanceof Set)
				? (raw as Record<string, NodeState>)
				: {}
	}
	return { state, write }
}
