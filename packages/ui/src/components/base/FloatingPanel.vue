<script setup lang="ts">
import { onClickOutside } from '@vueuse/core'
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'

import ButtonStyled from './ButtonStyled.vue'

const PANEL_VIEWPORT_MARGIN = 8

const props = withDefaults(
	defineProps<{
		placement?: 'bottom-start' | 'bottom-end' | 'top-start' | 'top-end'
		distance?: number
		disabled?: boolean
		buttonClass?: string
		panelClass?: string
	}>(),
	{
		placement: 'bottom-end',
		distance: 8,
		disabled: false,
	},
)

const emit = defineEmits<{
	open: []
	close: []
}>()

const isOpen = ref(false)
const triggerRef = ref<HTMLElement>()
const panelRef = ref<HTMLElement>()
const rafId = ref<number | null>(null)

const openDirection = ref<'up' | 'down'>('down')
const horizontalAlignment = ref<'start' | 'end'>('end')

const panelStyle = ref({
	top: '0px',
	left: '0px',
})

const transformOrigin = computed(() => {
	const vertical = openDirection.value === 'down' ? 'top' : 'bottom'
	const horizontal = horizontalAlignment.value === 'end' ? 'right' : 'left'
	return `${vertical} ${horizontal}`
})

function determineOpenDirection(
	triggerRect: DOMRect,
	panelRect: DOMRect,
	viewportHeight: number,
): 'up' | 'down' {
	const preferDown = props.placement.startsWith('bottom')

	const hasSpaceBelow =
		triggerRect.bottom + props.distance + panelRect.height + PANEL_VIEWPORT_MARGIN <= viewportHeight
	const hasSpaceAbove =
		triggerRect.top - props.distance - panelRect.height - PANEL_VIEWPORT_MARGIN >= 0

	if (preferDown) {
		return hasSpaceBelow ? 'down' : hasSpaceAbove ? 'up' : 'down'
	} else {
		return hasSpaceAbove ? 'up' : hasSpaceBelow ? 'down' : 'up'
	}
}

function calculateVerticalPosition(
	triggerRect: DOMRect,
	panelRect: DOMRect,
	direction: 'up' | 'down',
): number {
	return direction === 'up'
		? triggerRect.top - panelRect.height - props.distance
		: triggerRect.bottom + props.distance
}

function calculateHorizontalPosition(
	triggerRect: DOMRect,
	panelRect: DOMRect,
	viewportWidth: number,
): number {
	const alignEnd = props.placement.endsWith('end')
	let left: number

	if (alignEnd) {
		left = triggerRect.right - panelRect.width
	} else {
		left = triggerRect.left
	}

	if (left + panelRect.width > viewportWidth - PANEL_VIEWPORT_MARGIN) {
		left = Math.max(PANEL_VIEWPORT_MARGIN, viewportWidth - panelRect.width - PANEL_VIEWPORT_MARGIN)
	}
	if (left < PANEL_VIEWPORT_MARGIN) {
		left = PANEL_VIEWPORT_MARGIN
	}

	return left
}

async function updatePanelPosition() {
	if (!triggerRef.value || !panelRef.value) return

	await nextTick()

	const triggerRect = triggerRef.value.getBoundingClientRect()
	const panelRect = panelRef.value.getBoundingClientRect()
	const viewportHeight = window.innerHeight
	const viewportWidth = window.innerWidth

	const direction = determineOpenDirection(triggerRect, panelRect, viewportHeight)
	const top = calculateVerticalPosition(triggerRect, panelRect, direction)
	const left = calculateHorizontalPosition(triggerRect, panelRect, viewportWidth)

	panelStyle.value = {
		top: `${top}px`,
		left: `${left}px`,
	}

	openDirection.value = direction
	horizontalAlignment.value = props.placement.endsWith('end') ? 'end' : 'start'
}

function startPositionTracking() {
	function track() {
		updatePanelPosition()
		rafId.value = requestAnimationFrame(track)
	}
	rafId.value = requestAnimationFrame(track)
}

function stopPositionTracking() {
	if (rafId.value !== null) {
		cancelAnimationFrame(rafId.value)
		rafId.value = null
	}
}

function focusPanelContent() {
	if (!panelRef.value) return

	const focusable = panelRef.value.querySelector<HTMLElement>(
		'button:not([data-focus-trap]), [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
	)
	if (focusable) {
		focusable.focus()
	}
}

async function open() {
	if (props.disabled || isOpen.value) return

	isOpen.value = true
	emit('open')

	await nextTick()
	await updatePanelPosition()
	startPositionTracking()

	setTimeout(() => {
		focusPanelContent()
	}, 50)
}

function close() {
	if (!isOpen.value) return

	stopPositionTracking()
	isOpen.value = false
	emit('close')

	nextTick(() => {
		triggerRef.value?.focus()
	})
}

function toggle() {
	if (isOpen.value) {
		close()
	} else {
		open()
	}
}

onClickOutside(
	panelRef,
	() => {
		close()
	},
	{ ignore: [triggerRef, '#teleports'] },
)

function handleTriggerKeydown(event: KeyboardEvent) {
	switch (event.key) {
		case 'Enter':
		case ' ':
			event.preventDefault()
			toggle()
			break
		case 'ArrowDown':
			event.preventDefault()
			open()
			break
		case 'ArrowUp':
			event.preventDefault()
			open()
			break
		case 'Escape':
			if (isOpen.value) {
				event.preventDefault()
				close()
			}
			break
	}
}

function handlePanelKeydown(event: KeyboardEvent) {
	if (event.key === 'Escape') {
		event.preventDefault()
		close()
	}
}

function handleWindowResize() {
	if (isOpen.value) {
		updatePanelPosition()
	}
}

onMounted(() => {
	window.addEventListener('resize', handleWindowResize)
})

onUnmounted(() => {
	window.removeEventListener('resize', handleWindowResize)
	stopPositionTracking()
})

defineExpose({
	open,
	close,
	toggle,
})
</script>

<template>
	<div class="relative inline-block">
		<ButtonStyled v-bind="$attrs">
			<button
				ref="triggerRef"
				:class="buttonClass"
				:disabled="disabled"
				:aria-expanded="isOpen"
				aria-haspopup="true"
				@click="toggle"
				@keydown="handleTriggerKeydown"
			>
				<slot></slot>
			</button>
		</ButtonStyled>

		<Teleport to="body">
			<Transition
				enter-active-class="floating-panel-enter-active"
				enter-from-class="floating-panel-enter-from"
				enter-to-class="floating-panel-enter-to"
				leave-active-class="floating-panel-leave-active"
				leave-from-class="floating-panel-leave-from"
				leave-to-class="floating-panel-leave-to"
			>
				<div
					v-if="isOpen"
					ref="panelRef"
					class="fixed z-[9995] w-fit rounded-[14px] border border-surface-5 bg-surface-3 border-solid border-px p-3 shadow-2xl"
					:class="panelClass"
					:style="[panelStyle, { transformOrigin }]"
					role="dialog"
					tabindex="-1"
					@keydown="handlePanelKeydown"
					@mousedown.stop
				>
					<button class="sr-only" data-focus-trap @focusin="close"></button>
					<slot name="panel"></slot>
					<button class="sr-only" data-focus-trap @focusin="close"></button>
				</div>
			</Transition>
		</Teleport>
	</div>
</template>

<style scoped>
/* .floating-panel-enter-active,
.floating-panel-leave-active {
	transition:
		transform 0.125s ease-in-out,
		opacity 0.125s ease-in-out;
}

.floating-panel-enter-from,
.floating-panel-leave-to {
	transform: scale(0.85);
	opacity: 0;
}

.floating-panel-enter-to,
.floating-panel-leave-from {
	transform: scale(1);
	opacity: 1;
} */
</style>
