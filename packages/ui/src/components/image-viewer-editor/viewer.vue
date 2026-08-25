<script setup lang="ts">
import {
	EditIcon,
	LeftArrowIcon,
	RightArrowIcon,
	XIcon,
	ZoomInIcon,
	ZoomOutIcon,
} from '@modrinth/assets'
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'

import Button from '#ui/components/base/buttons/Button.vue'
import IconButton from '#ui/components/base/buttons/IconButton.vue'
import { useVIntl } from '#ui/composables/i18n'

import { imageViewerEditorMessages } from './image-viewer-editor-messages'
import type { ImageViewerEditorItem } from './types'

type FabricModule = typeof import('fabric')
type FabricCanvas = import('fabric').Canvas
type FabricImage = import('fabric').FabricImage
type ViewerCanvasEvents = import('fabric').CanvasEvents
type ViewerDrag = {
	startX: number
	startY: number
	viewportX: number
	viewportY: number
}
type ImageDisplayBounds = {
	left: number
	top: number
	width: number
	height: number
}

const DEFAULT_ZOOM = 1
const CLICK_ZOOM = 2
const MAX_ZOOM = 5
const ZOOM_STEP = 1.25

const props = defineProps<{
	item: ImageViewerEditorItem
	index: number
	count: number
	canEdit: boolean
}>()

const emit = defineEmits<{
	close: []
	edit: []
	next: []
	previous: []
}>()

const { formatMessage } = useVIntl()
const viewport = ref<HTMLElement>()
const fitBounds = ref<HTMLElement>()
const canvasElement = ref<HTMLCanvasElement>()
const zoom = ref(DEFAULT_ZOOM)

let fabric: FabricModule | undefined
let canvas: FabricCanvas | undefined
let image: FabricImage | undefined
let imageDisplayBounds: ImageDisplayBounds = { left: 0, top: 0, width: 0, height: 0 }
let drag: ViewerDrag | undefined
let zoomedOnPointerDown = false
let resizeObserver: ResizeObserver | undefined
let loadGeneration = 0
let disposed = false

watch(
	() => props.item.src,
	(source) => {
		if (canvas) void loadViewerImage(source)
	},
)
watch(() => props.item.id, resetView)

async function initializeViewer() {
	if (!canvasElement.value || !viewport.value) return
	fabric = await import('fabric')
	if (disposed || !canvasElement.value || !viewport.value) return

	const bounds = viewport.value.getBoundingClientRect()
	canvas = new fabric.Canvas(canvasElement.value, {
		width: Math.round(bounds.width),
		height: Math.round(bounds.height),
		enableRetinaScaling: false,
		selection: false,
		preserveObjectStacking: true,
		defaultCursor: 'default',
		hoverCursor: 'zoom-in',
		moveCursor: 'grab',
	})
	bindCanvasEvents(canvas)
	resizeObserver = new ResizeObserver(resizeViewer)
	resizeObserver.observe(viewport.value)
	if (fitBounds.value) resizeObserver.observe(fitBounds.value)
	await loadViewerImage(props.item.src)
}

async function loadViewerImage(source: string) {
	const viewerCanvas = canvas
	const viewerFabric = fabric
	if (!viewerCanvas || !viewerFabric) return
	const generation = ++loadGeneration

	try {
		const element = await loadImage(source)
		if (generation !== loadGeneration || disposed || canvas !== viewerCanvas) return
		viewerCanvas.clear()
		image = new viewerFabric.FabricImage(element, {
			originX: 'center',
			originY: 'center',
			selectable: false,
			evented: true,
			hoverCursor: 'zoom-in',
		})
		viewerCanvas.add(image)
		fitImage()
	} catch {
		if (generation !== loadGeneration || canvas !== viewerCanvas) return
		image = undefined
		viewerCanvas.clear()
		viewerCanvas.requestRenderAll()
	}
}

function loadImage(source: string) {
	return new Promise<HTMLImageElement>((resolve, reject) => {
		const element = new Image()
		element.decoding = 'async'
		element.onload = () => resolve(element)
		element.onerror = () => reject(new Error('Failed to load image'))
		element.src = source
	})
}

function fitImage() {
	if (!canvas || !image || !viewport.value) return
	const viewportRect = viewport.value.getBoundingClientRect()
	const fitRect = fitBounds.value?.getBoundingClientRect() ?? viewportRect
	const imageWidth = image.width || 1
	const imageHeight = image.height || 1
	const scale = Math.min(1, fitRect.width / imageWidth, fitRect.height / imageHeight)
	const centerX = fitRect.left - viewportRect.left + fitRect.width / 2
	const centerY = fitRect.top - viewportRect.top + fitRect.height / 2
	imageDisplayBounds = {
		left: centerX - (imageWidth * scale) / 2,
		top: centerY - (imageHeight * scale) / 2,
		width: imageWidth * scale,
		height: imageHeight * scale,
	}
	image.set({
		left: centerX,
		top: centerY,
		scaleX: scale,
		scaleY: scale,
	})
	image.setCoords()
	resetView()
}

function resetView() {
	if (!canvas) return
	zoom.value = DEFAULT_ZOOM
	canvas.setViewportTransform([DEFAULT_ZOOM, 0, 0, DEFAULT_ZOOM, 0, 0])
	updateCursor('zoom-in')
	canvas.requestRenderAll()
}

function zoomAt(point: { x: number; y: number }, nextZoom: number) {
	if (!canvas || !fabric || !image) return
	const normalizedZoom = Math.min(MAX_ZOOM, Math.max(DEFAULT_ZOOM, nextZoom))
	if (normalizedZoom === DEFAULT_ZOOM) {
		resetView()
		return
	}
	canvas.zoomToPoint(new fabric.Point(point.x, point.y), normalizedZoom)
	zoom.value = normalizedZoom
	clampViewport()
	updateCursor('grab')
}

function setZoom(nextZoom: number) {
	if (!canvas) return
	zoomAt({ x: canvas.getWidth() / 2, y: canvas.getHeight() / 2 }, nextZoom)
}

function bindCanvasEvents(viewerCanvas: FabricCanvas) {
	viewerCanvas.on('mouse:down', handlePointerDown)
	viewerCanvas.on('mouse:move', handlePointerMove)
	viewerCanvas.on('mouse:up', handlePointerUp)
	viewerCanvas.on('mouse:dblclick', handleDoubleClick)
	viewerCanvas.on('mouse:wheel', handleWheel)
}

function handlePointerDown(event: ViewerCanvasEvents['mouse:down']) {
	if (!canvas || event.target !== image) return
	const pointer = clientPoint(event.e)
	if (!pointer || (event.e instanceof MouseEvent && event.e.button !== 0)) return
	zoomedOnPointerDown = canvas.getZoom() > 1
	if (!zoomedOnPointerDown) return
	const viewportTransform = canvas.viewportTransform
	drag = {
		startX: pointer.x,
		startY: pointer.y,
		viewportX: viewportTransform[4],
		viewportY: viewportTransform[5],
	}
	updateCursor('grabbing')
}

function handlePointerMove(event: ViewerCanvasEvents['mouse:move']) {
	if (!canvas || !drag) return
	const pointer = clientPoint(event.e)
	if (!pointer) return
	const viewportTransform = [...canvas.viewportTransform]
	viewportTransform[4] = drag.viewportX + pointer.x - drag.startX
	viewportTransform[5] = drag.viewportY + pointer.y - drag.startY
	canvas.setViewportTransform(viewportTransform)
	clampViewport()
}

function handlePointerUp(event: ViewerCanvasEvents['mouse:up']) {
	if (!canvas) return
	drag = undefined
	if (event.isClick && !event.target) {
		emit('close')
	} else if (event.isClick && event.target === image) {
		if (zoomedOnPointerDown) resetView()
		else zoomAt(event.viewportPoint, CLICK_ZOOM)
	} else {
		updateCursor(canvas.getZoom() > DEFAULT_ZOOM ? 'grab' : 'zoom-in')
	}
	zoomedOnPointerDown = false
}

function handleDoubleClick(event: ViewerCanvasEvents['mouse:dblclick']) {
	event.e.preventDefault()
	resetView()
}

function handleWheel(event: ViewerCanvasEvents['mouse:wheel']) {
	if (!canvas) return
	event.e.preventDefault()
	event.e.stopPropagation()
	const sensitivity = event.e.deltaMode === WheelEvent.DOM_DELTA_PIXEL ? 0.002 : 0.08
	zoomAt(event.viewportPoint, canvas.getZoom() * Math.exp(-event.e.deltaY * sensitivity))
}

function clampViewport() {
	if (!canvas || !image) return
	const zoom = canvas.getZoom()
	const width = canvas.getWidth()
	const height = canvas.getHeight()
	const viewportTransform = [...canvas.viewportTransform]
	viewportTransform[4] = clampViewportAxis(
		viewportTransform[4],
		width,
		imageDisplayBounds.left,
		imageDisplayBounds.left + imageDisplayBounds.width,
		zoom,
	)
	viewportTransform[5] = clampViewportAxis(
		viewportTransform[5],
		height,
		imageDisplayBounds.top,
		imageDisplayBounds.top + imageDisplayBounds.height,
		zoom,
	)
	canvas.setViewportTransform(viewportTransform)
	canvas.requestRenderAll()
}

function clampViewportAxis(
	translation: number,
	canvasSize: number,
	imageStart: number,
	imageEnd: number,
	zoom: number,
) {
	const imageSize = imageEnd - imageStart
	const imageCenter = imageStart + imageSize / 2
	if (imageSize * zoom <= canvasSize) return imageCenter - imageCenter * zoom
	return Math.min(-imageStart * zoom, Math.max(canvasSize - imageEnd * zoom, translation))
}

function updateCursor(cursor: 'zoom-in' | 'grab' | 'grabbing') {
	if (!canvas) return
	canvas.hoverCursor = cursor
	canvas.moveCursor = cursor
	if (image) image.hoverCursor = cursor
	canvas.setCursor(cursor)
}

function resizeViewer() {
	if (!canvas || !viewport.value) return
	const bounds = viewport.value.getBoundingClientRect()
	const width = Math.round(bounds.width)
	const height = Math.round(bounds.height)
	if (width < 1 || height < 1) return
	canvas.setDimensions({ width, height })
	fitImage()
}

function clientPoint(event: MouseEvent | TouchEvent | PointerEvent) {
	if (event instanceof MouseEvent) return { x: event.clientX, y: event.clientY }
	const touch = event.touches[0] ?? event.changedTouches[0]
	return touch ? { x: touch.clientX, y: touch.clientY } : undefined
}

onMounted(() => void initializeViewer())

onBeforeUnmount(() => {
	disposed = true
	loadGeneration++
	resizeObserver?.disconnect()
	void canvas?.dispose()
	canvas = undefined
	image = undefined
})
</script>

<template>
	<div ref="viewport" class="absolute inset-0 overflow-hidden" @click.stop>
		<canvas ref="canvasElement" :aria-label="item.alt" role="img" />
	</div>
	<div
		ref="fitBounds"
		class="pointer-events-none absolute inset-x-6 bottom-[5.75rem] top-[4.75rem]"
		aria-hidden="true"
	/>

	<div
		class="absolute bottom-6 left-1/2 z-10 flex max-w-[calc(100%_-_3rem)] -translate-x-1/2 items-center gap-2 rounded-[20px] border border-solid border-white/10 bg-surface-3 px-3 py-2.5 shadow-[0_1rem_3rem_rgb(0_0_0_/_32%)]"
		@click.stop
	>
		<div v-if="count > 1" class="flex items-center gap-2">
			<IconButton
				:label="formatMessage(imageViewerEditorMessages.previous)"
				type="quiet"
				@click="emit('previous')"
			>
				<LeftArrowIcon aria-hidden="true" />
			</IconButton>
			<span
				class="flex min-w-14 justify-center gap-1 text-sm tabular-nums text-white/50"
				aria-live="polite"
			>
				<strong class="font-semibold text-white">{{ index + 1 }}</strong>
				<span>/ {{ count }}</span>
			</span>
			<IconButton
				:label="formatMessage(imageViewerEditorMessages.next)"
				type="quiet"
				@click="emit('next')"
			>
				<RightArrowIcon aria-hidden="true" />
			</IconButton>
		</div>
		<div v-if="count > 1" class="h-6 w-px bg-white/10" />
		<div class="flex items-center gap-1">
			<IconButton
				v-tooltip="formatMessage(imageViewerEditorMessages.zoomOut)"
				:label="formatMessage(imageViewerEditorMessages.zoomOut)"
				type="quiet"
				:disabled="zoom <= DEFAULT_ZOOM"
				@click="setZoom(zoom / ZOOM_STEP)"
			>
				<ZoomOutIcon aria-hidden="true" />
			</IconButton>
			<Button
				v-tooltip="formatMessage(imageViewerEditorMessages.fitToWorkspace)"
				type="quiet"
				class="w-16 px-2 text-sm tabular-nums text-white/60"
				@click="resetView"
			>
				{{ Math.round(zoom * 100) }}%
			</Button>
			<IconButton
				v-tooltip="formatMessage(imageViewerEditorMessages.zoomIn)"
				:label="formatMessage(imageViewerEditorMessages.zoomIn)"
				type="quiet"
				:disabled="zoom >= MAX_ZOOM"
				@click="setZoom(zoom * ZOOM_STEP)"
			>
				<ZoomInIcon aria-hidden="true" />
			</IconButton>
		</div>
		<div class="h-6 w-px bg-white/10" />
		<IconButton
			v-if="canEdit"
			v-tooltip="formatMessage(imageViewerEditorMessages.edit)"
			:label="formatMessage(imageViewerEditorMessages.edit)"
			type="quiet"
			@click="emit('edit')"
		>
			<EditIcon aria-hidden="true" />
		</IconButton>
		<div class="flex items-center gap-2">
			<slot name="actions" />
		</div>
		<div class="h-6 w-px bg-white/10" />
		<IconButton
			:label="formatMessage(imageViewerEditorMessages.close)"
			type="quiet"
			@click="emit('close')"
		>
			<XIcon aria-hidden="true" />
		</IconButton>
	</div>
</template>
