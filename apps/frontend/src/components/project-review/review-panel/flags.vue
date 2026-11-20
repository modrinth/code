<template>
	<div v-if="flags.length" class="flex flex-wrap gap-1">
		<Tooltip
			v-for="flag in flags"
			:key="flag.key"
			interactive
			panel-class="w-[min(20rem,calc(100vw-1rem))] !border-0 !bg-transparent !p-0 font-normal !shadow-none [&_.tooltip-arrow]:!border-surface-3 [&_.tooltip-arrow]:!bg-surface-2"
		>
			<span class="inline-flex items-center gap-1.5 text-sm font-medium text-contrast">
				<TagCategoryFlagIcon class="size-4 shrink-0 text-purple" aria-hidden="true" />
				{{ flag.name || flag.key }}
			</span>
			<template #popper>
				<FlagCard
					:flag="flag"
					:location-title="locationTitle(flag.location?.review_panel_key)"
					@reveal="reviewContext.revealPanel"
					@pointerenter="setPreviewHovered(true)"
					@pointerleave="setPreviewHovered(false)"
					@pointerdown.stop
				/>
			</template>
		</Tooltip>
	</div>
</template>

<script setup lang="ts">
import { TagCategoryFlagIcon } from '@modrinth/assets'
import { Tooltip } from '@modrinth/ui'
import { computed, onBeforeUnmount } from 'vue'

import { injectReviewFlags } from '~/providers/project-review/review-flags'
import { injectReviewPanels } from '~/providers/project-review/review-panels'

import FlagCard from '../flag-list/flag-card.vue'
import { injectReviewContext } from './context'

const props = defineProps<{ panelKey?: string; panelId: string }>()
const reviewFlags = injectReviewFlags(null)
const panels = injectReviewPanels()
const reviewContext = injectReviewContext()
const flags = computed(() => (props.panelKey ? (reviewFlags?.forPanel(props.panelKey) ?? []) : []))
let previewHovered = false

function locationTitle(key: string | undefined) {
	return key ? panels.resolve({ kind: 'panel', key })?.panel.title : undefined
}

function setPreviewHovered(hovered: boolean) {
	if (previewHovered === hovered) return
	previewHovered = hovered
	reviewContext.setDropdownOpen(props.panelId, hovered)
}

onBeforeUnmount(() => setPreviewHovered(false))
</script>
