<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'

const props = defineProps<{
	modelValue: number
	label: string
	side: 'left' | 'right'
	min: number
	max: number
	reserve: number
	defaultWidth: number
	widthProperty: string
}>()
const emit = defineEmits<{ 'update:modelValue': [width: number] }>()
const dragging = ref(false)
let drag:
	| {
			pointerId: number
			startX: number
			startWidth: number
			width: number
			maximum: number
			parent: HTMLElement
			view: Window
			frame?: number
	  }
	| undefined

function layoutParent(element: HTMLElement) {
	const view = element.ownerDocument.defaultView
	let parent = element.parentElement
	while (parent && view?.getComputedStyle(parent).display === 'contents') {
		parent = parent.parentElement
	}
	return parent
}

function maximum(element: HTMLElement, parent = layoutParent(element)) {
	return Math.max(
		props.min,
		Math.min(props.max, (parent?.clientWidth ?? props.max) - props.reserve - element.offsetWidth),
	)
}

function update(element: HTMLElement, width: number) {
	emit('update:modelValue', Math.round(Math.max(props.min, Math.min(maximum(element), width))))
}

function start(event: PointerEvent) {
	if (event.button !== 0 || !event.isPrimary) return
	const element = event.currentTarget as HTMLElement
	const parent = layoutParent(element)
	const view = element.ownerDocument.defaultView
	if (!parent || !view) return
	event.preventDefault()
	const bounds = element.getBoundingClientRect()
	const parentBounds = parent.getBoundingClientRect()
	const width =
		props.side === 'left' ? bounds.left - parentBounds.left : parentBounds.right - bounds.right
	drag = {
		pointerId: event.pointerId,
		startX: event.clientX,
		startWidth: width,
		width,
		maximum: maximum(element, parent),
		parent,
		view,
	}
	dragging.value = true
	element.setPointerCapture(event.pointerId)
}

function move(event: PointerEvent) {
	const current = drag
	if (!current || current.pointerId !== event.pointerId) return
	const delta = (event.clientX - current.startX) * (props.side === 'left' ? 1 : -1)
	current.width = Math.round(
		Math.max(props.min, Math.min(current.maximum, current.startWidth + delta)),
	)
	if (current.frame !== undefined) return
	current.frame = current.view.requestAnimationFrame(() => {
		current.frame = undefined
		current.parent.style.setProperty(props.widthProperty, `${current.width}px`)
	})
}

function stop(event: PointerEvent) {
	const current = drag
	if (!current || current.pointerId !== event.pointerId) return
	if (current.frame !== undefined) current.view.cancelAnimationFrame(current.frame)
	current.parent.style.setProperty(props.widthProperty, `${current.width}px`)
	drag = undefined
	dragging.value = false
	emit('update:modelValue', current.width)
	const element = event.currentTarget as HTMLElement
	if (element.hasPointerCapture(event.pointerId)) element.releasePointerCapture(event.pointerId)
}

function keydown(event: KeyboardEvent) {
	const element = event.currentTarget as HTMLElement
	const step = event.shiftKey ? 50 : 10
	const direction = props.side === 'left' ? 1 : -1
	let width: number
	switch (event.key) {
		case 'ArrowLeft':
			width = props.modelValue - step * direction
			break
		case 'ArrowRight':
			width = props.modelValue + step * direction
			break
		case 'Home':
			width = props.min
			break
		case 'End':
			width = maximum(element)
			break
		default:
			return
	}
	event.preventDefault()
	update(element, width)
}

onBeforeUnmount(() => {
	if (drag?.frame !== undefined) drag.view.cancelAnimationFrame(drag.frame)
	drag = undefined
})
</script>

<template>
	<div
		role="separator"
		tabindex="0"
		aria-orientation="vertical"
		:aria-label="label"
		:aria-valuemin="min"
		:aria-valuemax="max"
		:aria-valuenow="modelValue"
		class="panel-divider relative min-h-0 cursor-col-resize touch-none select-none focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
		:class="{ 'is-dragging': dragging }"
		@pointerdown="start"
		@pointermove="move"
		@pointerup="stop"
		@pointercancel="stop"
		@lostpointercapture="stop"
		@keydown="keydown"
		@dblclick="update($event.currentTarget as HTMLElement, defaultWidth)"
	/>
</template>

<style scoped>
.panel-divider::after {
	content: '';
	position: absolute;
	inset: 0 auto 0 50%;
	width: 1px;
	background: var(--surface-3);
}

.panel-divider:hover::after {
	background: var(--color-brand);
	transition: background-color 0s 0.5s;
}

.panel-divider:focus-visible::after,
.panel-divider.is-dragging::after {
	background: var(--color-brand);
	transition: none;
}
</style>
