<template>
	<div
		v-if="visibleTargets.length"
		ref="actionsElement"
		class="relative flex min-h-0 min-w-0 shrink-0 flex-col border-0 border-b border-solid border-divider"
		:style="{
			height: `${actionsHeight ?? 218}px`,
			maxHeight: 'max(80px, calc(100% - 300px))',
		}"
	>
		<div
			class="-mx-1 flex min-h-0 min-w-0 flex-1 flex-col gap-4 overflow-y-auto overscroll-contain px-1 pb-4 pt-1"
		>
			<Tabs
				:value="selectedTarget"
				:tabs="tabs"
				wrap
				class="max-w-full shrink-0"
				@update:value="selectedTarget = $event as ActionTarget"
			/>
			<ReviewPanel
				v-if="visibleTargets.includes(selectedTarget)"
				:key="selectedTarget"
				mode="inline"
				class="min-w-0 shrink-0 !overflow-visible"
				:target="{ kind: selectedTarget }"
			/>
		</div>
		<div
			role="separator"
			tabindex="0"
			aria-orientation="horizontal"
			:aria-label="formatMessage(messages.resizeActions)"
			:aria-valuenow="Math.round(measuredHeight)"
			:aria-valuemin="80"
			:aria-valuemax="maxActionsHeight"
			class="actions-resize absolute -bottom-2 left-0 z-10 h-4 w-full cursor-row-resize touch-none"
			:class="{ 'is-resizing': actionsDrag }"
			@pointerdown="startActionsResize"
			@pointermove="resizeActions"
			@pointerup="stopActionsResize"
			@pointercancel="stopActionsResize"
			@lostpointercapture="stopActionsResize"
			@dblclick.prevent="actionsHeight = null"
			@keydown.up.prevent="setActionsHeight(measuredHeight - 20)"
			@keydown.down.prevent="setActionsHeight(measuredHeight + 20)"
		/>
	</div>
</template>

<script setup lang="ts">
import { defineMessages, Tabs, useVIntl } from '@modrinth/ui'
import { useElementSize } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import { injectReviewPanels } from '~/providers/project-review/review-panels'

import ReviewPanel from './review-panel/index.vue'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	resizeActions: {
		id: 'project-review.resize-checks',
		defaultMessage: 'Resize project checks',
	},
})
const actionsElement = ref<HTMLElement | null>(null)
const { height: measuredHeight } = useElementSize(actionsElement)
const actionsHeight = ref<number | null>(null)
const actionsDrag = ref<{ y: number; height: number } | null>(null)
const maxActionsHeight = ref(400)

function setActionsHeight(height: number) {
	maxActionsHeight.value = Math.max(
		80,
		(actionsElement.value?.parentElement?.clientHeight ?? 700) - 300,
	)
	actionsHeight.value = Math.min(maxActionsHeight.value, Math.max(80, height))
}

function startActionsResize(event: PointerEvent) {
	if (event.button !== 0) return
	event.preventDefault()
	const target = event.currentTarget as HTMLElement
	target.setPointerCapture(event.pointerId)
	actionsDrag.value = { y: event.clientY, height: measuredHeight.value }
}

function resizeActions(event: PointerEvent) {
	if (actionsDrag.value) {
		setActionsHeight(actionsDrag.value.height + event.clientY - actionsDrag.value.y)
	}
}

function stopActionsResize(event: PointerEvent) {
	actionsDrag.value = null
	const target = event.currentTarget as HTMLElement
	if (target.hasPointerCapture(event.pointerId)) target.releasePointerCapture(event.pointerId)
}

const { resolve } = injectReviewPanels()
const targets = ['re-review', 'reupload', 'rules', 'post-approval', 'status-alerts'] as const
type ActionTarget = (typeof targets)[number]
const selectedTarget = ref<ActionTarget>('re-review')
const visibleTargets = computed(() => targets.filter((kind) => resolve({ kind })))
const tabs = computed(() =>
	visibleTargets.value.map((kind) => ({
		value: kind,
		label: resolve({ kind })!.panel.title,
	})),
)

watch(
	visibleTargets,
	(visible, previous = []) => {
		if (visible.includes('re-review') && !previous.includes('re-review')) {
			selectedTarget.value = 're-review'
		} else if (!visible.includes(selectedTarget.value) && visible[0]) {
			selectedTarget.value = visible[0]
		}
	},
	{ immediate: true },
)
</script>

<style scoped>
.actions-resize::after {
	content: '';
	position: absolute;
	top: calc(50% - 1px);
	left: 0;
	width: 100%;
	height: 1px;
	pointer-events: none;
	background-color: var(--color-brand);
	opacity: 0;
}

.actions-resize:hover::after {
	opacity: 1;
	transition: opacity 0s 0.5s;
}

.actions-resize:focus-visible::after,
.actions-resize.is-resizing::after {
	opacity: 1;
	transition: none;
}
</style>
