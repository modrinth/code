<template>
	<Teleport to="body">
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
import { onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'

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

const panelEl = ref<HTMLElement | null>(null)
const pos = reactive({ x: 0, y: 0 })
/** True once the moderator has dragged the panel — stop tracking the anchor after that. */
const detached = ref(false)

function clamp(x: number, y: number) {
	const el = panelEl.value
	const w = el?.offsetWidth ?? 352
	const h = el?.offsetHeight ?? 240
	return {
		x: Math.min(Math.max(x, 8), Math.max(8, window.innerWidth - w - 8)),
		y: Math.min(Math.max(y, 8), Math.max(8, window.innerHeight - h - 8)),
	}
}

function anchorToPos() {
	if (detached.value) return
	const rect = props.anchor?.getBoundingClientRect()
	const next = rect ? clamp(rect.left, rect.bottom + 6) : clamp(window.innerWidth / 2 - 176, 120)
	pos.x = next.x
	pos.y = next.y
}

let dragOffsetX = 0
let dragOffsetY = 0

function onDragMove(event: PointerEvent) {
	const next = clamp(event.clientX - dragOffsetX, event.clientY - dragOffsetY)
	pos.x = next.x
	pos.y = next.y
}

function onDragEnd() {
	window.removeEventListener('pointermove', onDragMove)
	window.removeEventListener('pointerup', onDragEnd)
}

function startDrag(event: PointerEvent) {
	if (event.button !== 0) return
	event.preventDefault()
	detached.value = true
	emit('setPin', true)
	dragOffsetX = event.clientX - pos.x
	dragOffsetY = event.clientY - pos.y
	window.addEventListener('pointermove', onDragMove)
	window.addEventListener('pointerup', onDragEnd)
}

function onKey(event: KeyboardEvent) {
	if (event.key === 'Escape') emit('close')
}

function onReflow() {
	anchorToPos()
}

onMounted(() => {
	anchorToPos()
	// A second pass once the panel has real dimensions for clamping.
	requestAnimationFrame(anchorToPos)
	window.addEventListener('keydown', onKey)
	window.addEventListener('resize', onReflow)
	window.addEventListener('scroll', onReflow, true)
})

onBeforeUnmount(() => {
	onDragEnd()
	window.removeEventListener('keydown', onKey)
	window.removeEventListener('resize', onReflow)
	window.removeEventListener('scroll', onReflow, true)
})

watch(() => props.anchor, anchorToPos)
</script>
