import { evalSegment } from './messages'
import { hasCap, isNodeActive, resolveActionState, walkNodes } from './resolve'
import type { ChildNode } from './builder'
import type { MessageSegment, NodeState } from './state'

export interface ActiveAction2 {
	node: object
	state: Record<string, NodeState>
	statePath: string[]
}

export function collectActiveActions(
	children: ChildNode[],
	stageState: Record<string, NodeState>,
	basePath: string[] = [],
): ActiveAction2[] {
	const actions: ActiveAction2[] = []
	walkNodes(
		children,
		stageState,
		(node, nodeState, localState, path) => {
			if (!hasCap(node, '_segments')) return
			const segments = node._segments as MessageSegment[]
			if (segments.length === 0) return
			if (!isNodeActive(node, nodeState, localState)) return
			actions.push({ node, state: resolveActionState(node, nodeState, localState), statePath: path })
		},
		basePath,
	)
	return actions
}

export function collectMessageNodes(
	children: ChildNode[],
	stageState: Record<string, NodeState>,
	basePath: string[] = [],
): ActiveAction2[] {
	const actions: ActiveAction2[] = []
	walkNodes(
		children,
		stageState,
		(node, nodeState, localState, path) => {
			if (!hasCap(node, '_segments')) return
			const segments = node._segments as MessageSegment[]
			if (segments.length === 0) return
			actions.push({ node, state: resolveActionState(node, nodeState, localState), statePath: path })
		},
		basePath,
	)
	return actions
}

function isDescendant(childPath: string[], ancestorPath: string[]): boolean {
	return childPath.length > ancestorPath.length && ancestorPath.every((key, i) => childPath[i] === key)
}

export async function evalActiveAction(
	entry: ActiveAction2,
	allActions: ActiveAction2[],
	consumed: Set<object>,
): Promise<string> {
	let result = ''
	const segments = (entry.node as { _segments: MessageSegment[] })._segments
	for (const seg of segments) {
		if (seg.type === 'collect') {
			let collected = ''
			for (const child of allActions) {
				if (consumed.has(child.node)) continue
				if (!isDescendant(child.statePath, entry.statePath)) continue
				consumed.add(child.node)
				collected += await evalActiveAction(child, allActions, consumed)
			}
			if (!collected.trim() && seg.fallback) {
				collected = await evalSegment(seg.fallback, entry.state, entry.statePath)
			}
			result += collected
		} else {
			result += await evalSegment(seg, entry.state, entry.statePath)
		}
	}
	return result
}
