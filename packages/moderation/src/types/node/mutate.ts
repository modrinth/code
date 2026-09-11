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
		// Writing into a container that has never been touched at all (not even `false`) has to
		// mark the container itself active (`value: true`) — otherwise the parent toggle reads
		// back as inactive despite now holding child state, hiding/gating that very child (see
		// `booleanValue`/`isNodeActive`). This is reachable whenever a child is set before its
		// parent's own toggle has ever been clicked, e.g. picking a sub-option straight from a
		// hover menu.
		const container: Record<string, NodeState> =
			existing && typeof existing === 'object' && !(existing instanceof Set)
				? { ...(existing as Record<string, NodeState>) }
				: { value: existing !== undefined ? existing : true }
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
