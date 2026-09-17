<template>
	<section
		class="review-panel h-full overflow-auto p-3"
		:inert="sidebarHidden"
		:aria-label="formatMessage(messages[slotName])"
	>
		<component :is="content" v-if="content" />
		<template v-else>
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages[slotName]) }}
			</h2>
			<p class="m-0 text-secondary">{{ formatMessage(messages.empty) }}</p>
		</template>
	</section>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'
import type { IDockviewPanelProps } from 'dockview-vue'
import { computed } from 'vue'

import { projectReviewMessages as messages } from '../messages'
import { injectProjectReviewContext } from './context'
import type { ProjectReviewSlot } from './types'

defineOptions({ inheritAttrs: false })
type PanelParams = { slot: ProjectReviewSlot }

const props = defineProps<{
	params: PanelParams | IDockviewPanelProps<PanelParams>
}>()
const { slots, leftVisible, rightVisible } = injectProjectReviewContext()
const { formatMessage } = useVIntl()
const slotName = computed(() =>
	'slot' in props.params ? props.params.slot : props.params.params.slot,
)
const content = computed(() => slots[slotName.value])
const sidebarHidden = computed(
	() =>
		(slotName.value === 'left' && !leftVisible.value) ||
		(slotName.value === 'right' && !rightVisible.value),
)
</script>

<style scoped>
@property --review-panel-scrollbar-color {
	syntax: '<color>';
	inherits: true;
	initial-value: transparent;
}

.review-panel {
	--review-panel-scrollbar-color: transparent;
	transition: --review-panel-scrollbar-color 1750ms ease;
}

.review-panel:hover {
	--review-panel-scrollbar-color: var(--color-scrollbar);
	transition-duration: 200ms;
}

.review-panel,
.review-panel :deep(*) {
	scrollbar-color: var(--review-panel-scrollbar-color) transparent;
}

.review-panel::-webkit-scrollbar-thumb,
.review-panel :deep(*::-webkit-scrollbar-thumb) {
	background: var(--review-panel-scrollbar-color);
}

@media (hover: none) {
	.review-panel {
		--review-panel-scrollbar-color: var(--color-scrollbar);
	}
}

@media (prefers-reduced-motion: reduce) {
	.review-panel {
		transition: none;
	}
}
</style>
