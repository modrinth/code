<template>
	<div class="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto">
		<FlagCard
			v-for="flag in flags"
			:key="flag.key"
			:flag="flag"
			:location-title="locationTitle(flag.location?.review_panel_key)"
			@reveal="revealPanel"
		/>
		<p v-if="isPending" class="m-0 text-sm text-secondary">Loading flags...</p>
		<p v-else-if="isError" class="m-0 text-sm text-red">Failed to load project flags.</p>
		<p v-else-if="!flags.length" class="m-0 text-sm text-secondary">
			No flags on this project.
		</p>
	</div>
</template>

<script setup lang="ts">
import { injectReviewFlags } from '~/providers/project-review/review-flags'
import { injectReviewPanels } from '~/providers/project-review/review-panels'

import { injectReviewContext } from '../review-panel/context'
import FlagCard from './flag-card.vue'

const { flags, isPending, isError } = injectReviewFlags()
const panels = injectReviewPanels()
const { revealPanel } = injectReviewContext()
function locationTitle(key: string | undefined) {
	return key ? panels.resolve({ kind: 'panel', key })?.panel.title : undefined
}
</script>
