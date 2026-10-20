<template>
	<span>{{ reviewPanelLabels[tab] }}</span>
	<span v-if="count !== undefined" class="relative top-px rounded text-xs">({{ count }})</span>
	<KbdChip :keybind="`review-tab-${tab}`" class="relative bottom-px" />
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import KbdChip from '../kdb-chip.vue'
import { reviewPanelLabels } from './labels'
import type { ProjectReviewTab } from './types'

/** A review tab's label, with its item count and keyboard shortcut. */
const props = defineProps<{ tab: ProjectReviewTab }>()

const { tabCounts } = injectProjectReviewPageContext()

const count = computed(() =>
	props.tab in tabCounts.value
		? tabCounts.value[props.tab as keyof typeof tabCounts.value]
		: undefined,
)
</script>
