<template>
	<Teleport :to="teleportTarget">
		<div
			ref="panelEl"
			class="fixed z-[70] flex max-h-[70vh] w-[28rem] max-w-[calc(100vw-1rem)] flex-col overflow-hidden rounded-lg border border-solid border-divider bg-bg-raised text-sm shadow-xl"
			:style="{ left: `${pos.x}px`, top: `${pos.y}px` }"
			@pointerenter="emit('hoverin')"
			@pointerleave="emit('hoverout')"
		>
			<div
				class="flex shrink-0 cursor-grab touch-none select-none items-center gap-1.5 border-0 border-b border-solid border-divider bg-surface-2 px-2 py-1.5 active:cursor-grabbing"
				@pointerdown="startDrag"
			>
				<GripVerticalIcon class="size-4 shrink-0 text-secondary" />
				<span
					class="min-w-0 flex-1 truncate text-xs font-semibold uppercase tracking-wide text-secondary"
				>
					{{ title }}
				</span>
				<button
					v-tooltip="pinned ? 'Following the button — click to detach' : 'Keep open'"
					class="rounded p-0.5 hover:bg-button-bg hover:text-contrast"
					:class="pinned ? 'text-brand' : 'text-secondary'"
					aria-label="Pin panel"
					@click="emit('togglePin')"
				>
					<PinIcon class="size-4" />
				</button>
				<button
					v-tooltip="'Close'"
					class="rounded p-0.5 text-secondary hover:bg-button-bg hover:text-contrast"
					aria-label="Close panel"
					@click="emit('close')"
				>
					<XIcon class="size-4" />
				</button>
			</div>

			<div class="min-h-0 flex-1 overflow-y-auto p-3">
				<slot />
			</div>
		</div>
	</Teleport>
</template>

<script setup lang="ts">
import { GripVerticalIcon, PinIcon, XIcon } from '@modrinth/assets'
import { computed, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'

const props = withDefaults(
	defineProps<{
		title: string
		anchor?: HTMLElement | null
		pinned?: boolean
	}>(),
	{ anchor: null, pinned: false },
)

const emit = defineEmits<{
	setPin: [boolean]
	togglePin: []
	close: []
	hoverin: []
	hoverout: []
}>()

// The anchor button can live inside the moderation review's Picture-in-Picture window, which
// shares the opener's JS realm — a bare `window`/`document` here would always mean the *main*
// window, teleporting this panel and sizing it against the wrong viewport. Derive both from the
// anchor element itself so this always operates against whichever window it actually lives in.
const teleportTarget = computed<HTMLElement | string>(
	() => props.anchor?.ownerDocument?.body ?? 'body',
)
const anchorWindow = computed<Window>(() => props.anchor?.ownerDocument?.defaultView ?? window)

const panelEl = ref<HTMLElement | null>(null)
const pos = reactive({ x: 0, y: 0 })
/** True once the moderator has dragged the panel — stop tracking the anchor after that. */
const detached = ref(false)

function clamp(x: number, y: number) {
	const el = panelEl.value
	const win = anchorWindow.value
	const w = el?.offsetWidth ?? 352
	const h = el?.offsetHeight ?? 240
	return {
		x: Math.min(Math.max(x, 8), Math.max(8, win.innerWidth - w - 8)),
		y: Math.min(Math.max(y, 8), Math.max(8, win.innerHeight - h - 8)),
	}
}

function anchorToPos() {
	if (detached.value) return
	console.log(props.anchor)
	const rect = props.anchor?.getBoundingClientRect()
	const next = rect
		? clamp(rect.left, rect.bottom + 6)
		: clamp(anchorWindow.value.innerWidth / 2 - 176, 120)
	pos.x = next.x
	pos.y = next.y
}

let dragOffsetX = 0
let dragOffsetY = 0
/** The window a drag started in — kept stable for the drag's duration regardless of `anchorWindow`. */
let dragWindow: Window = window

function onDragMove(event: PointerEvent) {
	const next = clamp(event.clientX - dragOffsetX, event.clientY - dragOffsetY)
	pos.x = next.x
	pos.y = next.y
}

function onDragEnd() {
	dragWindow.removeEventListener('pointermove', onDragMove)
	dragWindow.removeEventListener('pointerup', onDragEnd)
}

function startDrag(event: PointerEvent) {
	if (event.button !== 0) return
	event.preventDefault()
	detached.value = true
	emit('setPin', true)
	dragOffsetX = event.clientX - pos.x
	dragOffsetY = event.clientY - pos.y
	dragWindow = anchorWindow.value
	dragWindow.addEventListener('pointermove', onDragMove)
	dragWindow.addEventListener('pointerup', onDragEnd)
}

function onKey(event: KeyboardEvent) {
	if (event.key === 'Escape') emit('close')
}

function onReflow() {
	anchorToPos()
}

/** The window `onKey`/`onReflow` listeners were registered against — matched on cleanup. */
let listenerWindow: Window = window

onMounted(() => {
	anchorToPos()
	// A second pass once the panel has real dimensions for clamping.
	anchorWindow.value.requestAnimationFrame(anchorToPos)
	listenerWindow = anchorWindow.value
	listenerWindow.addEventListener('keydown', onKey)
	listenerWindow.addEventListener('resize', onReflow)
	listenerWindow.addEventListener('scroll', onReflow, true)
})

onBeforeUnmount(() => {
	onDragEnd()
	listenerWindow.removeEventListener('keydown', onKey)
	listenerWindow.removeEventListener('resize', onReflow)
	listenerWindow.removeEventListener('scroll', onReflow, true)
})

watch(() => props.anchor, anchorToPos)
</script>
