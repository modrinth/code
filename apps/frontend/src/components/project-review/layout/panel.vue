<template>
	<section
		class="layout-panel h-full p-3"
		:class="slotName === 'right' ? 'overflow-hidden' : 'overflow-auto'"
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
@property --layout-panel-scrollbar-color {
	syntax: '<color>';
	inherits: true;
	initial-value: transparent;
}

.layout-panel {
	--layout-panel-scrollbar-color: transparent;
	transition: --layout-panel-scrollbar-color 1750ms ease;
}

.layout-panel:hover {
	--layout-panel-scrollbar-color: var(--color-scrollbar);
	transition-duration: 200ms;
}

.layout-panel,
.layout-panel :deep(*) {
	scrollbar-color: var(--layout-panel-scrollbar-color) transparent;
}

.layout-panel::-webkit-scrollbar-thumb,
.layout-panel :deep(*::-webkit-scrollbar-thumb) {
	background: var(--layout-panel-scrollbar-color);
}

@media (hover: none) {
	.layout-panel {
		--layout-panel-scrollbar-color: var(--color-scrollbar);
	}
}

@media (prefers-reduced-motion: reduce) {
	.layout-panel {
		transition: none;
	}
}
</style>
