<template>
	<Teleport :to="anchor.element.ownerDocument.body">
		<div
			:id="panelId"
			ref="element"
			role="dialog"
			:aria-labelledby="titleId"
			:aria-label="titleId ? undefined : label"
			:data-review-panel="anchor.id"
			tabindex="-1"
			class="review-popover z-[100] box-border flex w-max min-w-[min(280px,calc(100vw-1rem))] flex-col gap-2 overflow-y-auto rounded-xl border-solid p-2.5 pb-3 text-sm text-primary"
			:style="[
				floatingStyles,
				appearanceStyle,
				{ maxWidth: `min(${maxWidth ?? '28rem'}, calc(100vw - 1rem))` },
			]"
			@pointerenter="setPopoverHovered(anchor.id, true)"
			@pointerleave="setPopoverHovered(anchor.id, false)"
			@focusin="cancelClose"
			@focusout="leave(anchor.id)"
			@keydown="onKeydown"
			@click="retainExternalLink(anchor.id, $event)"
		>
			<div ref="header" class="flex items-center justify-between gap-3">
				<slot name="title" />
				<div class="flex shrink-0 items-center gap-1">
					<IconButton
						v-tooltip="pinned ? 'Unpin review controls' : 'Pin review controls'"
						size="sm"
						:label="pinned ? 'Unpin review controls' : 'Pin review controls'"
						:aria-pressed="pinned"
						type="quiet"
						:color="pinned ? 'green' : undefined"
						@click="pinned = !pinned"
					>
						<PinIcon />
					</IconButton>
					<IconButton size="sm" label="Close review controls" type="quiet" @click="dismiss()">
						<XIcon />
					</IconButton>
				</div>
			</div>
			<div class="space-y-4 break-words [&_a]:break-all">
				<slot />
			</div>
		</div>
	</Teleport>
</template>

<script setup lang="ts">
import { autoUpdate, flip, offset, shift, size, useFloating } from '@floating-ui/vue'
import { PinIcon, XIcon } from '@modrinth/assets'
import { IconButton } from '@modrinth/ui'
import { useEventListener, useMutationObserver } from '@vueuse/core'
import { computed, nextTick, onBeforeUnmount, shallowRef, watch } from 'vue'

import { useFloatingReviewPanelAppearance } from '~/composables/floating-review-panel-appearance'

import { useReviewEventListener } from '../shortcuts'
import { injectReviewContext, type ReviewAnchor } from './context'

const { style: appearanceStyle } = useFloatingReviewPanelAppearance()
const props = defineProps<{
	anchor: ReviewAnchor
	titleId?: string
	label?: string
	maxWidth?: string
}>()
const {
	active,
	panel,
	panelId,
	pinned,
	close,
	leave,
	setPopoverHovered,
	cancelClose,
	contains,
	retainExternalLink,
} = injectReviewContext()
const element = shallowRef<HTMLElement | null>(null)
const header = shallowRef<HTMLElement | null>(null)
const reference = computed(() => props.anchor.element)
const viewport = computed(() => element.value?.ownerDocument.defaultView)
let widthDirty = true
const { floatingStyles, isPositioned, update } = useFloating(reference, element, {
	placement: 'right-start',
	strategy: 'fixed',
	whileElementsMounted: (reference, floating, update) => {
		widthDirty = true
		return autoUpdate(reference, floating, update)
	},
	middleware: [
		offset(10),
		flip({
			padding: 8,
			crossAxis: false,
			flipAlignment: false,
			fallbackPlacements: ['left-start', 'bottom-start', 'top-start'],
		}),
		shift({ padding: 8, crossAxis: false }),
		size({
			padding: 8,
			apply: ({ availableHeight, elements }) => {
				elements.floating.style.maxHeight = `${Math.max(0, availableHeight)}px`
				if (widthDirty) {
					widthDirty = false
					fitContentWidth(elements.floating)
				}
			},
		}),
	],
})
useMutationObserver(element, invalidateWidth, {
	childList: true,
	characterData: true,
	subtree: true,
})
useEventListener(viewport, 'resize', invalidateWidth)
useEventListener(() => viewport.value?.visualViewport, 'resize', invalidateWidth)
watch(
	[() => props.maxWidth, () => appearanceStyle.value.borderWidth],
	invalidateWidth,
	{ flush: 'post' },
)

function invalidateWidth() {
	if (widthDirty) return
	widthDirty = true
	update()
}

function fitContentWidth(popover: HTMLElement) {
	popover.style.width = 'max-content'
	const rows = popover.querySelectorAll<HTMLElement>('[data-review-control-row]')
	const view = popover.ownerDocument.defaultView
	if (!rows.length || !header.value || !view) return

	const outerWidth = popover.getBoundingClientRect().width
	const headerChildren = Array.from(header.value.children)
	const headerGap = Number.parseFloat(view.getComputedStyle(header.value).columnGap) || 0
	let width =
		outerWidth -
		header.value.getBoundingClientRect().width +
		headerChildren.reduce((total, child) => total + child.getBoundingClientRect().width, 0) +
		headerGap * Math.max(0, headerChildren.length - 1)

	for (const row of rows) {
		const bounds = row.getBoundingClientRect()
		const right = Math.max(
			bounds.left,
			...Array.from(row.children, (child) => child.getBoundingClientRect().right),
		)
		width = Math.max(width, outerWidth - bounds.width + right - bounds.left)
	}
	popover.style.width = `${Math.ceil(width)}px`
}

function dismiss() {
	if (active.value?.id === props.anchor.id) close()
}

function onKeydown(event: KeyboardEvent) {
	if (event.key !== 'Escape' || event.defaultPrevented) return
	event.preventDefault()
	event.stopPropagation()
	dismiss()
}

watch(element, (value, previous) => {
	if (active.value?.id === props.anchor.id) panel.value = value
	else if (panel.value === previous) panel.value = null
})
onBeforeUnmount(() => {
	if (panel.value === element.value) panel.value = null
})

watch([isPositioned, pinned], () => {
	if (!isPositioned.value || !pinned.value) return
	void nextTick(() => {
		if (pinned.value && active.value?.id === props.anchor.id)
			element.value?.focus({ preventScroll: true })
	})
})
useReviewEventListener('pointerdown', (event) => {
	if (pinned.value) return
	if (active.value?.id !== props.anchor.id || !(event.target instanceof Node)) return
	if (props.anchor.element.contains(event.target) || contains(event.target)) return
	const focused = props.anchor.element.ownerDocument.activeElement
	if (
		focused instanceof HTMLElement &&
		(props.anchor.element.contains(focused) || contains(focused))
	) {
		focused.blur()
	}
	dismiss()
})
</script>
