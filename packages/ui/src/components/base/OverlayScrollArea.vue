<script setup lang="ts">
import 'overlayscrollbars/overlayscrollbars.css'

import { OverlayScrollbars } from 'overlayscrollbars'
import { onBeforeUnmount, onMounted, ref } from 'vue'

const props = withDefaults(defineProps<{ autoHide?: boolean; scrollShadows?: boolean }>(), {
	autoHide: true,
})

const target = ref<HTMLElement>()
const viewport = ref<HTMLElement>()
const content = ref<HTMLElement>()
const canScrollUp = ref(false)
const canScrollDown = ref(false)
let scrollbars: ReturnType<typeof OverlayScrollbars>

function updateScrollShadows(instance: OverlayScrollbars) {
	const { scrollTop } = instance.elements().scrollOffsetElement
	canScrollUp.value = scrollTop > 0
	canScrollDown.value = scrollTop + 1 < instance.state().overflowAmount.y
}

onMounted(() => {
	if (!target.value || !viewport.value || !content.value) return
	scrollbars = OverlayScrollbars(
		{ target: target.value, elements: { viewport: viewport.value, content: content.value } },
		{
			overflow: { x: 'hidden', y: 'scroll' },
			scrollbars: {
				theme: 'os-theme-modrinth',
				autoHide: props.autoHide ? 'leave' : 'never',
				autoHideSuspend: true,
			},
		},
		props.scrollShadows ? { updated: updateScrollShadows, scroll: updateScrollShadows } : undefined,
	)
	if (props.scrollShadows) updateScrollShadows(scrollbars)
})

onBeforeUnmount(() => scrollbars?.destroy())

defineExpose({ viewport })
</script>

<template>
	<div
		ref="target"
		class="overlay-scroll-area relative min-h-0 min-w-0 overflow-hidden"
		:class="{
			'scroll-shadows': scrollShadows,
			'can-scroll-up': canScrollUp,
			'can-scroll-down': canScrollDown,
		}"
		data-overlayscrollbars-initialize
	>
		<div ref="viewport" class="h-full w-full overflow-y-auto" data-overlayscrollbars-viewport>
			<div ref="content"><slot /></div>
		</div>
	</div>
</template>

<style scoped>
.scroll-shadows {
	background-color: inherit;
}

.scroll-shadows::before,
.scroll-shadows::after {
	position: absolute;
	inset-inline: 0;
	z-index: 1;
	height: 1.5rem;
	background-color: inherit;
	content: '';
	pointer-events: none;
	opacity: 0;
	transition: opacity 150ms ease;
}

.scroll-shadows::before {
	top: 0;
	mask-image: linear-gradient(to bottom, black, transparent);
}

.scroll-shadows::after {
	bottom: 0;
	mask-image: linear-gradient(to top, black, transparent);
}

.can-scroll-up::before,
.can-scroll-down::after {
	opacity: 1;
}

@media (prefers-reduced-motion: reduce) {
	.scroll-shadows::before,
	.scroll-shadows::after {
		transition: none;
	}
}

.overlay-scroll-area :deep(.os-theme-modrinth) {
	--os-size: 6px;
	--os-padding-perpendicular: 0;
	--os-padding-axis: 2px;
	--os-track-bg: transparent;
	--os-track-bg-hover: transparent;
	--os-track-bg-active: transparent;
	--os-handle-border-radius: 9999px;
	--os-handle-bg: var(--overlay-scrollbar-color, var(--color-scrollbar));
	--os-handle-bg-hover: var(--overlay-scrollbar-color, var(--surface-5));
	--os-handle-bg-active: var(--overlay-scrollbar-color, var(--surface-5));
}
</style>
