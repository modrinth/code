<template>
	<div class="shrink-0 border-0 border-t border-solid border-divider bg-surface-1 text-sm">
		<!-- Header row (always visible) -->
		<div class="flex items-center gap-2 px-3 py-2">
			<div class="flex items-center gap-2 pr-1" v-if="showWalkthrough">
				<component :is="stage?._icon ?? ScaleIcon" class="size-4 shrink-0 text-orange" />
				<span class="font-semibold text-contrast">{{ stageTitle }}</span>
				<span v-if="stagePosition" class="text-xs text-secondary">{{ stagePosition }}</span>
			</div>

			<div
				v-if="collapsed"
				class="flex gap-2 "
			>
				<Button
					size="xs"
					v-tooltip="'Toggle View back to default view'"
					aria-label="Toggle View back to default view"
					@click="settings.set(moderationSettings.Experimental.ExperimentalModerationView, false)"
				>
					<ToggleLeftIcon /> Toggle View
				</Button>
				<Button
					size="xs"
					:disabled="engine.isOnFirstStage.value && !engine.checklistHasState.value"
					@click="engine.resetProgress()"
				>
					<BrushCleaningIcon /> Reset
				</Button>
				<Button v-if="showWalkthrough" size="xs" @click="layout.setChecklistConnected(!layout.checklistConnected.value)">
					<LinkIcon v-if="layout.checklistConnected.value" />
					<UnlinkIcon v-else />
					{{ layout.checklistConnected.value ? 'Unlink tabs' : 'Relink tabs' }}
				</Button>
				<Button v-if="canSkip" size="xs" @click="engine.skipCurrentProject()">
					<RightArrowIcon /> Skip project
				</Button>
				<Button
					v-if="engine.alreadyReviewed.value && !engine.reviewedAnyway.value"
					size="xs"
					@click="engine.reviewAnyway()"
				>
					<ScaleIcon /> Review anyway
				</Button>
				<Button size="xs" color="red" type="colored" @click="engine.handleExit()">
					<XIcon /> Exit
				</Button>

				<div
					v-if="lockBanner"
					class="flex items-center gap-2 rounded-md bg-bg px-2 py-1.5 text-xs text-orange"
				>
					<LockIcon class="size-3.5 shrink-0" />
					{{ lockBanner }}
				</div>
			</div>

			<div v-if="queuePosition" :class="['flex items-center gap-4', showWalkthrough ? 'mx-auto' : 'ml-auto']">
				<span>
					Completed: {{ queuePosition.completed }}
				</span>
				<span>
					Skipped: {{ queuePosition.skipped }}
				</span>
				<span>
					Total: {{ queuePosition.total }}
				</span>
			</div>

			<div v-if="showWalkthrough" class="ml-auto flex items-center gap-1">
				<ButtonLink
					v-if="stage?._guidanceUrl"
					size="xs"
					type="quiet"
					target="_blank"
					:href="stage._guidanceUrl"
				>
					<FileTextIcon /> Guidance
				</ButtonLink>
				<button
					v-tooltip="'Previous stage'"
					class="rounded p-1 text-secondary hover:bg-button-bg hover:text-contrast"
					aria-label="Previous stage"
					@click="engine.previousStage()"
				>
					<LeftArrowIcon class="size-4" />
				</button>
				<button
					v-tooltip="'Next stage'"
					class="rounded p-1 text-secondary hover:bg-button-bg hover:text-contrast"
					aria-label="Next stage"
					@click="engine.nextStage()"
				>
					<RightArrowIcon class="size-4" />
				</button>
				<button
					v-tooltip="collapsed ? 'Expand walkthrough' : 'Collapse walkthrough'"
					class="rounded p-1 text-secondary hover:bg-button-bg hover:text-contrast"
					:aria-label="collapsed ? 'Expand walkthrough' : 'Collapse walkthrough'"
					@click="layout.toggleWalkthroughCollapsed()"
				>
					<ChevronDownIcon v-if="!collapsed" class="size-4" />
					<ChevronUpIcon v-else class="size-4" />
				</button>
			</div>
		</div>

		<!-- Expanded body -->
		<div
			v-if="!collapsed"
			class="flex flex-col gap-2 border-0 border-t border-solid border-divider px-3 py-2"
		>
			<p v-if="stage?._hint" class="m-0 text-secondary">{{ stage._hint }}</p>

			<div
				v-if="lockBanner"
				class="flex items-center gap-2 rounded-md bg-bg px-2 py-1.5 text-xs text-orange"
			>
				<LockIcon class="size-3.5 shrink-0" />
				{{ lockBanner }}
			</div>

			<div class="flex flex-wrap items-center gap-1.5">
				<Button
					size="xs"
					:disabled="engine.isOnFirstStage.value && !engine.checklistHasState.value"
					@click="engine.resetProgress()"
				>
					<BrushCleaningIcon /> Reset
				</Button>
				<Button size="xs" @click="layout.setChecklistConnected(!layout.checklistConnected.value)">
					<LinkIcon v-if="layout.checklistConnected.value" />
					<UnlinkIcon v-else />
					{{ layout.checklistConnected.value ? 'Unlink tabs' : 'Relink tabs' }}
				</Button>
				<Button v-if="canSkip" size="xs" @click="engine.skipCurrentProject()">
					<RightArrowIcon /> Skip project
				</Button>
				<Button
					v-if="engine.alreadyReviewed.value && !engine.reviewedAnyway.value"
					size="xs"
					@click="engine.reviewAnyway()"
				>
					<ScaleIcon /> Review anyway
				</Button>
				<Button size="xs" color="red" type="colored" @click="engine.handleExit()">
					<XIcon /> Exit
				</Button>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	ToggleLeftIcon,
	BrushCleaningIcon,
	ChevronDownIcon,
	ChevronUpIcon,
	FileTextIcon,
	LeftArrowIcon,
	LinkIcon,
	LockIcon,
	MousePointer2Icon,
	RightArrowIcon,
	ScaleIcon,
	UnlinkIcon,
	XIcon,
} from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import { Button, ButtonLink } from '@modrinth/ui'
import { computed } from 'vue'

import { injectModerationChecklist } from '~/components/ui/moderation/checklist/checklist-context'
import { useModerationReviewLayout } from '~/services/moderation/review-layout'
import {useModerationQueue} from "~/services/moderation/queue.ts";

const engine = injectModerationChecklist()
const layout = useModerationReviewLayout()
const settings = useModerationSettings()

const contextMenuEnabled = computed(
	() => settings.value.get(moderationSettings.Experimental.InlineChecklistMenu),
)
function toggleContextMenu() {
	settings.value.set(moderationSettings.Experimental.InlineChecklistMenu, !contextMenuEnabled.value)
}

const moderationQueue = useModerationQueue()

const queuePosition = computed(() => {
	if (!moderationQueue.isQueueMode) return null;

	const items = moderationQueue.currentQueue.items
	const completed = moderationQueue.currentQueue.completed;
	const skipped = moderationQueue.currentQueue.skipped;

	return {
		completed: `${completed.length}`,
		skipped: `${skipped.length}`,
		total: `${items.length}`
	}
})

const showWalkthrough = computed(
	() => settings.value.get(moderationSettings.Experimental.ShowChecklistWalkthrough),
)

const collapsed = computed(() => !showWalkthrough.value ? true : layout.walkthroughCollapsed.value)

const stage = computed(() => engine.currentStageObj.value)
const stageTitle = computed(() => stage.value?.label ?? stage.value?.id ?? 'Moderation')
const stagePosition = computed(() => {
	const total = engine.resolvedStages.value.length
	const idx = engine.currentStage.value + 1
	return total > 0 ? `${idx} / ${total}` : ''
})

const canSkip = computed(() => !engine.done.value)

const lockBanner = computed(() => {
	const s = engine.lockStatus.value
	if (engine.isLockedByOther.value && s?.lockedBy?.username) {
		return `Locked by @${s.lockedBy.username}${engine.lockTimeRemaining.value ? ` — ${engine.lockTimeRemaining.value}` : ''}`
	}
	if (engine.alreadyReviewed.value && !engine.reviewedAnyway.value) {
		return 'This project was already moderated.'
	}
	return null
})
</script>
