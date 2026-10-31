<script setup lang="ts">
import { useElementSize } from '@vueuse/core'
import { computed, ref } from 'vue'

const height = defineModel<number | null>({ required: true })
const divider = ref<HTMLElement>()
const parent = computed(() => divider.value?.parentElement)
const pane = computed(() => divider.value?.previousElementSibling as HTMLElement | undefined)
const { height: parentHeight } = useElementSize(parent)
const { height: paneHeight } = useElementSize(pane)
const maximum = computed(() => Math.floor(parentHeight.value / 2))
const drag = ref<{ pointerId: number; y: number; height: number }>()

function update(value: number) {
	height.value = Math.round(Math.max(0, Math.min(maximum.value, value)))
}

function start(event: PointerEvent) {
	if (event.button !== 0 || !event.isPrimary) return
	event.preventDefault()
	const element = event.currentTarget as HTMLElement
	drag.value = { pointerId: event.pointerId, y: event.clientY, height: paneHeight.value }
	element.setPointerCapture(event.pointerId)
}

function move(event: PointerEvent) {
	if (drag.value?.pointerId !== event.pointerId) return
	update(drag.value.height + event.clientY - drag.value.y)
}

function stop(event: PointerEvent) {
	if (drag.value?.pointerId !== event.pointerId) return
	drag.value = undefined
	const element = event.currentTarget as HTMLElement
	if (element.hasPointerCapture(event.pointerId)) element.releasePointerCapture(event.pointerId)
}

function keydown(event: KeyboardEvent) {
	const step = event.shiftKey ? 50 : 10
	switch (event.key) {
		case 'ArrowUp':
			update(paneHeight.value - step)
			break
		case 'ArrowDown':
			update(paneHeight.value + step)
			break
		case 'Home':
			update(0)
			break
		case 'End':
			update(maximum.value)
			break
		default:
			return
	}
	event.preventDefault()
}
</script>

<template>
	<div
		ref="divider"
		role="separator"
		tabindex="0"
		aria-orientation="horizontal"
		aria-label="Resize traces height"
		:aria-valuemin="0"
		:aria-valuemax="maximum"
		:aria-valuenow="Math.round(paneHeight)"
		class="height-divider relative h-4 shrink-0 cursor-row-resize touch-none select-none focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
		:class="{ 'is-dragging': drag }"
		@pointerdown="start"
		@pointermove="move"
		@pointerup="stop"
		@pointercancel="stop"
		@lostpointercapture="stop"
		@keydown="keydown"
		@dblclick.prevent="height = null"
	/>
</template>

<style scoped>
.height-divider::after {
	content: '';
	position: absolute;
	top: 50%;
	left: 0;
	width: 100%;
	height: 1px;
	background: var(--color-divider);
}

.height-divider:hover::after,
.height-divider:focus-visible::after,
.height-divider.is-dragging::after {
	background: var(--color-brand);
}
</style>
