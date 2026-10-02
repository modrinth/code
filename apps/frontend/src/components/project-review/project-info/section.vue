<template>
	<component
		:is="target ? ReviewPanel : 'section'"
		v-bind="target ? { mode: 'anchored', as: 'section', target, showFindingBadge: false } : {}"
		:aria-label="heading"
		class="review-section"
		:class="{ 'editable-review-section': !!$slots.right }"
	>
		<div class="flex items-center gap-1">
			<h3 class="m-0 flex-1 text-sm font-semibold text-secondary">{{ heading }}</h3>
			<div
				v-if="$slots.right || selectedFindingCount > 0 || reReviewFindingCount > 0"
				class="flex shrink-0 items-center gap-2"
			>
				<slot name="right" />
				<ReReviewFindingsCountBadge :count="reReviewFindingCount" />
				<FindingsCountBadge :count="selectedFindingCount" />
			</div>
		</div>
		<slot />
	</component>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import type { ReviewTarget } from '~/providers/project-review/review'
import { injectReviewPanels } from '~/providers/project-review/review-panels'
import { injectReviewPreviousIssues } from '~/providers/project-review/review-previous-issues'

import FindingsCountBadge from '../review-panel/findings-count-badge.vue'
import ReReviewFindingsCountBadge from '../review-panel/re-review-findings-count-badge.vue'
import ReviewPanel from '../review-panel/index.vue'

const props = defineProps<{
	heading: string
	target?: ReviewTarget
}>()
const panels = injectReviewPanels()
const previousIssues = injectReviewPreviousIssues()
const reReviewFindingCount = computed(() => {
	const binding = props.target ? panels.resolve(props.target) : undefined
	return binding ? previousIssues.reReviewFindingCount(binding) : 0
})
const selectedFindingCount = computed(() => {
	const binding = props.target ? panels.resolve(props.target) : undefined
	return binding ? panels.selectedFindingCount(binding) : 0
})
</script>

<style scoped>
.review-section {
	@apply flex flex-col gap-2 border-0 border-t border-solid border-divider px-0.5 pb-3 pt-3;
}
</style>
