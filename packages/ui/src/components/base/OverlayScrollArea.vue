<script setup lang="ts">
import 'overlayscrollbars/overlayscrollbars.css'

import { OverlayScrollbars } from 'overlayscrollbars'
import { onBeforeUnmount, onMounted, ref } from 'vue'

const target = ref<HTMLElement>()
const viewport = ref<HTMLElement>()
const content = ref<HTMLElement>()
let scrollbars: ReturnType<typeof OverlayScrollbars>

onMounted(() => {
	if (!target.value || !viewport.value || !content.value) return
	scrollbars = OverlayScrollbars(
		{ target: target.value, elements: { viewport: viewport.value, content: content.value } },
		{
			overflow: { x: 'hidden', y: 'scroll' },
			scrollbars: { theme: 'os-theme-modrinth', autoHide: 'leave', autoHideSuspend: true },
		},
	)
})

onBeforeUnmount(() => scrollbars?.destroy())
</script>

<template>
	<div
		ref="target"
		class="overlay-scroll-area relative min-h-0 min-w-0 overflow-hidden"
		data-overlayscrollbars-initialize
	>
		<div ref="viewport" class="h-full w-full overflow-y-auto" data-overlayscrollbars-viewport>
			<div ref="content"><slot /></div>
		</div>
	</div>
</template>

<style scoped>
.overlay-scroll-area :deep(.os-theme-modrinth) {
	--os-size: 6px;
	--os-padding-perpendicular: 0;
	--os-padding-axis: 2px;
	--os-track-bg: transparent;
	--os-track-bg-hover: transparent;
	--os-track-bg-active: transparent;
	--os-handle-border-radius: 9999px;
	--os-handle-bg: var(--color-scrollbar);
	--os-handle-bg-hover: var(--surface-5);
	--os-handle-bg-active: var(--surface-5);
}
</style>
