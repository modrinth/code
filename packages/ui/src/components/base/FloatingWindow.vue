<template>
	<Teleport to="body">
		<section
			v-if="open"
			ref="windowElement"
			role="dialog"
			tabindex="-1"
			:aria-label="title"
			class="floating-window fixed z-[9000] flex flex-col overflow-hidden rounded-2xl border border-solid border-surface-5 bg-surface-3 shadow-2xl outline-none"
			:style="windowStyle"
			@keydown.esc.stop="open = false"
		>
			<header
				class="flex shrink-0 cursor-move touch-none select-none items-center gap-2 border-0 border-b border-solid border-surface-5 py-1.5 pl-4 pr-1.5"
				@pointerdown="startDrag"
				@pointermove="drag"
				@pointerup="stopDrag"
				@pointercancel="stopDrag"
				@dblclick="recenter"
			>
				<component
					:is="icon"
					v-if="icon"
					class="size-4 shrink-0 text-secondary"
					aria-hidden="true"
				/>
				<h2 class="m-0 min-w-0 flex-1 truncate text-base font-semibold text-contrast">
					{{ title }}
				</h2>
				<div class="flex shrink-0 items-center gap-1" @pointerdown.stop>
					<slot name="actions" />
					<IconButton
						v-tooltip="formatMessage(commonMessages.closeButton)"
						type="quiet"
						size="sm"
						:label="formatMessage(commonMessages.closeButton)"
						@click="open = false"
					>
						<XIcon aria-hidden="true" />
					</IconButton>
				</div>
			</header>
			<div class="min-h-0 flex-1 overflow-auto p-4">
				<slot />
			</div>
		</section>
	</Teleport>
</template>

<script setup lang="ts">
import { XIcon } from '@modrinth/assets'
import { useLocalStorage, useResizeObserver } from '@vueuse/core'
import { type Component, computed, nextTick, ref, watch } from 'vue'

import { useVIntl } from '../../composables/i18n'
import { commonMessages } from '../../utils/common-messages'
import IconButton from './buttons/IconButton.vue'

/**
 * A window floating above the page without blocking it, moved by dragging its header and resized
 * from its corner, so whatever is behind it stays in reach. `storageKey` remembers where it was
 * left and how big it was.
 */
const props = withDefaults(
	defineProps<{
		title: string
		icon?: Component
		storageKey?: string
		/** The size the window opens at until it's resized, in pixels. */
		defaultSize?: { width: number; height: number }
	}>(),
	{ icon: undefined, storageKey: undefined, defaultSize: () => ({ width: 640, height: 560 }) },
)

const open = defineModel<boolean>('open', { default: false })

const { formatMessage } = useVIntl()

/** How much of the window is kept within the viewport while it's dragged. */
const KEPT_IN_VIEW = 48
const MINIMUM_WIDTH = 320
const MINIMUM_HEIGHT = 200

interface Placement {
	x: number | null
	y: number | null
	width: number
	height: number
}

const initialPlacement = (): Placement => ({ x: null, y: null, ...props.defaultSize })
const placement = props.storageKey
	? useLocalStorage<Placement>(props.storageKey, initialPlacement(), { mergeDefaults: true })
	: ref<Placement>(initialPlacement())

const windowElement = ref<HTMLElement | null>(null)

function viewport() {
	const view = windowElement.value?.ownerDocument.defaultView ?? window
	return { width: view.innerWidth, height: view.innerHeight }
}

function clampedSize() {
	const { width, height } = viewport()
	return {
		width: Math.max(MINIMUM_WIDTH, Math.min(placement.value.width, width - 16)),
		height: Math.max(MINIMUM_HEIGHT, Math.min(placement.value.height, height - 16)),
	}
}

function clampedPosition(x: number, y: number) {
	const { width, height } = viewport()
	const size = clampedSize()
	return {
		x: Math.min(Math.max(x, KEPT_IN_VIEW - size.width), width - KEPT_IN_VIEW),
		y: Math.min(Math.max(y, 0), height - KEPT_IN_VIEW),
	}
}

const windowStyle = computed(() => {
	const size = clampedSize()
	const { width, height } = viewport()
	const position = clampedPosition(
		placement.value.x ?? (width - size.width) / 2,
		placement.value.y ?? (height - size.height) / 2,
	)
	return {
		left: `${position.x}px`,
		top: `${position.y}px`,
		width: `${size.width}px`,
		height: `${size.height}px`,
		minWidth: `${MINIMUM_WIDTH}px`,
		minHeight: `${MINIMUM_HEIGHT}px`,
	}
})

let dragging: { pointerX: number; pointerY: number; x: number; y: number } | null = null

function startDrag(event: PointerEvent) {
	if (event.button !== 0 || !windowElement.value) return
	const bounds = windowElement.value.getBoundingClientRect()
	;(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId)
	dragging = { pointerX: event.clientX, pointerY: event.clientY, x: bounds.left, y: bounds.top }
}

function drag(event: PointerEvent) {
	if (!dragging) return
	const position = clampedPosition(
		dragging.x + event.clientX - dragging.pointerX,
		dragging.y + event.clientY - dragging.pointerY,
	)
	placement.value = { ...placement.value, ...position }
}

function stopDrag(event: PointerEvent) {
	if (!dragging) return
	dragging = null
	const target = event.currentTarget as HTMLElement
	if (target.hasPointerCapture(event.pointerId)) target.releasePointerCapture(event.pointerId)
}

function recenter() {
	placement.value = { ...placement.value, x: null, y: null }
}

useResizeObserver(windowElement, () => {
	const element = windowElement.value
	if (!element || dragging) return
	const width = element.offsetWidth
	const height = element.offsetHeight
	if (
		width !== Math.round(placement.value.width) ||
		height !== Math.round(placement.value.height)
	) {
		const { x, y } = clampedPosition(element.offsetLeft, element.offsetTop)
		placement.value = { x, y, width, height }
	}
})

watch(open, async (shown) => {
	if (!shown) return
	await nextTick()
	windowElement.value?.focus({ preventScroll: true })
})
</script>

<style scoped>
.floating-window {
	resize: both;
}
</style>
