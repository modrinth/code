<template>
	<aside
		class="flex shrink-0 flex-col overflow-hidden border-0 border-l border-solid border-divider bg-surface-1 text-sm"
	>
		<div class="flex max-h-[45%] shrink-0 flex-col gap-2 overflow-y-auto p-2">
			<!-- Project-wide (no on-screen element) checks -->
			<section
				v-if="globalStages.length > 0"
				class="rounded-lg border border-solid border-divider bg-surface-2"
			>
				<h2
					class="m-0 px-2.5 pt-1.5 text-[0.7rem] font-semibold uppercase tracking-wide text-secondary"
				>
					Project-wide
				</h2>
				<ModerationElementFrame element-key="global" bar-only />
			</section>

			<!-- Everything flagged so far — click a pill to toggle it active/inactive -->
			<section class="flex flex-col gap-1.5">
				<h2 class="m-0 text-[0.7rem] font-semibold uppercase tracking-wide text-secondary">
					Flagged issues
				</h2>
				<template v-if="flaggedGroups.length > 0">
					<div
						v-for="grp in flaggedGroups"
						:key="grp.stageId"
						class="flex flex-wrap items-center gap-1"
					>
						<button
							class="shrink-0 text-[0.7rem] font-semibold text-secondary hover:text-contrast"
							@click="goToStageById(grp.stageId)"
						>
							{{ grp.label }}:
						</button>
						<button
							v-for="node in grp.nodes"
							:key="node.key"
							v-tooltip="
								node.active ? 'Active — click to deactivate' : 'Inactive — click to reactivate'
							"
							class="rounded-full border border-solid px-2 py-0.5 text-xs transition-colors"
							:class="
								node.active
									? 'border-brand bg-brand-highlight font-medium text-contrast'
									: 'border-divider bg-transparent text-secondary opacity-60 hover:opacity-100'
							"
							@click="engine.setNodeActive(node.statePath, !node.active)"
						>
							{{ node.label }}
						</button>
					</div>
				</template>
				<p v-else class="m-0 text-xs text-secondary">No issues flagged yet.</p>
			</section>
		</div>

		<div class="h-px w-full shrink-0 bg-divider" />

		<div class="flex min-h-0 flex-1 flex-col">
			<Suspense>
				<ModerationReviewThread class="min-h-0 flex-1" />
				<template #fallback>
					<div class="flex items-center gap-2 p-3 text-secondary">
						<SpinnerIcon class="size-4 animate-spin" /> Loading thread…
					</div>
				</template>
			</Suspense>
		</div>
	</aside>
</template>

<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { injectModerationChecklist } from '~/components/ui/moderation/checklist/checklist-context'

import ModerationElementFrame from './ModerationElementFrame.vue'
import ModerationReviewThread from './ModerationReviewThread.vue'

const engine = injectModerationChecklist()

const globalStages = computed(() => engine.stagesForElement('global'))

/** Every issue ever flagged this project, grouped by stage, with its live active state. */
const flaggedGroups = computed(() => {
	const stageLabel = (id: string) =>
		engine.resolvedStages.value.find((s) => s.id === id)?.label ?? id
	const byStage = new Map<
		string,
		{ key: string; label: string; statePath: string[]; active: boolean }[]
	>()
	for (const [key, n] of Object.entries(engine.touchedNodes.value)) {
		if (!byStage.has(n.stageId)) byStage.set(n.stageId, [])
		byStage.get(n.stageId)!.push({
			key,
			label: n.label,
			statePath: n.statePath,
			active: engine.activeNodePaths.value.has(key),
		})
	}
	return [...byStage.entries()].map(([stageId, nodes]) => ({
		stageId,
		label: stageLabel(stageId),
		nodes,
	}))
})

function goToStageById(stageId: string) {
	engine.setStage(stageId)
	engine.focusStage(engine.resolvedStages.value.find((s) => s.id === stageId))
}
</script>
