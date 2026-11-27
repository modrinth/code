<script setup lang="ts">
import { useElementSize } from '@vueuse/core'
import { computed, useTemplateRef } from 'vue'

import { useDividerDrag } from './use-divider-drag'

const props = defineProps<{
	modelValue: number
	label: string
	side: 'left' | 'right'
	min: number
	max?: number
	reserve: number | ((parent: HTMLElement) => number)
	defaultWidth?: number
	widthProperty: string
	responsive?: boolean
}>()
const emit = defineEmits<{ 'update:modelValue': [width: number] }>()
const drag = useDividerDrag()
const { dragging } = drag
const handle = useTemplateRef<HTMLElement>('handle')
const direction = computed(() => (props.side === 'left' ? 1 : -1))
const pane = computed(() => (props.responsive && handle.value ? adjacentPane(handle.value) : null))
const { width: paneWidth } = useElementSize(pane, undefined, { box: 'border-box' })
const container = computed(() => (props.responsive ? handle.value?.parentElement : null))
const { width: containerWidth } = useElementSize(container)

function adjacentPane(element: HTMLElement) {
	return (props.side === 'left' ? element.previousElementSibling : element.nextElementSibling) as
		| HTMLElement
		| null
}

function layoutParent(element: HTMLElement) {
	const view = element.ownerDocument.defaultView
	let parent = element.parentElement
	while (parent && view?.getComputedStyle(parent).display === 'contents') {
		parent = parent.parentElement
	}
	return parent
}

function maximum(element: HTMLElement, parent = layoutParent(element)) {
	const limit = props.max ?? Infinity
	const available = parent ? parent.clientWidth - element.offsetWidth : limit
	const reserve = typeof props.reserve === 'number' ? props.reserve : parent && props.reserve(parent)
	return Math.max(props.min, Math.min(limit, available - (reserve ?? 0)))
}

function update(element: HTMLElement, width: number) {
	emit('update:modelValue', Math.round(Math.max(props.min, Math.min(maximum(element), width))))
}

function renderedWidth(element: HTMLElement, parent: HTMLElement) {
	if (props.responsive) return adjacentPane(element)?.offsetWidth ?? props.modelValue
	const bounds = element.getBoundingClientRect()
	const parentBounds = parent.getBoundingClientRect()
	return props.side === 'left' ? bounds.left - parentBounds.left : parentBounds.right - bounds.right
}

function start(event: PointerEvent) {
	if (event.button !== 0 || !event.isPrimary) return
	const element = event.currentTarget as HTMLElement
	const parent = layoutParent(element)
	if (!parent) return
	drag.start(event, {
		axis: 'x',
		direction: direction.value,
		size: renderedWidth(element, parent),
		min: props.min,
		max: maximum(element, parent),
		container: parent,
		property: `${props.widthProperty}-drag`,
		commit: (width) => emit('update:modelValue', width),
	})
}

function reset(event: MouseEvent) {
	if (props.defaultWidth === undefined) {
		emit('update:modelValue', 0)
	} else {
		update(event.currentTarget as HTMLElement, props.defaultWidth)
	}
}

function keydown(event: KeyboardEvent) {
	const element = event.currentTarget as HTMLElement
	const step = event.shiftKey ? 50 : 10
	const current = props.responsive
		? (adjacentPane(element)?.offsetWidth ?? props.modelValue)
		: props.modelValue
	let width: number
	switch (event.key) {
		case 'ArrowLeft':
			width = current - step * direction.value
			break
		case 'ArrowRight':
			width = current + step * direction.value
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
</script>

<template>
	<div
		ref="handle"
		role="separator"
		tabindex="0"
		aria-orientation="vertical"
		:aria-label="label"
		:aria-valuemin="min"
		:aria-valuemax="max ?? Math.round(containerWidth)"
		:aria-valuenow="responsive ? Math.round(paneWidth) : modelValue"
		class="panel-divider relative min-h-0 cursor-col-resize touch-none select-none focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
		:class="{ 'is-dragging': dragging }"
		@pointerdown="start"
		@pointermove="drag.move"
		@pointerup="drag.stop"
		@pointercancel="drag.stop"
		@lostpointercapture="drag.stop"
		@keydown="keydown"
		@dblclick="reset"
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
