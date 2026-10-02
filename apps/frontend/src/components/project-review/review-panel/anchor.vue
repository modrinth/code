<template>
	<component
		:is="as"
		ref="element"
		class="relative"
		:data-review-anchor="id"
		:data-review-target="target.kind"
		:data-review-key="'key' in target ? target.key : undefined"
		@pointerover.stop="onPointerOver"
		@pointermove="onPointerOver"
		@pointerleave="leave(id)"
		@focusin="active?.id === id && cancelClose()"
		@focusout="leave(id)"
	>
		<Highlight
			:active="active?.id === id"
			:enabled="settings.get(moderationSettings.General.ShowFloatingPanelFieldHoverHighlight)"
			layer="behind"
		>
			<slot />
			<div v-if="showFindingBadge" class="absolute right-0 top-0 flex items-center gap-1">
				<ReReviewFindingsCountBadge :count="reReviewFindingCount" />
				<FindingsCountBadge :count="selectedFindingCount" />
			</div>
		</Highlight>
	</component>
</template>

<script setup lang="ts">
import { moderationSettings } from '@modrinth/moderation'
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import { useModerationSettings } from '~/composables/moderation'
import type { ReviewTarget } from '~/providers/project-review/review'
import { injectReviewPanels } from '~/providers/project-review/review-panels'
import { injectReviewPreviousIssues } from '~/providers/project-review/review-previous-issues'

import { injectReviewContext } from './context'
import FindingsCountBadge from './findings-count-badge.vue'
import ReReviewFindingsCountBadge from './re-review-findings-count-badge.vue'
import Highlight from './highlight.vue'

const props = withDefaults(
	defineProps<{
		anchorId: string
		target: ReviewTarget
		as?: 'section' | 'div' | 'article'
		disabled?: boolean
		showFindingBadge?: boolean
	}>(),
	{ as: 'div', showFindingBadge: true },
)
const { active, isAvailable, open, release, leave, enter, cancelClose, revealAnchor } =
	injectReviewContext()
const id = props.anchorId
const element = ref<HTMLElement>()
const available = computed(() => !props.disabled && isAvailable(props.target))
const settings = useModerationSettings()
const panels = injectReviewPanels()
const previousIssues = injectReviewPreviousIssues()
const reReviewFindingCount = computed(() => {
	const binding = panels.resolve(props.target)
	return binding ? previousIssues.reReviewFindingCount(binding) : 0
})
const selectedFindingCount = computed(() => {
	const binding = panels.resolve(props.target)
	if (!binding) return 0
	return panels.selectedFindingCount(binding)
})

defineExpose({
	element,
	reveal: () => {
		if (!element.value) return
		revealAnchor({
			id,
			target: props.target,
			element: element.value,
			available: () => available.value,
		})
	},
})

function show() {
	if (!element.value || !available.value) return
	open({
		id,
		target: props.target,
		element: element.value,
		available: () => available.value,
	})
}

function onPointerOver(event: PointerEvent) {
	if (event.pointerType === 'touch') return
	if (active.value?.id === id) {
		enter(id)
		return
	}
	show()
}

watch(available, (value) => {
	if (!value) release(id)
})
watch([() => props.target.kind, () => ('key' in props.target ? props.target.key : undefined)], () =>
	release(id),
)
onBeforeUnmount(() => release(id))
</script>
