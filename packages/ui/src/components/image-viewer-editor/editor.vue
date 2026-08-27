<script setup lang="ts">
import {
	EditIcon,
	LeftArrowIcon,
	RightArrowIcon,
	XIcon,
	ZoomInIcon,
	ZoomOutIcon,
} from '@modrinth/assets'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import Button from '#ui/components/base/buttons/Button.vue'
import IconButton from '#ui/components/base/buttons/IconButton.vue'
import { useVIntl } from '#ui/composables/i18n'
import { injectNotificationManager } from '#ui/providers'

import Controls from './controls.vue'
import { imageViewerEditorMessages as messages } from './image-viewer-editor-messages'
import type { ImageViewerEditorMode } from './image-viewer-editor-types'
import Toolbar from './toolbar.vue'
import type {
	ImageViewerEditorData,
	ImageViewerEditorItem,
	ImageViewerEditorSavePayload,
} from './types'
import { useImageEditor } from './use-image-editor'

const CLICK_ZOOM = 2
const MAX_VIEW_ZOOM = 5
const ZOOM_STEP = 1.25

const props = defineProps<{
	item: ImageViewerEditorItem
	mode: ImageViewerEditorMode
	index: number
	count: number
	canEdit: boolean
	saving: boolean
	loadData: (item: ImageViewerEditorItem) => Promise<ImageViewerEditorData>
}>()

const emit = defineEmits<{
	cancel: []
	close: []
	edit: []
	imageReady: []
	next: []
	previous: []
	save: [payload: ImageViewerEditorSavePayload]
}>()

const canvasElement = ref<HTMLCanvasElement>()
const viewport = ref<HTMLElement>()
const fitBounds = ref<HTMLElement>()
const exporting = ref(false)
const discarding = ref(false)
const loadingEditorData = ref(true)
const spacePressed = ref(false)
const panning = ref<{
	x: number
	y: number
	scrollLeft: number
	scrollTop: number
	moved: boolean
}>()
const brushPointer = ref({ x: 0, y: 0, visible: false })
const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const editor = useImageEditor()
const {
	loading,
	tool,
	color,
	strokeWidth,
	eraserMode,
	zoom,
	fitScale,
	isFit,
	canZoomIn,
	initialize,
	dispose,
	setTool,
	setInteractionEnabled,
	discardChanges,
	fitToViewport,
	setZoom,
	setFit,
	exportPng,
	handleKeyboardShortcut,
	isTextEditing,
	resetHistory,
} = editor

const busy = computed(
	() =>
		props.saving || exporting.value || discarding.value || loadingEditorData.value || loading.value,
)
const hasBrushPointer = computed(
	() =>
		props.mode === 'edit' &&
		(tool.value === 'pen' ||
			tool.value === 'highlight' ||
			(tool.value === 'eraser' && eraserMode.value === 'area')),
)
const brushPointerSize = computed(() => Math.max(2, strokeWidth.value * zoom.value))
const brushPointerColor = computed(() => (tool.value === 'eraser' ? '#ffffff' : color.value))
const viewZoom = computed(() => zoom.value / Math.max(fitScale.value, Number.EPSILON))
const viewZoomPercent = computed(() => Math.round(viewZoom.value * 100))
const viewZoomed = computed(() => viewZoom.value > 1.001)
const canViewZoomIn = computed(() => canZoomIn.value && viewZoom.value < MAX_VIEW_ZOOM)

let resizeObserver: ResizeObserver | undefined
let initializationGeneration = 0
let initializationChain = Promise.resolve()
let imageReadyFrame: number | undefined

function queueInitialization() {
	const generation = ++initializationGeneration
	loadingEditorData.value = true
	const editorDataPromise = props.loadData(props.item)
	initializationChain = initializationChain.then(async () => {
		if (generation !== initializationGeneration) return
		let initialized = false
		try {
			const editorData = await editorDataPromise
			if (generation !== initializationGeneration || !canvasElement.value) return
			await initialize(canvasElement.value, editorData, getViewportSize())
			if (generation !== initializationGeneration) return
			setInteractionEnabled(props.mode === 'edit')
			observeViewport()
			initialized = true
		} catch (error) {
			if (generation !== initializationGeneration) return
			handleError(error)
			emit('close')
		} finally {
			if (generation === initializationGeneration) {
				loadingEditorData.value = false
				if (initialized) notifyImageReady()
			}
		}
	})
}

function notifyImageReady() {
	if (imageReadyFrame !== undefined) cancelAnimationFrame(imageReadyFrame)
	imageReadyFrame = requestAnimationFrame(() => {
		imageReadyFrame = undefined
		emit('imageReady')
	})
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
	if (isFit.value) centerViewport()
	else notifyImageReady()
}

function getViewportSize() {
	if (!fitBounds.value) return undefined
	return {
		width: fitBounds.value.clientWidth,
		height: fitBounds.value.clientHeight,
	}
}

function centerViewport() {
	requestAnimationFrame(() => {
		if (!viewport.value) return
		viewport.value.scrollLeft = Math.max(
			0,
			(viewport.value.scrollWidth - viewport.value.clientWidth) / 2,
		)
		viewport.value.scrollTop = Math.max(
			0,
			(viewport.value.scrollHeight - viewport.value.clientHeight) / 2,
		)
		notifyImageReady()
	})
}

function getTextContrast(target: HTMLElement): 'dark' | 'light' {
	const renderedCanvas = viewport.value?.querySelector<HTMLCanvasElement>('canvas.lower-canvas')
	const context = renderedCanvas?.getContext('2d', { willReadFrequently: true })
	if (!renderedCanvas || !context) return 'light'

	const targetBounds = target.getBoundingClientRect()
	const canvasBounds = renderedCanvas.getBoundingClientRect()
	const intersection = {
		left: Math.max(targetBounds.left, canvasBounds.left),
		top: Math.max(targetBounds.top, canvasBounds.top),
		right: Math.min(targetBounds.right, canvasBounds.right),
		bottom: Math.min(targetBounds.bottom, canvasBounds.bottom),
	}
	if (intersection.right <= intersection.left || intersection.bottom <= intersection.top)
		return 'light'

	const scaleX = renderedCanvas.width / canvasBounds.width
	const scaleY = renderedCanvas.height / canvasBounds.height
	const sourceX = Math.max(0, Math.floor((intersection.left - canvasBounds.left) * scaleX))
	const sourceY = Math.max(0, Math.floor((intersection.top - canvasBounds.top) * scaleY))
	const sourceWidth = Math.min(
		renderedCanvas.width - sourceX,
		Math.max(1, Math.ceil((intersection.right - intersection.left) * scaleX)),
	)
	const sourceHeight = Math.min(
		renderedCanvas.height - sourceY,
		Math.max(1, Math.ceil((intersection.bottom - intersection.top) * scaleY)),
	)

	try {
		const pixels = context.getImageData(sourceX, sourceY, sourceWidth, sourceHeight).data
		const sampleStride = Math.max(1, Math.floor(Math.sqrt((sourceWidth * sourceHeight) / 4096)))
		let luminanceTotal = 0
		let sampleCount = 0
		for (let y = 0; y < sourceHeight; y += sampleStride) {
			for (let x = 0; x < sourceWidth; x += sampleStride) {
				const offset = (y * sourceWidth + x) * 4
				const red = srgbToLinear(pixels[offset] / 255)
				const green = srgbToLinear(pixels[offset + 1] / 255)
				const blue = srgbToLinear(pixels[offset + 2] / 255)
				const alpha = pixels[offset + 3] / 255
				luminanceTotal += (0.2126 * red + 0.7152 * green + 0.0722 * blue) * alpha
				sampleCount++
			}
		}

		const targetArea = targetBounds.width * targetBounds.height
		const intersectionArea =
			(intersection.right - intersection.left) * (intersection.bottom - intersection.top)
		const coverage = targetArea > 0 ? intersectionArea / targetArea : 0
		if (coverage < 0.9) return 'light'
		const averageLuminance = sampleCount > 0 ? luminanceTotal / sampleCount : 0
		return averageLuminance > 0.179 ? 'dark' : 'light'
	} catch {
		return 'light'
	}
}

function srgbToLinear(value: number) {
	return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4
}

function resetView() {
	setFit()
	centerViewport()
}

function setViewZoom(nextZoom: number) {
	if (nextZoom <= 1.001) {
		resetView()
		return
	}
	setZoom(fitScale.value * Math.min(MAX_VIEW_ZOOM, nextZoom))
	centerViewport()
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

async function cancel() {
	if (busy.value) return
	discarding.value = true
	try {
		await discardChanges()
		emit('cancel')
	} catch (error) {
		handleError(error)
	} finally {
		discarding.value = false
	}
}

function handleKeydown(event: KeyboardEvent) {
	if (props.mode !== 'edit' || document.querySelector('.modal-root')) return
	if (handleKeyboardShortcut(event)) return
	if (event.code === 'Space' && !isTextEditing() && !isTypingTarget(event.target)) {
		spacePressed.value = true
	}
	if (event.key === 'Escape') {
		event.preventDefault()
		void cancel()
	}
}

function handleKeyup(event: KeyboardEvent) {
	if (event.code === 'Space') spacePressed.value = false
}

function handleEditMenuUndo(event: Event) {
	if (props.mode !== 'edit' || isTextEditing() || isTypingTarget(document.activeElement)) return
	event.preventDefault()
	void editor.undo()
}

function handleEditMenuRedo(event: Event) {
	if (props.mode !== 'edit' || isTextEditing() || isTypingTarget(document.activeElement)) return
	event.preventDefault()
	void editor.redo()
}

function handlePointerDown(event: PointerEvent) {
	if (!viewport.value) return
	const target = event.target
	if (props.mode === 'view' && event.button === 0 && target === event.currentTarget) {
		emit('close')
		return
	}
	const isCanvasTarget = target instanceof Element && Boolean(target.closest('.editor-canvas'))
	const isViewPan = props.mode === 'view' && event.button === 0 && isCanvasTarget
	const isEditPan =
		props.mode === 'edit' && (event.button === 1 || (event.button === 0 && spacePressed.value))
	if (!isViewPan && !isEditPan) return
	event.preventDefault()
	event.stopPropagation()
	panning.value = {
		x: event.clientX,
		y: event.clientY,
		scrollLeft: viewport.value.scrollLeft,
		scrollTop: viewport.value.scrollTop,
		moved: false,
	}
	viewport.value.setPointerCapture(event.pointerId)
}

function movePan(event: PointerEvent) {
	if (!viewport.value || !panning.value) return
	const deltaX = event.clientX - panning.value.x
	const deltaY = event.clientY - panning.value.y
	if (Math.abs(deltaX) > 2 || Math.abs(deltaY) > 2) panning.value.moved = true
	viewport.value.scrollLeft = panning.value.scrollLeft - deltaX
	viewport.value.scrollTop = panning.value.scrollTop - deltaY
}

function stopPan() {
	const pan = panning.value
	panning.value = undefined
	if (!pan || props.mode !== 'view') return
	if (pan.moved) {
		notifyImageReady()
		return
	}
	if (viewZoomed.value) resetView()
	else setViewZoom(CLICK_ZOOM)
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
	if (props.mode === 'view') {
		event.preventDefault()
		setViewZoom(viewZoom.value * (event.deltaY < 0 ? 1.1 : 1 / 1.1))
		return
	}
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

watch(() => [props.item.id, props.item.src], queueInitialization)
watch(
	() => props.mode,
	(mode) => {
		spacePressed.value = false
		panning.value = undefined
		brushPointer.value.visible = false
		setInteractionEnabled(mode === 'edit')
	},
)

onMounted(async () => {
	document.addEventListener('keydown', handleKeydown)
	document.addEventListener('keyup', handleKeyup)
	document.addEventListener('edit-menu:undo', handleEditMenuUndo)
	document.addEventListener('edit-menu:redo', handleEditMenuRedo)
	await nextTick()
	queueInitialization()
})

onBeforeUnmount(() => {
	initializationGeneration++
	if (imageReadyFrame !== undefined) cancelAnimationFrame(imageReadyFrame)
	document.removeEventListener('keydown', handleKeydown)
	document.removeEventListener('keyup', handleKeyup)
	document.removeEventListener('edit-menu:undo', handleEditMenuUndo)
	document.removeEventListener('edit-menu:redo', handleEditMenuRedo)
	resizeObserver?.disconnect()
	void dispose()
})

defineExpose({ getTextContrast, markSaved })
</script>

<template>
	<div
		ref="viewport"
		class="editor-viewport absolute inset-0 flex min-h-0 min-w-0 select-none overflow-auto px-6 pb-[5.75rem] pt-[4.75rem] max-[900px]:px-4 max-[900px]:pb-[8.5rem] max-[900px]:pt-[4.5rem]"
		:class="{
			'is-view': mode === 'view',
			'is-view-zoomed': mode === 'view' && viewZoomed,
			'is-panning': panning,
			'is-pan-ready': mode === 'edit' && spacePressed && !panning,
		}"
		@pointerdown.capture="handlePointerDown"
		@pointermove="movePan"
		@pointerup="stopPan"
		@pointercancel="panning = undefined"
		@wheel="handleWheel"
	>
		<img
			v-if="mode === 'view' && loadingEditorData"
			:src="item.src"
			:alt="item.alt"
			class="pointer-events-none relative z-[2] m-auto block max-h-full max-w-full shrink-0 object-contain"
			draggable="false"
		/>
		<div
			v-show="!loadingEditorData"
			class="editor-canvas relative z-[2] m-auto shrink-0"
			@pointerenter="updateBrushPointer"
			@pointermove="updateBrushPointer"
			@pointerleave="brushPointer.visible = false"
		>
			<canvas ref="canvasElement" :aria-label="item.alt" role="img" />
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
	/>

	<template v-if="mode === 'edit'">
		<Toolbar :tool="tool" @select="setTool" />
		<Controls :editor="editor" :busy="busy" @cancel="cancel" @save="requestSave" />
	</template>
	<div
		v-else
		class="absolute bottom-6 left-1/2 z-10 flex max-w-[calc(100%_-_3rem)] -translate-x-1/2 items-center gap-2 rounded-[20px] border border-solid border-white/10 bg-surface-3 p-2 shadow-[0_1rem_3rem_rgb(0_0_0_/_32%)]"
		@click.stop
	>
		<div v-if="count > 1" class="flex items-center gap-2">
			<IconButton :label="formatMessage(messages.previous)" type="quiet" @click="emit('previous')">
				<LeftArrowIcon aria-hidden="true" />
			</IconButton>
			<span
				class="flex min-w-14 justify-center gap-1 text-base font-semibold leading-5 tabular-nums text-white/50"
				aria-live="polite"
			>
				<strong class="font-semibold text-white">{{ index + 1 }}</strong>
				<span>/ {{ count }}</span>
			</span>
			<IconButton :label="formatMessage(messages.next)" type="quiet" @click="emit('next')">
				<RightArrowIcon aria-hidden="true" />
			</IconButton>
		</div>
		<div v-if="count > 1" class="h-6 w-px bg-white/10" />
		<div class="flex items-center gap-2">
			<IconButton
				v-tooltip="formatMessage(messages.zoomOut)"
				:label="formatMessage(messages.zoomOut)"
				type="quiet"
				:disabled="!viewZoomed"
				@click="setViewZoom(viewZoom / ZOOM_STEP)"
			>
				<ZoomOutIcon aria-hidden="true" />
			</IconButton>
			<Button
				v-tooltip="formatMessage(messages.fitToWorkspace)"
				type="quiet"
				class="w-16 px-2 tabular-nums text-white/60"
				@click="resetView"
			>
				{{ viewZoomPercent }}%
			</Button>
			<IconButton
				v-tooltip="formatMessage(messages.zoomIn)"
				:label="formatMessage(messages.zoomIn)"
				type="quiet"
				:disabled="!canViewZoomIn"
				@click="setViewZoom(viewZoom * ZOOM_STEP)"
			>
				<ZoomInIcon aria-hidden="true" />
			</IconButton>
		</div>
		<div class="h-6 w-px bg-white/10" />
		<IconButton
			v-if="canEdit"
			v-tooltip="formatMessage(messages.edit)"
			:label="formatMessage(messages.edit)"
			type="quiet"
			@click="emit('edit')"
		>
			<EditIcon aria-hidden="true" />
		</IconButton>
		<div class="flex items-center gap-2">
			<slot name="actions" />
		</div>
		<div class="h-6 w-px bg-white/10" />
		<IconButton :label="formatMessage(messages.close)" type="quiet" @click="emit('close')">
			<XIcon aria-hidden="true" />
		</IconButton>
	</div>
</template>

<style scoped>
.editor-viewport {
	scrollbar-width: none;

	&::-webkit-scrollbar {
		display: none;
	}
}

.editor-canvas :deep(.canvas-container) {
	flex: none;
	box-shadow: 0 1.5rem 4rem rgb(0 0 0 / 24%);
}

.editor-viewport.is-view :deep(.upper-canvas) {
	cursor: zoom-in !important;
}

.editor-viewport.is-view-zoomed :deep(.upper-canvas),
.editor-viewport.is-pan-ready :deep(.upper-canvas) {
	cursor: grab !important;
}

.editor-viewport.is-panning :deep(.upper-canvas) {
	cursor: grabbing !important;
}
</style>
