import type { HasValue, Identified } from './capabilities'
import type { NodeState } from './state'

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
		if (value === undefined) delete container[childId]
		else container[childId] = value
		parentWrite(containerId, Object.keys(container).length === 0 ? undefined : container)
	}
}

export function writeNodeValue<V>(
	node: HasValue<V> & Identified,
	read: Record<string, NodeState>,
	write: Writer,
	next: V,
): void {
	write(node.id, node._setValue(read[node.id], next))
}
