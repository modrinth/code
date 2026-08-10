import type { NodeState, StageNode } from '@modrinth/moderation/src/types/node'
import type { ComputedRef, Ref } from 'vue'
import { ref, toRaw, watch } from 'vue'

import {
	getSessionChecklistState,
	patchSessionChecklistState,
} from '~/services/moderation/checklist-session-storage.ts'
import {
	clearChecklistState,
	loadChecklistState,
	saveChecklistState,
} from '~/services/moderation/checklist-storage.ts'

export async function loadChecklistPersistence(projectId: string) {
	const persistedState = import.meta.client ? await loadChecklistState(projectId) : null
	const activatedStages = ref<Set<string>>(new Set(persistedState?.activatedStages ?? []))
	const visitedStages = ref<Set<string>>(
		new Set(
			import.meta.client ? (getSessionChecklistState(projectId).visitedStages ?? []) : [],
		),
	)
	const reviewedAnyway = ref(persistedState?.reviewAnyway ?? false)
	const message = ref<string | null>(persistedState?.message ?? null)

	function markStageVisited(stageId: string | undefined) {
		if (!stageId || visitedStages.value.has(stageId)) return
		visitedStages.value.add(stageId)
		patchSessionChecklistState(projectId, {
			visitedStages: [...visitedStages.value],
		})
	}

	return {
		activatedStages,
		markStageVisited,
		message,
		persistedState,
		reviewedAnyway,
		visitedStages,
	}
}

interface ChecklistPersistenceOptions {
	projectId: string
	nodeStates: Ref<Record<string, Record<string, NodeState>>>
	activatedStages: Ref<Set<string>>
	reviewedAnyway: Ref<boolean>
	message: Ref<string | null>
	currentStage: Ref<number>
	currentStageNode: ComputedRef<StageNode>
	firstVisibleStage: () => number
	markStageVisited: (stageId: string | undefined) => void
	visitCurrentStageImmediately: boolean
}

export function useChecklistPersistence({
	projectId,
	nodeStates,
	activatedStages,
	reviewedAnyway,
	message,
	currentStage,
	currentStageNode,
	firstVisibleStage,
	markStageVisited,
	visitCurrentStageImmediately,
}: ChecklistPersistenceOptions) {
	let enabled = true
	let timer: ReturnType<typeof setTimeout> | null = null

	function cancelPendingSave() {
		if (timer === null) return
		clearTimeout(timer)
		timer = null
	}

	function save(open: boolean, resetReviewAnyway = false) {
		const rawState = toRaw(nodeStates.value)
		const openValue = open || undefined
		const reviewedAnywayValue = resetReviewAnyway ? undefined : reviewedAnyway.value || undefined
		const stageValue =
			currentStage.value !== firstVisibleStage() ? currentStageNode.value.id : undefined
		const messageValue = message.value ?? undefined
		const stateValue = Object.keys(rawState).length > 0 ? rawState : undefined
		const activatedStagesValue =
			activatedStages.value.size > 0 ? [...activatedStages.value] : undefined

		if (
			!openValue &&
			!reviewedAnywayValue &&
			!stageValue &&
			!messageValue &&
			!stateValue &&
			!activatedStagesValue
		) {
			return clearChecklistState(projectId)
		}

		return saveChecklistState(projectId, {
			...(openValue && { open: openValue }),
			...(reviewedAnywayValue && { reviewAnyway: reviewedAnywayValue }),
			...(stageValue && { stage: stageValue }),
			...(messageValue && { message: messageValue }),
			...(stateValue && { state: stateValue }),
			...(activatedStagesValue && { activatedStages: activatedStagesValue }),
		})
	}

	function persist() {
		if (!enabled || !import.meta.client) return
		cancelPendingSave()
		timer = setTimeout(() => {
			timer = null
			void save(true)
		}, 150)
	}

	async function persistImmediately(open: boolean, resetReviewAnyway = false) {
		if (!import.meta.client) return
		cancelPendingSave()
		await save(open, resetReviewAnyway)
	}

	function disable() {
		enabled = false
		cancelPendingSave()
	}

	function dispose() {
		cancelPendingSave()
		if (enabled) void save(true)
	}

	watch(currentStage, persist)
	watch(nodeStates, persist, { deep: true })
	watch(activatedStages, persist, { deep: true })
	watch(message, persist)
	watch(currentStageNode, (stage) => markStageVisited(stage.id), {
		immediate: visitCurrentStageImmediately,
	})

	return { disable, dispose, persist, persistImmediately }
}
