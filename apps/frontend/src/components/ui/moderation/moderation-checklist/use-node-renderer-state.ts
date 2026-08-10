import type { Labrinth } from '@modrinth/api-client'
import { expandVariables } from '@modrinth/moderation'
import {
	collectMessageNodes,
	computeAttentionMap,
	computeNodeMeta,
	evalActiveAction,
	resolveChildren,
} from '@modrinth/moderation/src/types/node'
import type {
	FixBuilder,
	NodeState,
	StageNode,
	Writer,
} from '@modrinth/moderation/src/types/node'
import { renderHighlightedString } from '@modrinth/utils'
import type { ComputedRef, Ref } from 'vue'
import { computed, ref, watchEffect } from 'vue'

interface NodeRendererStateOptions {
	currentStage: ComputedRef<StageNode>
	nodeStates: Ref<Record<string, Record<string, NodeState>>>
	project: Ref<Labrinth.Projects.v3.Project>
	projectV2: Ref<Labrinth.Projects.v2.Project>
	isFixActionable: (fixes: FixBuilder[], state: Record<string, NodeState>) => boolean
}

export function useNodeRendererState({
	currentStage,
	nodeStates,
	project,
	projectV2,
	isFixActionable,
}: NodeRendererStateOptions) {
	const tooltipHtml = ref(new Map<object, string>())
	const state = computed(
		() => (nodeStates.value[currentStage.value.id] ?? {}) as Record<string, NodeState>,
	)
	const nodes = computed(() => resolveChildren(currentStage.value, state.value))

	const write: Writer = (id, value) => {
		const stageId = currentStage.value.id
		const existing = nodeStates.value[stageId]
		const next: Record<string, NodeState> = existing ? { ...existing } : {}
		if (value === undefined) Reflect.deleteProperty(next, id)
		else next[id] = value
		if (Object.keys(next).length === 0) {
			if (existing !== undefined) Reflect.deleteProperty(nodeStates.value, stageId)
		} else {
			nodeStates.value[stageId] = next
		}
	}

	watchEffect(async (onCleanup) => {
		let cancelled = false
		onCleanup(() => {
			cancelled = true
		})
		const stage = currentStage.value
		const actions = collectMessageNodes(nodes.value, state.value, [stage.id])
		const next = new Map<object, string>()
		await Promise.all(
			actions.map(async (entry) => {
				try {
					const raw = await evalActiveAction(entry, actions, new Set())
					const expanded = expandVariables(raw, projectV2.value, project.value).trim()
					next.set(
						entry.node,
						expanded
							? `<div class="markdown-body moderation-tooltip-markdown">${renderHighlightedString(expanded)}</div>`
							: '',
					)
				} catch {
					next.set(entry.node, '')
				}
			}),
		)
		if (!cancelled) tooltipHtml.value = next
	})

	const meta = computed(() => {
		const metaMap = computeNodeMeta(nodes.value, state.value, isFixActionable)
		const attentionMap = computeAttentionMap(nodes.value, state.value, metaMap)
		return { metaMap, attentionMap, tooltipHtml: tooltipHtml.value }
	})

	return { meta, nodes, state, write }
}
