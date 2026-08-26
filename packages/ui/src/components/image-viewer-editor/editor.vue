<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'

import { injectNotificationManager } from '#ui/providers'
import { injectImageViewerEditor } from '#ui/providers/image-viewer-editor'

import Controls from './controls.vue'
import Toolbar from './toolbar.vue'
import type { ImageViewerEditorItem, ImageViewerEditorSavePayload } from './types'
import { useImageEditor } from './use-image-editor'

const props = defineProps<{
	item: ImageViewerEditorItem
	saving: boolean
}>()

const emit = defineEmits<{
	cancel: []
	save: [payload: ImageViewerEditorSavePayload]
}>()

const canvasElement = ref<HTMLCanvasElement>()
const viewport = ref<HTMLElement>()
const fitBounds = ref<HTMLElement>()
const exporting = ref(false)
const loadingEditorData = ref(false)
const spacePressed = ref(false)
const panning = ref<{ x: number; y: number; scrollLeft: number; scrollTop: number }>()
const brushPointer = ref({ x: 0, y: 0, visible: false })
const imageViewerEditor = injectImageViewerEditor()
const { handleError } = injectNotificationManager()
const editor = useImageEditor()
const {
	loading,
	tool,
	color,
	strokeWidth,
	eraserMode,
	zoom,
	isFit,
	initialize,
	dispose,
	setTool,
	fitToViewport,
	setZoom,
	exportPng,
	handleKeyboardShortcut,
	isTextEditing,
	resetHistory,
} = editor

const busy = computed(
	() => props.saving || exporting.value || loadingEditorData.value || loading.value,
)
const hasBrushPointer = computed(
	() =>
		tool.value === 'pen' ||
		tool.value === 'highlight' ||
		(tool.value === 'eraser' && eraserMode.value === 'area'),
)
const brushPointerSize = computed(() => Math.max(2, strokeWidth.value * zoom.value))
const brushPointerColor = computed(() => (tool.value === 'eraser' ? '#ffffff' : color.value))

let resizeObserver: ResizeObserver | undefined

async function initializeEditor() {
	const source = props.item.editorSource
	if (!source || !canvasElement.value) return

	loadingEditorData.value = true
	try {
		const editorData = await imageViewerEditor.loadEditorData(source)
		await initialize(canvasElement.value, editorData, getViewportSize())
		observeViewport()
	} catch (error) {
		handleError(error)
		emit('cancel')
	} finally {
		loadingEditorData.value = false
	}
}

function observeViewport() {
	resizeObserver?.disconnect()
	if (!viewport.value || !fitBounds.value) return
	resizeObserver = new ResizeObserver(fitEditorToViewport)
	resizeObserver.observe(viewport.value)
	resizeObserver.observe(fitBounds.value)
	fitEditorToViewport()
}

function fitEditorToViewport() {
	const viewportSize = getViewportSize()
	if (!viewportSize) return
	fitToViewport(viewportSize.width, viewportSize.height)
	if (isFit.value) {
		requestAnimationFrame(() => {
			if (!viewport.value || !isFit.value) return
			viewport.value.scrollLeft = 0
			viewport.value.scrollTop = 0
		})
	}
}

function getViewportSize() {
	if (!fitBounds.value) return undefined
	return {
		width: fitBounds.value.clientWidth,
		height: fitBounds.value.clientHeight,
	}
}

async function requestSave(mode: 'create_copy' | 'replace_edit') {
	if (busy.value) return
	exporting.value = true
	try {
		emit('save', {
			item: props.item,
			pngBytes: await exportPng(),
			mode,
		})
	} catch (error) {
		handleError(error)
	} finally {
		exporting.value = false
	}
}

function cancel() {
	if (!props.saving && !exporting.value) emit('cancel')
}

function handleKeydown(event: KeyboardEvent) {
	if (document.querySelector('.modal-root')) return
	if (handleKeyboardShortcut(event)) return
	if (event.code === 'Space' && !isTextEditing() && !isTypingTarget(event.target)) {
		spacePressed.value = true
	}
	if (event.key === 'Escape') {
		event.preventDefault()
		cancel()
	}
}

function handleKeyup(event: KeyboardEvent) {
	if (event.code === 'Space') spacePressed.value = false
}

function handleEditMenuUndo(event: Event) {
	if (isTextEditing() || isTypingTarget(document.activeElement)) return
	event.preventDefault()
	void editor.undo()
}

function handleEditMenuRedo(event: Event) {
	if (isTextEditing() || isTypingTarget(document.activeElement)) return
	event.preventDefault()
	void editor.redo()
}

function startPan(event: PointerEvent) {
	if (!viewport.value || (event.button !== 1 && !(event.button === 0 && spacePressed.value))) return
	event.preventDefault()
	event.stopPropagation()
	panning.value = {
		x: event.clientX,
		y: event.clientY,
		scrollLeft: viewport.value.scrollLeft,
		scrollTop: viewport.value.scrollTop,
	}
	viewport.value.setPointerCapture(event.pointerId)
}

function movePan(event: PointerEvent) {
	if (!viewport.value || !panning.value) return
	viewport.value.scrollLeft = panning.value.scrollLeft - (event.clientX - panning.value.x)
	viewport.value.scrollTop = panning.value.scrollTop - (event.clientY - panning.value.y)
}

function stopPan() {
	panning.value = undefined
}

function updateBrushPointer(event: PointerEvent) {
	if (!hasBrushPointer.value || panning.value) {
		brushPointer.value.visible = false
		return
	}
	const bounds = (event.currentTarget as HTMLElement).getBoundingClientRect()
	brushPointer.value = {
		x: event.clientX - bounds.left,
		y: event.clientY - bounds.top,
		visible: true,
	}
}

function handleWheel(event: WheelEvent) {
	if (!event.ctrlKey && !event.metaKey) return
	event.preventDefault()
	setZoom(zoom.value + (event.deltaY < 0 ? 0.1 : -0.1))
}

function isTypingTarget(target: EventTarget | null) {
	return (
		target instanceof HTMLInputElement ||
		target instanceof HTMLTextAreaElement ||
		target instanceof HTMLSelectElement ||
		(target instanceof HTMLElement && target.isContentEditable)
	)
}

function markSaved() {
	resetHistory()
}

onMounted(async () => {
	document.addEventListener('keydown', handleKeydown)
	document.addEventListener('keyup', handleKeyup)
	document.addEventListener('edit-menu:undo', handleEditMenuUndo)
	document.addEventListener('edit-menu:redo', handleEditMenuRedo)
	await nextTick()
	await initializeEditor()
})

onBeforeUnmount(() => {
	document.removeEventListener('keydown', handleKeydown)
	document.removeEventListener('keyup', handleKeyup)
	document.removeEventListener('edit-menu:undo', handleEditMenuUndo)
	document.removeEventListener('edit-menu:redo', handleEditMenuRedo)
	resizeObserver?.disconnect()
	dispose()
})

defineExpose({ markSaved })
</script>

<template>
	<div
		ref="viewport"
		class="editor-viewport absolute inset-0 flex min-h-0 min-w-0 overflow-auto"
		:class="{ 'is-panning': panning, 'is-pan-ready': spacePressed && !panning }"
		@click.stop
		@pointerdown.capture="startPan"
		@pointermove="movePan"
		@pointerup="stopPan"
		@pointercancel="stopPan"
		@wheel="handleWheel"
	>
		<div
			class="editor-canvas relative z-[2] m-auto shrink-0 transition-opacity duration-150"
			:class="{ 'opacity-0': loading || loadingEditorData }"
			@pointerenter="updateBrushPointer"
			@pointermove="updateBrushPointer"
			@pointerleave="brushPointer.visible = false"
		>
			<canvas ref="canvasElement" />
			<div
				v-if="hasBrushPointer && brushPointer.visible"
				class="pointer-events-none absolute z-10 -translate-x-1/2 -translate-y-1/2 rounded-full border border-solid shadow-[0_0_0_1px_rgb(0_0_0_/_80%),inset_0_0_0_1px_rgb(255_255_255_/_35%)]"
				:style="{
					left: `${brushPointer.x}px`,
					top: `${brushPointer.y}px`,
					width: `${brushPointerSize}px`,
					height: `${brushPointerSize}px`,
					borderColor: brushPointerColor,
				}"
			>
				<div
					v-if="tool !== 'eraser'"
					class="absolute inset-0 rounded-full opacity-[0.15]"
					:style="{ backgroundColor: color }"
				/>
			</div>
		</div>
	</div>
	<div
		ref="fitBounds"
		class="pointer-events-none absolute inset-x-6 bottom-[5.75rem] top-[4.75rem] z-[3] flex items-center justify-center max-[900px]:inset-x-4 max-[900px]:bottom-[8.5rem] max-[900px]:top-[4.5rem]"
		aria-hidden="true"
	>
		<img
			v-if="loading || loadingEditorData"
			class="max-h-full max-w-full rounded-xl object-contain shadow-[0_1.5rem_4rem_rgb(0_0_0_/_24%)]"
			:src="item.src"
			alt=""
		/>
	</div>

	<Toolbar :tool="tool" @select="setTool" />
	<Controls :editor="editor" :busy="busy" @cancel="cancel" @save="requestSave" />
</template>

<style scoped>
.editor-canvas :deep(.canvas-container) {
	flex: none;
	box-shadow: 0 1.5rem 4rem rgb(0 0 0 / 24%);
}

.editor-viewport.is-pan-ready :deep(.upper-canvas) {
	cursor: grab !important;
}

.editor-viewport.is-panning :deep(.upper-canvas) {
	cursor: grabbing !important;
}
</style>
