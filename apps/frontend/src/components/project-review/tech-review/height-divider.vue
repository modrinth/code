<script setup lang="ts">
import { useElementSize } from '@vueuse/core'
import { computed, ref } from 'vue'

import { useDividerDrag } from './use-divider-drag'

const height = defineModel<number | null>({ required: true })
const props = withDefaults(
	defineProps<{
		heightProperty: string
		label?: string
		side?: 'top' | 'bottom'
		min?: number
		reserve?: number
	}>(),
	{ label: 'Resize traces height', side: 'top', min: 0 },
)
const divider = ref<HTMLElement>()
const parent = computed(() => divider.value?.parentElement)
const pane = computed(
	() =>
		(props.side === 'bottom'
			? divider.value?.nextElementSibling
			: divider.value?.previousElementSibling) as HTMLElement | undefined,
)
const { height: parentHeight } = useElementSize(parent)
const { height: paneHeight } = useElementSize(pane)
function maxHeight(available: number, handleSize: number) {
	return Math.max(
		props.min,
		Math.floor(
			props.reserve === undefined
				? available / 2
				: available - props.reserve - handleSize,
		),
	)
}
const maximum = computed(() => maxHeight(parentHeight.value, divider.value?.offsetHeight ?? 0))
const direction = computed(() => (props.side === 'bottom' ? -1 : 1))
const drag = useDividerDrag()
const { dragging } = drag

function update(value: number) {
	height.value = Math.round(Math.max(props.min, Math.min(maximum.value, value)))
}

function start(event: PointerEvent) {
	if (event.button !== 0 || !event.isPrimary) return
	const element = event.currentTarget as HTMLElement
	const container = element.parentElement
	if (!container || !pane.value) return
	drag.start(event, {
		axis: 'y',
		direction: direction.value,
		size: pane.value.offsetHeight,
		min: props.min,
		max: maxHeight(container.clientHeight, element.offsetHeight),
		container,
		property: props.heightProperty,
		commit: (size) => (height.value = size),
	})
}

function keydown(event: KeyboardEvent) {
	const step = event.shiftKey ? 50 : 10
	switch (event.key) {
		case 'ArrowUp':
			update(paneHeight.value - step * direction.value)
			break
		case 'ArrowDown':
			update(paneHeight.value + step * direction.value)
			break
		case 'Home':
			update(props.min)
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
		:aria-label="label"
		:aria-valuemin="min"
		:aria-valuemax="maximum"
		:aria-valuenow="Math.round(paneHeight)"
		class="height-divider relative h-4 shrink-0 cursor-row-resize touch-none select-none focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
		:class="{ 'is-dragging': dragging }"
		@pointerdown="start"
		@pointermove="drag.move"
		@pointerup="drag.stop"
		@pointercancel="drag.stop"
		@lostpointercapture="drag.stop"
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
	background: var(--surface-3);
}

.height-divider:hover::after {
	background: var(--color-brand);
	transition: background-color 0s 0.5s;
}

.height-divider:focus-visible::after,
.height-divider.is-dragging::after {
	background: var(--color-brand);
	transition: none;
}
</style>
