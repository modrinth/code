<template>
	<div
		v-if="visible"
		class="flex min-w-0 flex-col gap-1"
		:class="
			variant === 'bar' ? 'rounded-md border border-solid border-divider bg-surface-2 p-2' : ''
		"
	>
		<div class="flex flex-wrap gap-1">
			<NodeRenderer
				v-if="topNodes.length > 0"
				:stage-id="stageId"
				:nodes="topNodes"
				:state="stageState"
				:write="writer"
				:global-state="engine.nodeStates.value"
				:on-image-upload="engine.onUploadHandler"
				:app-components="engine.appComponents"
				:mode="inlineNodeMode ?? 'buttons'"
				class="max-w-fit"
				:flex="true"
				:title-depth="3"
				button-size="xs"
			>
				<button
					v-if="!hideHeading"
					ref="anchorEl"
					v-tooltip="stage?._hint"
					class="flex min-w-0 items-center gap-1 rounded px-1 text-left text-[0.7rem] font-semibold uppercase tracking-wide text-secondary hover:text-contrast"
					@click="openStage"
					@pointerenter="onEnter"
					@pointerleave="onLeave"
				>
					<component :is="stage?._icon" v-if="stage?._icon" class="size-3.5 shrink-0 text-orange" />
					<span class="truncate">{{ heading }}</span>
				</button>
				<button
					v-tooltip="
						hideHeading && stage?._hint
							? stage._hint
							: panelOpen
								? 'Hide checklist details'
								: 'Show checklist details'
					"
					class="shrink-0 rounded p-0.5 hover:bg-button-bg"
					:class="panelOpen ? 'text-brand' : 'text-secondary hover:text-contrast'"
					:aria-label="`${heading} checklist details`"
					@click="() => {}"
					@pointerenter="onEnter"
					@pointerleave="onLeave"
				>
					<ChevronDownIcon
						class="size-3.5 transition-transform"
						:class="{ 'rotate-180': panelOpen }"
					/>
				</button>
			</NodeRenderer>
		</div>

		<div v-if="$slots.default" class="py-2">
			<slot />
		</div>

		<ChecklistDetailsPanel
			v-if="panelOpen"
			:title="heading"
			:anchor="anchorEl"
			:pinned="pinned"
			@set-pin="(value) => (pinned = value)"
			@toggle-pin="togglePin"
			@close="closePanel"
			@hoverin="onEnter"
			@hoverout="onLeave"
		>
			<p v-if="stage?._hint" class="mb-3 mt-0 text-xs text-secondary">{{ stage._hint }}</p>
			<NodeRenderer
				:stage-id="stageId"
				:nodes="topNodes"
				:state="stageState"
				:write="writer"
				:global-state="engine.nodeStates.value"
				:on-image-upload="engine.onUploadHandler"
				:app-components="engine.appComponents"
				button-size="xs"
			/>
		</ChecklistDetailsPanel>
	</div>
</template>

<script setup lang="ts">
import { ChevronDownIcon } from '@modrinth/assets'
import { expandVariables } from '@modrinth/moderation'
import type { ChecklistMetaContext } from '@modrinth/moderation/src/types/node'
import {
	CHECKLIST_META_KEY,
	collectMessageNodes,
	evalActiveAction,
	resolveChildren,
} from '@modrinth/moderation/src/types/node'
import NodeRenderer from '@modrinth/moderation/src/types/node/components/NodeRenderer.vue'
import {type ButtonSize, injectProjectPageContext} from '@modrinth/ui'
import { renderHighlightedString } from '@modrinth/utils'
import { computed, onBeforeUnmount, provide, ref, watchEffect } from 'vue'

import {
	elementForStage,
	injectModerationChecklist,
} from '~/components/ui/moderation/checklist/checklist-context'

import ChecklistDetailsPanel from './ChecklistDetailsPanel.vue'

const props = withDefaults(
	defineProps<{
		stageId: string
		/** `bar` wraps the cluster in a bordered surface; `inline` sits flush next to content. */
		variant?: 'bar' | 'inline'
		/** Drop the label row (the host already shows the section name, e.g. sidebar cards). */
		hideHeading?: boolean
		inlineNodeMode?: 'buttons' | 'full'
	}>(),
	{ variant: 'inline', hideHeading: false, inlineNodeMode: 'buttons' },
)

const engine = injectModerationChecklist()
const { projectV2, projectV3 } = injectProjectPageContext()

const stage = computed(() => engine.resolvedStages.value.find((s) => s.id === props.stageId))

/** Only render when the stage is currently visible for its mapped element. */
const visible = computed(() => {
	const key = elementForStage(props.stageId)
	return !!key && engine.stagesForElement(key).some((s) => s.id === props.stageId)
})

const heading = computed(() => stage.value?.label ?? props.stageId)
const stageState = computed(() => engine.nodeStates.value[props.stageId] ?? {})
const writer = computed(() => engine.writerForStage(props.stageId))
const topNodes = computed(() => (stage.value ? resolveChildren(stage.value, stageState.value) : []))

/**
 * The floating checklist widget only ever renders `currentStageObj`, so it can afford one
 * global `CHECKLIST_META_KEY` scoped to whatever stage is current. Every element on the review
 * page shows several stages' buttons at once, so each `ChecklistStageButtons` instance computes
 * and provides its own — scoped to just its stage — for its bar *and* its detail panel. This is
 * what gives a childless toggle's tooltip the message it would actually generate.
 */
const tooltipHtmlMap = ref(new Map<object, string>())

watchEffect(async () => {
	const s = stage.value
	if (!s) {
		tooltipHtmlMap.value = new Map()
		return
	}
	const nodes = topNodes.value
	const actions = collectMessageNodes(nodes, stageState.value, [s.id])

	const newMap = new Map<object, string>()
	await Promise.all(
		actions.map(async (entry) => {
			try {
				const raw = await evalActiveAction(entry, actions, new Set())
				const expanded = expandVariables(raw, projectV2.value, projectV3.value).trim()
				newMap.set(
					entry.node,
					expanded
						? `<div class="markdown-body moderation-tooltip-markdown">${renderHighlightedString(expanded)}</div>`
						: '',
				)
			} catch {
				newMap.set(entry.node, '')
			}
		}),
	)
	tooltipHtmlMap.value = newMap
})

provide(
	CHECKLIST_META_KEY,
	computed<ChecklistMetaContext>(() => ({
		metaMap: new Map(),
		attentionMap: new Map(),
		tooltipHtml: tooltipHtmlMap.value,
	})),
)

const anchorEl = ref<HTMLElement | null>(null)
const pinned = ref(false)
const hoverOpen = ref(false)
const panelOpen = computed(() => pinned.value || hoverOpen.value)

let showTimer: ReturnType<typeof setTimeout> | null = null
let hideTimer: ReturnType<typeof setTimeout> | null = null

function clearTimers() {
	if (showTimer) clearTimeout(showTimer)
	if (hideTimer) clearTimeout(hideTimer)
	showTimer = hideTimer = null
}

function onEnter() {
	if (hideTimer) {
		clearTimeout(hideTimer)
		hideTimer = null
	}
	if (panelOpen.value || showTimer) return
	showTimer = setTimeout(() => {
		hoverOpen.value = true
		showTimer = null
	}, 140)
}

function onLeave() {
	if (showTimer) {
		clearTimeout(showTimer)
		showTimer = null
	}
	if (pinned.value) return
	hideTimer = setTimeout(() => {
		hoverOpen.value = false
		hideTimer = null
	}, 220)
}

function togglePin() {
	if (pinned.value) {
		pinned.value = false
		hoverOpen.value = false
	} else {
		pinned.value = true
	}
}

function closePanel() {
	pinned.value = false
	hoverOpen.value = false
}

function openStage() {
	if (!stage.value) return
	engine.setStage(props.stageId)
	engine.focusStage(stage.value)
}

onBeforeUnmount(clearTimers)
</script>
