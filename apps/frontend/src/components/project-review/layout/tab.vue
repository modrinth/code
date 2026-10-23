<template>
	<span>{{ reviewPanelLabels[tab] }}</span>
	<span v-if="count !== undefined" class="relative top-px rounded text-xs">({{ count }})</span>
	<KbdChip v-if="keybind" :keybind="keybind" class="relative bottom-px" />
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import KbdChip from '../kdb-chip.vue'
import { injectProjectReviewContext } from './context'
import { reviewPanelLabels } from './labels'
import { type ProjectReviewSlot, projectReviewTabs } from './types'

/** A review panel's tab label, with its item count and keyboard shortcut. */
const props = defineProps<{ tab: ProjectReviewSlot }>()

const { tabCounts } = injectProjectReviewPageContext()
const { counts } = injectProjectReviewContext()

const count = computed(() => {
	if (props.tab in counts.value) return counts.value[props.tab]
	return props.tab in tabCounts.value
		? tabCounts.value[props.tab as keyof typeof tabCounts.value]
		: undefined
})

const keybind = computed(() => {
	if ((projectReviewTabs as readonly string[]).includes(props.tab)) return `review-tab-${props.tab}`
	return props.tab === 'rulings' ? 'review-panel-rulings' : undefined
})
</script>
