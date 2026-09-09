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
	waitForRender,
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
const nativeImageView = computed(
	() =>
		props.mode === 'view' &&
		[props.item.src, props.item.editorSource?.path].some(isGifOrWebpSource),
)

let resizeObserver: ResizeObserver | undefined
let initializationGeneration = 0
let initializationChain = Promise.resolve()

function isGifOrWebpSource(source?: string) {
	if (!source) return false
	let pathname = source
	try {
		pathname = new URL(source, 'https://modrinth.invalid').pathname
	} catch {
		// ... ignore
	}
	try {
		pathname = decodeURIComponent(pathname)
	} catch {
		// ... ignore
	}
	return /\.(?:gif|webp)$/i.test(pathname)
}

function queueInitialization() {
	const generation = ++initializationGeneration
	loadingEditorData.value = true
	const editorDataPromise = props.loadData(props.item)
	initializationChain = initializationChain.then(async () => {
		if (generation !== initializationGeneration) return
		try {
			const editorData = await editorDataPromise
			if (generation !== initializationGeneration || !canvasElement.value) return
			await initialize(canvasElement.value, editorData, getViewportSize())
			if (generation !== initializationGeneration) return
			setInteractionEnabled(props.mode === 'edit')
			observeViewport()
			await waitForRender()
			if (generation !== initializationGeneration) return
		} catch (error) {
			if (generation !== initializationGeneration) return
			handleError(error)
			emit('close')
		} finally {
			if (generation === initializationGeneration) {
				loadingEditorData.value = false
			}
		}
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
	})
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
	if (pan.moved) return
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
	document.removeEventListener('keydown', handleKeydown)
	document.removeEventListener('keyup', handleKeyup)
	document.removeEventListener('edit-menu:undo', handleEditMenuUndo)
	document.removeEventListener('edit-menu:redo', handleEditMenuRedo)
	resizeObserver?.disconnect()
	void dispose()
})

defineExpose({ markSaved })
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
			v-if="mode === 'view' && loadingEditorData && !nativeImageView"
			:src="item.src"
			:alt="item.alt"
			class="pointer-events-none relative z-[2] m-auto block h-full w-full shrink-0 object-contain"
			draggable="false"
		/>
		<div
			v-show="nativeImageView || !loadingEditorData"
			class="editor-canvas relative z-[2] m-auto shrink-0"
			:class="{
				'h-full w-full': nativeImageView && loadingEditorData,
				'is-native-image-view': nativeImageView,
			}"
			@pointerenter="updateBrushPointer"
			@pointermove="updateBrushPointer"
			@pointerleave="brushPointer.visible = false"
		>
			<canvas ref="canvasElement" :aria-label="item.alt" role="img" />
			<img
				v-if="nativeImageView"
				:src="item.src"
				:alt="item.alt"
				class="pointer-events-none absolute inset-0 z-[2] h-full w-full object-contain"
				draggable="false"
			/>
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
				class="flex min-w-14 justify-center gap-1 text-base font-semibold leading-5 tabular-nums text-secondary"
				aria-live="polite"
			>
				<strong class="font-semibold text-contrast">{{ index + 1 }}</strong>
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
				class="w-16 px-2 tabular-nums text-secondary"
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

.editor-canvas.is-native-image-view :deep(.canvas-container) {
	visibility: hidden;
}

.editor-viewport.is-view .editor-canvas.is-native-image-view {
	cursor: zoom-in;
}

.editor-viewport.is-view-zoomed .editor-canvas.is-native-image-view {
	cursor: grab;
}

.editor-viewport.is-panning .editor-canvas.is-native-image-view {
	cursor: grabbing;
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
