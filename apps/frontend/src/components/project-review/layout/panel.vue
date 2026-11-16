<template>
	<section
		class="layout-panel review-scrollbars h-full p-3"
		:class="
			slotName === 'right' ||
			slotName === 'disclosures' ||
			slotName === 'permissions' ||
			slotName === 'gallery' ||
			slotName === 'slicer' ||
			slotName === 'tech-review'
				? 'overflow-hidden'
				: 'overflow-auto'
		"
		:inert="panelHidden"
		:data-scrollbars-inactive="!panelVisible || panelHidden || !content || undefined"
		:aria-label="reviewPanelLabels[slotName]"
	>
		<component :is="content" v-if="content" />
		<template v-else>
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ reviewPanelLabels[slotName] }}
			</h2>
			<p class="m-0 text-secondary">No project selected</p>
		</template>
	</section>
</template>

<script setup lang="ts">
import type { IDockviewPanelProps } from 'dockview-vue'
import { computed, ref, watchEffect } from 'vue'

import { injectProjectReviewContext, provideReviewPanelVisible, provideReviewSlot } from './context'
import { reviewPanelLabels } from './labels'
import type { ProjectReviewSlot } from './types'

defineOptions({ inheritAttrs: false })
type PanelParams = { slot: ProjectReviewSlot }

const props = defineProps<{
	params: PanelParams | IDockviewPanelProps<PanelParams>
}>()
const { slots, leftVisible, rightVisible } = injectProjectReviewContext()

const slotName = computed(() =>
	'slot' in props.params ? props.params.slot : props.params.params.slot,
)
provideReviewSlot(slotName)
const panelVisible = ref(true)
watchEffect((onCleanup) => {
	if (!('api' in props.params)) return
	const api = props.params.api
	panelVisible.value = api.isVisible
	const subscription = api.onDidVisibilityChange((event) => {
		panelVisible.value = event.isVisible
	})
	onCleanup(() => subscription.dispose())
})
provideReviewPanelVisible(panelVisible)
const content = computed(() => slots[slotName.value])
const panelHidden = computed(
	() =>
		(slotName.value === 'left' && !leftVisible.value) ||
		(slotName.value === 'right' && !rightVisible.value),
)
</script>

<style scoped>
@property --review-scrollbar-color {
	syntax: '<color>';
	inherits: true;
	initial-value: transparent;
}

.review-scrollbars {
	--review-scrollbar-color: transparent;
	--overlay-scrollbar-color: var(--review-scrollbar-color);
	transition: --review-scrollbar-color 1750ms ease;
}

.review-scrollbars:hover {
	--review-scrollbar-color: var(--color-scrollbar);
	transition-duration: 200ms;
}

.review-scrollbars,
.review-scrollbars :deep(*) {
	scrollbar-color: var(--review-scrollbar-color) transparent;
}

.review-scrollbars::-webkit-scrollbar-thumb,
.review-scrollbars :deep(*::-webkit-scrollbar-thumb) {
	background: var(--review-scrollbar-color);
}

.layout-panel[data-scrollbars-inactive],
.layout-panel[data-scrollbars-inactive] :deep(.review-scrollbars) {
	--review-scrollbar-color: transparent !important;
	--overlay-scrollbar-color: transparent !important;
	transition: none !important;
}

.layout-panel[data-scrollbars-inactive],
.layout-panel[data-scrollbars-inactive] :deep(*) {
	scrollbar-color: transparent transparent !important;
}

.layout-panel[data-scrollbars-inactive]::-webkit-scrollbar-thumb,
.layout-panel[data-scrollbars-inactive] :deep(*::-webkit-scrollbar-thumb) {
	background: transparent !important;
}

.layout-panel[data-scrollbars-inactive] :deep(.os-scrollbar) {
	visibility: hidden;
	transition: none !important;
}

@media (hover: none) {
	.review-scrollbars {
		--review-scrollbar-color: var(--color-scrollbar);
	}
}

@media (prefers-reduced-motion: reduce) {
	.review-scrollbars {
		transition: none;
	}
}
</style>
