<template>
	<component
		:is="target ? ReviewPanel : 'section'"
		v-bind="target ? { mode: 'anchored', as: 'section', target, showFindingBadge: false } : {}"
		:aria-label="heading"
		class="review-section"
		:class="[
			{ 'editable-review-section': !!$slots.right },
			settings.get(moderationSettings.General.SectionReducedGap)
				? 'gap-1 pb-1.5 pt-1.5'
				: 'gap-2 pb-3 pt-3',
		]"
	>
		<template v-if="showHeader">
			<div class="flex items-center gap-1">
				<div class="flex flex-1 items-center gap-1">
					<h3 class="m-0 text-sm font-semibold text-secondary">{{ heading }}</h3>
					<slot name="heading-after" />
				</div>
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
		</template>
		<template v-else>
			<corner-wrap>
				<template #corner>
					<div class="flex items-center gap-1">
						<slot name="right" />
						<ReReviewFindingsCountBadge :count="reReviewFindingCount" />
						<FindingsCountBadge :count="selectedFindingCount" />
					</div>
				</template>
				<slot />
			</corner-wrap>
		</template>
	</component>
</template>

<script setup lang="ts">
import { moderationSettings } from '@modrinth/moderation'
import { computed } from 'vue'

import CornerWrap from '~/components/project-review/project-info/corner-wrap.vue'
import { useModerationSettings } from '~/composables/moderation.ts'
import type { ReviewTarget } from '~/providers/project-review/review'
import { injectReviewPanels } from '~/providers/project-review/review-panels'
import { injectReviewPreviousIssues } from '~/providers/project-review/review-previous-issues'

import FindingsCountBadge from '../review-panel/findings-count-badge.vue'
import ReviewPanel from '../review-panel/index.vue'
import ReReviewFindingsCountBadge from '../review-panel/re-review-findings-count-badge.vue'

const settings = useModerationSettings()

const showHeader = computed(() =>
	settings.value.get(moderationSettings.General.SectionHeadersVisible),
)

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
	@apply flex flex-col border-0 border-t border-solid border-divider px-0.5;
}
</style>
