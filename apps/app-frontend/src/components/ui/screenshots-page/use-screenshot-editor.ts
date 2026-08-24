import { readFile } from '@tauri-apps/plugin-fs'
import { computed, ref, shallowRef } from 'vue'

import type { InstanceScreenshot } from '@/helpers/instance'

import { renderCensorRegion } from './censor-object'
import type {
	EditorHistoryEntry,
	ScreenshotCensorMode,
	ScreenshotEditorObjectKind,
	ScreenshotEditorObjectState,
	ScreenshotEditorPropertyKind,
	ScreenshotEditorSourceRect,
	ScreenshotEditorTool,
} from './editor-types'

type FabricModule = typeof import('fabric')
type FabricCanvas = import('fabric').Canvas
type FabricObject = import('fabric').FabricObject
type FabricImage = import('fabric').FabricImage
type FabricGroup = import('fabric').Group
type EditorFabricObject = FabricObject & {
	editorKind?: ScreenshotEditorObjectKind
	fontSize?: number
	sourceRect?: ScreenshotEditorSourceRect
}

type FabricPointerEvent = {
	e: MouseEvent | TouchEvent | PointerEvent
}

const MIN_ZOOM = 0.25
const MAX_ZOOM = 4
const CONTROL_SIZE = 14
const CONTROL_TOUCH_SIZE = 24
const SELECTION_COLOR = '#1bd96a'
const TOOL_SHORTCUTS: Partial<Record<string, ScreenshotEditorTool>> = {
	a: 'arrow',
	c: 'censor',
	h: 'highlight',
	o: 'ellipse',
	p: 'pen',
	r: 'rectangle',
	t: 'text',
	v: 'select',
}

type PropertyValueKind = 'size' | 'width'

export function useScreenshotEditor() {
	const canvas = shallowRef<FabricCanvas>()
	const loading = ref(false)
	const tool = ref<ScreenshotEditorTool>('select')
	const color = ref('#ffffff')
	const strokeWidth = ref(6)
	const fontSize = ref(30)
	const censorMode = ref<ScreenshotCensorMode>('blur')
	const zoom = ref(1)
	const fitScale = ref(1)
	const isFit = ref(true)
	const originalWidth = ref(0)
	const originalHeight = ref(0)
	const history = ref<EditorHistoryEntry[]>([])
	const historyIndex = ref(-1)
	const selectedPropertyKind = ref<ScreenshotEditorPropertyKind>()
	const selectionCount = ref(0)
	const canUndo = computed(() => historyIndex.value > 0)
	const canRedo = computed(() => historyIndex.value < history.value.length - 1)
	const canDelete = computed(() => selectionCount.value > 0)
	const propertyValueKind = computed<PropertyValueKind | undefined>(() => {
		const kind = selectedPropertyKind.value ?? (selectionCount.value === 0 ? tool.value : undefined)
		if (kind === 'text') return 'size'
		if (
			kind === 'pen' ||
			kind === 'highlight' ||
			kind === 'arrow' ||
			kind === 'rectangle' ||
			kind === 'ellipse'
		) {
			return 'width'
		}
		return undefined
	})
	const hasColorProperty = computed(() => {
		const kind = selectedPropertyKind.value ?? (selectionCount.value === 0 ? tool.value : undefined)
		return (
			kind === 'pen' ||
			kind === 'highlight' ||
			kind === 'arrow' ||
			kind === 'rectangle' ||
			kind === 'ellipse' ||
			kind === 'text' ||
			(kind === 'censor' && selectionCount.value === 0 && censorMode.value === 'solid')
		)
	})
	const showCensorMode = computed(() => selectionCount.value === 0 && tool.value === 'censor')
	const canZoomOut = computed(() => zoom.value > MIN_ZOOM)
	const canZoomIn = computed(() => zoom.value < MAX_ZOOM)

	const defaultWidths: Record<Exclude<PropertyValueKind, 'size'> | 'highlight', number> = {
		width: 6,
		highlight: 24,
	}
	let defaultColor = '#ffffff'
	let defaultFontSize = 30

	let fabric: FabricModule | undefined
	let background: FabricImage | undefined
	let sourceImage: HTMLImageElement | undefined
	let sourceUrl: string | undefined
	let drawingStart: { x: number; y: number } | undefined
	let drawingObject: EditorFabricObject | undefined
	let restoringHistory = false
	let constructingObject = false
	let propertyEditStart: string | undefined

	async function initialize(element: HTMLCanvasElement, screenshot: InstanceScreenshot) {
		dispose()
		loading.value = true
		try {
			fabric = await import('fabric')
			const bytes = await readFile(screenshot.path)
			sourceUrl = URL.createObjectURL(new Blob([bytes], { type: 'image/png' }))
			sourceImage = await loadImage(sourceUrl)
			originalWidth.value = sourceImage.naturalWidth
			originalHeight.value = sourceImage.naturalHeight

			const nextCanvas = new fabric.Canvas(element, {
				width: originalWidth.value,
				height: originalHeight.value,
				enableRetinaScaling: false,
				backgroundColor: '#111827',
				preserveObjectStacking: true,
				selection: true,
			})
			canvas.value = nextCanvas
			background = new fabric.FabricImage(sourceImage, {
				left: 0,
				top: 0,
				originX: 'left',
				originY: 'top',
				width: originalWidth.value,
				height: originalHeight.value,
				selectable: false,
				evented: false,
				excludeFromExport: false,
			})
			setEditorMetadata(background, 'background')
			nextCanvas.add(background)
			nextCanvas.sendObjectToBack(background)
			bindCanvasEvents(nextCanvas)
			resetHistory()
			setTool('select')
		} finally {
			loading.value = false
		}
	}

	function bindCanvasEvents(editorCanvas: FabricCanvas) {
		editorCanvas.on('mouse:down', handleMouseDown)
		editorCanvas.on('mouse:move', handleMouseMove)
		editorCanvas.on('mouse:up', handleMouseUp)
		editorCanvas.on('object:added', () => {
			if (!constructingObject && !restoringHistory && !editorCanvas.isDrawingMode) {
				recordHistory()
			}
		})
		editorCanvas.on('path:created', ({ path }) => {
			setEditorMetadata(path, tool.value === 'highlight' ? 'highlight' : 'pen')
			styleObjectControls(path)
			recordHistory()
			syncSelectionProperties()
		})
		editorCanvas.on('object:modified', () => {
			recordHistory()
			syncSelectionProperties()
		})
		editorCanvas.on('object:removed', () => {
			if (!constructingObject && !restoringHistory) recordHistory()
		})
		editorCanvas.on('selection:created', syncSelectionProperties)
		editorCanvas.on('selection:updated', syncSelectionProperties)
		editorCanvas.on('selection:cleared', syncSelectionProperties)
		editorCanvas.on('text:editing:exited', () => {
			recordHistory()
			syncSelectionProperties()
		})
	}

	function setTool(nextTool: ScreenshotEditorTool) {
		tool.value = nextTool
		const editorCanvas = canvas.value
		if (!editorCanvas || !fabric) return
		const activeObject = editorCanvas.getActiveObject() as
			| (EditorFabricObject & { isEditing?: boolean; exitEditing?: () => void })
			| undefined
		if (activeObject?.isEditing && nextTool !== 'text') activeObject.exitEditing?.()

		editorCanvas.isDrawingMode = nextTool === 'pen' || nextTool === 'highlight'
		editorCanvas.selection = nextTool === 'select'
		for (const object of annotationObjects()) {
			object.selectable = nextTool === 'select'
			object.evented = nextTool === 'select'
			styleObjectControls(object)
		}
		if (nextTool !== 'select') editorCanvas.discardActiveObject()
		syncSelectionProperties()
		configureCursor(editorCanvas, nextTool)
		configureBrush(editorCanvas)
		editorCanvas.requestRenderAll()
	}

	function refreshToolSettings() {
		const editorCanvas = canvas.value
		if (!editorCanvas) return
		configureBrush(editorCanvas)
		editorCanvas.requestRenderAll()
	}

	function configureBrush(editorCanvas: FabricCanvas) {
		if (!fabric || !editorCanvas.isDrawingMode) return
		const brush = new fabric.PencilBrush(editorCanvas)
		brush.color = tool.value === 'highlight' ? hexToRgba(color.value, 0.35) : color.value
		brush.width = strokeWidth.value
		editorCanvas.freeDrawingBrush = brush
	}

	function configureCursor(editorCanvas: FabricCanvas, nextTool: ScreenshotEditorTool) {
		const cursor = nextTool === 'select' ? 'default' : nextTool === 'text' ? 'text' : 'crosshair'
		editorCanvas.defaultCursor = cursor
		editorCanvas.freeDrawingCursor = cursor
		editorCanvas.hoverCursor = nextTool === 'select' ? 'move' : cursor
		editorCanvas.moveCursor = nextTool === 'select' ? 'move' : cursor
	}

	function syncSelectionProperties() {
		commitPropertyEdit()
		const activeObjects = (canvas.value?.getActiveObjects() ?? []).filter(
			(object) => object !== background,
		) as EditorFabricObject[]
		selectionCount.value = activeObjects.length
		const selected = activeObjects.length === 1 ? activeObjects[0] : undefined
		selectedPropertyKind.value = isPropertyKind(selected?.editorKind)
			? selected.editorKind
			: undefined

		if (selected && selectedPropertyKind.value) {
			const selectedColor = getObjectColor(selected, selectedPropertyKind.value)
			if (selectedColor) color.value = selectedColor
			if (selectedPropertyKind.value === 'text') {
				fontSize.value = Math.round(selected.fontSize ?? defaultFontSize)
			} else {
				const selectedWidth = getObjectStrokeWidth(selected, selectedPropertyKind.value)
				if (selectedWidth) strokeWidth.value = Math.round(selectedWidth)
			}
			return
		}

		if (activeObjects.length > 0) return
		color.value = defaultColor
		if (tool.value === 'text') {
			fontSize.value = defaultFontSize
		} else if (tool.value === 'highlight') {
			strokeWidth.value = defaultWidths.highlight
		} else {
			strokeWidth.value = defaultWidths.width
		}
	}

	function updateColor(nextColor: string) {
		if (selectionCount.value === 1) beginPropertyEdit()
		color.value = nextColor
		const selected = singleSelectedObject()
		const kind = selected && isPropertyKind(selected.editorKind) ? selected.editorKind : undefined
		if (selected && kind && kind !== 'censor') {
			setObjectColor(selected, kind, nextColor)
		} else if (selectionCount.value === 0) {
			defaultColor = nextColor
			refreshToolSettings()
		}
	}

	function updateStrokeWidth(nextWidth: number) {
		if (selectionCount.value === 1) beginPropertyEdit()
		strokeWidth.value = nextWidth
		const selected = singleSelectedObject()
		const kind = selected && isPropertyKind(selected.editorKind) ? selected.editorKind : undefined
		if (selected && kind && kind !== 'censor' && kind !== 'text') {
			setObjectStrokeWidth(selected, kind, nextWidth)
		} else if (selectionCount.value === 0) {
			if (tool.value === 'highlight') defaultWidths.highlight = nextWidth
			else defaultWidths.width = nextWidth
			refreshToolSettings()
		}
	}

	function updateFontSize(nextSize: number) {
		if (selectionCount.value === 1) beginPropertyEdit()
		fontSize.value = nextSize
		const selected = singleSelectedObject()
		if (selected?.editorKind === 'text') {
			selected.set({ fontSize: nextSize })
			selected.setCoords()
			selected.dirty = true
			canvas.value?.requestRenderAll()
		} else if (selectionCount.value === 0) {
			defaultFontSize = nextSize
		}
	}

	function singleSelectedObject() {
		const activeObjects = (canvas.value?.getActiveObjects() ?? []).filter(
			(object) => object !== background,
		) as EditorFabricObject[]
		return activeObjects.length === 1 ? activeObjects[0] : undefined
	}

	function getObjectColor(object: EditorFabricObject, kind: ScreenshotEditorPropertyKind) {
		if (kind === 'arrow') {
			const [line] = (object as FabricGroup).getObjects()
			return colorToHex(line?.stroke)
		}
		return colorToHex(kind === 'text' ? object.fill : object.stroke)
	}

	function getObjectStrokeWidth(object: EditorFabricObject, kind: ScreenshotEditorPropertyKind) {
		if (kind === 'arrow') {
			const [line] = (object as FabricGroup).getObjects()
			return line?.strokeWidth
		}
		return object.strokeWidth
	}

	function setObjectColor(
		object: EditorFabricObject,
		kind: ScreenshotEditorPropertyKind,
		nextColor: string,
	) {
		if (kind === 'arrow') {
			const [line, head] = (object as FabricGroup).getObjects()
			line?.set({ stroke: nextColor })
			head?.set({ fill: nextColor })
		} else if (kind === 'text') {
			object.set({ fill: nextColor })
		} else {
			object.set({ stroke: kind === 'highlight' ? hexToRgba(nextColor, 0.35) : nextColor })
		}
		object.dirty = true
		object.setCoords()
		canvas.value?.requestRenderAll()
	}

	function setObjectStrokeWidth(
		object: EditorFabricObject,
		kind: ScreenshotEditorPropertyKind,
		nextWidth: number,
	) {
		if (kind === 'arrow') {
			const group = object as FabricGroup
			const [line, head] = group.getObjects()
			const linePosition = line?.getXY()
			line?.set({ strokeWidth: nextWidth })
			head?.set({ width: nextWidth * 4, height: nextWidth * 5 })
			group.triggerLayout({ deep: true })
			if (line && linePosition) {
				const nextLinePosition = line.getXY()
				group.set({
					left: group.left + linePosition.x - nextLinePosition.x,
					top: group.top + linePosition.y - nextLinePosition.y,
				})
			}
		} else {
			object.set({ strokeWidth: nextWidth })
		}
		object.dirty = true
		object.setCoords()
		canvas.value?.requestRenderAll()
	}

	function handleMouseDown(event: FabricPointerEvent) {
		const editorCanvas = canvas.value
		if (!editorCanvas || !fabric || editorCanvas.isDrawingMode || tool.value === 'select') return
		const point = editorCanvas.getScenePoint(event.e)

		if (tool.value === 'text') {
			constructingObject = true
			const text = new fabric.IText('Text', {
				left: point.x,
				top: point.y,
				originX: 'left',
				originY: 'top',
				fill: color.value,
				fontFamily: 'Inter, sans-serif',
				fontSize: fontSize.value,
			})
			setEditorMetadata(text, 'text')
			styleObjectControls(text)
			editorCanvas.add(text)
			editorCanvas.setActiveObject(text)
			constructingObject = false
			text.enterEditing()
			text.selectAll()
			editorCanvas.requestRenderAll()
			return
		}

		drawingStart = point
		constructingObject = true
		if (tool.value === 'rectangle' || tool.value === 'censor') {
			drawingObject = new fabric.Rect({
				left: point.x,
				top: point.y,
				originX: 'left',
				originY: 'top',
				width: 1,
				height: 1,
				fill: 'transparent',
				stroke: tool.value === 'censor' ? '#ffffff' : color.value,
				strokeDashArray: tool.value === 'censor' ? [10, 8] : undefined,
				strokeWidth: tool.value === 'censor' ? 2 : strokeWidth.value,
				selectable: false,
				evented: false,
			})
		} else if (tool.value === 'ellipse') {
			drawingObject = new fabric.Ellipse({
				left: point.x,
				top: point.y,
				originX: 'left',
				originY: 'top',
				rx: 1,
				ry: 1,
				fill: 'transparent',
				stroke: color.value,
				strokeWidth: strokeWidth.value,
				selectable: false,
				evented: false,
			})
		} else if (tool.value === 'arrow') {
			drawingObject = new fabric.Line([point.x, point.y, point.x, point.y], {
				stroke: color.value,
				strokeWidth: strokeWidth.value,
				selectable: false,
				evented: false,
			})
		}
		if (drawingObject) {
			styleObjectControls(drawingObject)
			editorCanvas.add(drawingObject)
		}
	}

	function handleMouseMove(event: FabricPointerEvent) {
		const editorCanvas = canvas.value
		if (!editorCanvas || !drawingStart || !drawingObject) return
		const point = editorCanvas.getScenePoint(event.e)
		const rect = normalizedRect(drawingStart, point)

		if (tool.value === 'ellipse') {
			drawingObject.set({
				left: rect.left,
				top: rect.top,
				rx: rect.width / 2,
				ry: rect.height / 2,
			})
		} else if (tool.value === 'arrow') {
			drawingObject.set({ x2: point.x, y2: point.y })
		} else {
			drawingObject.set(rect)
		}
		drawingObject.setCoords()
		editorCanvas.requestRenderAll()
	}

	function handleMouseUp(event: FabricPointerEvent) {
		const editorCanvas = canvas.value
		if (!editorCanvas || !fabric || !drawingStart || !drawingObject) return
		const start = drawingStart
		const point = editorCanvas.getScenePoint(event.e)
		const rect = normalizedRect(start, point)
		const preview = drawingObject
		drawingStart = undefined
		drawingObject = undefined

		const isTooSmall =
			tool.value === 'arrow'
				? Math.hypot(point.x - start.x, point.y - start.y) < 2
				: rect.width < 2 || rect.height < 2
		if (isTooSmall) {
			editorCanvas.remove(preview)
			constructingObject = false
			editorCanvas.requestRenderAll()
			return
		}

		if (tool.value === 'censor') {
			editorCanvas.remove(preview)
			const censorCanvas = renderCensorRegion(sourceImage!, rect, censorMode.value, color.value)
			const censor = new fabric.FabricImage(censorCanvas, {
				left: rect.left,
				top: rect.top,
				originX: 'left',
				originY: 'top',
				selectable: false,
				evented: false,
			})
			setEditorMetadata(censor, 'censor', rect)
			styleObjectControls(censor)
			const censorCount = annotationObjects().filter(
				(object) => object.editorKind === 'censor',
			).length
			editorCanvas.insertAt(1 + censorCount, censor)
		} else if (tool.value === 'arrow') {
			editorCanvas.remove(preview)
			const line = new fabric.Line([start.x, start.y, point.x, point.y], {
				stroke: color.value,
				strokeWidth: strokeWidth.value,
			})
			const angle = (Math.atan2(point.y - start.y, point.x - start.x) * 180) / Math.PI + 90
			const head = new fabric.Triangle({
				left: point.x,
				top: point.y,
				originX: 'center',
				originY: 'center',
				width: strokeWidth.value * 4,
				height: strokeWidth.value * 5,
				fill: color.value,
				angle,
			})
			const arrow = new fabric.Group([line, head])
			setEditorMetadata(arrow, 'arrow')
			styleObjectControls(arrow)
			editorCanvas.add(arrow)
		} else {
			setEditorMetadata(preview, tool.value === 'ellipse' ? 'ellipse' : 'rectangle')
			styleObjectControls(preview)
		}

		constructingObject = false
		setTool(tool.value)
		recordHistory()
		editorCanvas.requestRenderAll()
	}

	function deleteSelection() {
		const editorCanvas = canvas.value
		if (!editorCanvas) return false
		const active = editorCanvas.getActiveObjects().filter((object) => object !== background)
		if (active.length === 0) return false
		editorCanvas.discardActiveObject()
		constructingObject = true
		for (const object of active) editorCanvas.remove(object)
		constructingObject = false
		recordHistory()
		syncSelectionProperties()
		editorCanvas.requestRenderAll()
		return true
	}

	function recordHistory() {
		if (restoringHistory || constructingObject || !canvas.value) return
		const entry = snapshot()
		const nextHistory = history.value.slice(0, historyIndex.value + 1)
		const currentEntry = nextHistory.at(-1)
		if (currentEntry && serializeSnapshot(currentEntry) === serializeSnapshot(entry)) return
		nextHistory.push(entry)
		history.value = nextHistory
		historyIndex.value = nextHistory.length - 1
	}

	function beginPropertyEdit() {
		propertyEditStart ??= serializeSnapshot(snapshot())
	}

	function commitPropertyEdit() {
		if (!propertyEditStart) return
		const hasChanged = propertyEditStart !== serializeSnapshot(snapshot())
		propertyEditStart = undefined
		if (hasChanged) recordHistory()
	}

	function resetHistory() {
		history.value = [snapshot()]
		historyIndex.value = 0
	}

	function snapshot(): EditorHistoryEntry {
		return {
			objects: annotationObjects().map((object) =>
				object.toObject(['editorKind', 'sourceRect']),
			) as ScreenshotEditorObjectState[],
		}
	}

	async function restoreHistory(index: number) {
		const editorCanvas = canvas.value
		const entry = history.value[index]
		if (!editorCanvas || !entry || !fabric) return
		restoringHistory = true
		try {
			editorCanvas.discardActiveObject()
			editorCanvas.remove(...annotationObjects())
			const enlivenObjects = fabric.util.enlivenObjects as unknown as (
				objects: ScreenshotEditorObjectState[],
			) => Promise<EditorFabricObject[]>
			const restored = await enlivenObjects(entry.objects)
			for (const object of restored) {
				styleObjectControls(object)
				editorCanvas.add(object)
			}
			historyIndex.value = index
			setTool(tool.value)
			syncSelectionProperties()
			editorCanvas.requestRenderAll()
		} finally {
			restoringHistory = false
		}
	}

	async function undo() {
		if (canUndo.value) await restoreHistory(historyIndex.value - 1)
	}

	async function redo() {
		if (canRedo.value) await restoreHistory(historyIndex.value + 1)
	}

	function fitToViewport(width: number, height: number) {
		if (!canvas.value || !originalWidth.value || !originalHeight.value) return
		fitScale.value = Math.min(
			Math.max(1, width) / originalWidth.value,
			Math.max(1, height) / originalHeight.value,
		)
		if (isFit.value) {
			zoom.value = fitScale.value
			applyDisplayScale()
		}
	}

	function setZoom(nextZoom: number) {
		isFit.value = false
		zoom.value = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, nextZoom))
		applyDisplayScale()
	}

	function setFit() {
		isFit.value = true
		zoom.value = fitScale.value
		applyDisplayScale()
	}

	function applyDisplayScale() {
		const editorCanvas = canvas.value
		if (!editorCanvas) return
		const scale = zoom.value
		editorCanvas.setDimensions(
			{
				width: Math.round(originalWidth.value * scale),
				height: Math.round(originalHeight.value * scale),
			},
			{ cssOnly: true },
		)
		for (const object of annotationObjects()) styleObjectControls(object)
		editorCanvas.calcOffset()
		editorCanvas.requestRenderAll()
	}

	function styleObjectControls(object: FabricObject) {
		const displayScale = Math.max(zoom.value, 0.01)
		object.set({
			borderColor: SELECTION_COLOR,
			borderScaleFactor: 2 / displayScale,
			cornerColor: SELECTION_COLOR,
			cornerSize: CONTROL_SIZE / displayScale,
			cornerStrokeColor: '#ffffff',
			cornerStyle: 'circle',
			padding: 3 / displayScale,
			touchCornerSize: CONTROL_TOUCH_SIZE / displayScale,
			transparentCorners: false,
		})
		object.setControlsVisibility({
			mb: true,
			ml: true,
			mr: true,
			mt: true,
			mtr: true,
		})
		if (fabric) {
			for (const key of ['ml', 'mr'] as const) {
				object.controls[key].actionHandler = fabric.controlsUtils.scalingX
				object.controls[key].cursorStyleHandler = fabric.controlsUtils.scaleCursorStyleHandler
				object.controls[key].getActionName = () => 'scaleX'
			}
			for (const key of ['mt', 'mb'] as const) {
				object.controls[key].actionHandler = fabric.controlsUtils.scalingY
				object.controls[key].cursorStyleHandler = fabric.controlsUtils.scaleCursorStyleHandler
				object.controls[key].getActionName = () => 'scaleY'
			}
		}
		if (object.controls.mtr) {
			object.controls.mtr.offsetY = -28 / displayScale
			object.controls.mtr.sizeX = 10 / displayScale
			object.controls.mtr.sizeY = 10 / displayScale
		}
		object.setCoords()
	}

	async function exportPng() {
		const editorCanvas = canvas.value
		if (!editorCanvas) throw new Error('Screenshot editor is not ready')
		editorCanvas.discardActiveObject()
		editorCanvas.requestRenderAll()
		const blob = await editorCanvas.toBlob({ format: 'png', multiplier: 1 })
		if (!blob) throw new Error('Could not export edited screenshot')
		return new Uint8Array(await blob.arrayBuffer())
	}

	function handleKeyboardShortcut(event: KeyboardEvent) {
		const activeObject = canvas.value?.getActiveObject() as
			| (EditorFabricObject & { isEditing?: boolean; exitEditing?: () => void })
			| undefined
		if (activeObject?.isEditing && event.key === 'Escape') {
			event.preventDefault()
			activeObject.exitEditing?.()
			canvas.value?.discardActiveObject()
			canvas.value?.requestRenderAll()
			return true
		}
		if (activeObject?.isEditing) return false
		if (isTypingTarget(event.target)) return false

		const modifier = event.metaKey || event.ctrlKey
		const key = event.key.toLowerCase()
		if (modifier && key === 'z') {
			event.preventDefault()
			void (event.shiftKey ? redo() : undo())
			return true
		}
		if (event.ctrlKey && key === 'y') {
			event.preventDefault()
			void redo()
			return true
		}
		if (event.key === 'Delete' || event.key === 'Backspace') {
			if (!deleteSelection()) return false
			event.preventDefault()
			return true
		}
		if (event.key === 'Escape') {
			event.preventDefault()
			if (drawingObject) {
				canvas.value?.remove(drawingObject)
				drawingObject = undefined
				drawingStart = undefined
				constructingObject = false
				canvas.value?.requestRenderAll()
				return true
			}
			if (tool.value !== 'select') {
				setTool('select')
				return true
			}
			if (canvas.value?.getActiveObjects().length) {
				canvas.value.discardActiveObject()
				canvas.value.requestRenderAll()
				return true
			}
			return false
		}
		if (!modifier && !event.altKey) {
			const shortcutTool = TOOL_SHORTCUTS[key]
			if (shortcutTool) {
				event.preventDefault()
				setTool(shortcutTool)
				return true
			}
		}
		return false
	}

	function isTextEditing() {
		return Boolean(
			(canvas.value?.getActiveObject() as { isEditing?: boolean } | undefined)?.isEditing,
		)
	}

	function annotationObjects() {
		return (canvas.value?.getObjects() ?? []).filter(
			(object) => object !== background,
		) as EditorFabricObject[]
	}

	function dispose() {
		void canvas.value?.dispose()
		canvas.value = undefined
		background = undefined
		sourceImage = undefined
		if (sourceUrl) URL.revokeObjectURL(sourceUrl)
		sourceUrl = undefined
		history.value = []
		historyIndex.value = -1
		zoom.value = 1
		fitScale.value = 1
		isFit.value = true
		originalWidth.value = 0
		originalHeight.value = 0
		selectedPropertyKind.value = undefined
		selectionCount.value = 0
		propertyEditStart = undefined
	}

	return {
		loading,
		tool,
		color,
		strokeWidth,
		fontSize,
		censorMode,
		zoom,
		isFit,
		canUndo,
		canRedo,
		canDelete,
		canZoomOut,
		canZoomIn,
		hasColorProperty,
		propertyValueKind,
		showCensorMode,
		initialize,
		dispose,
		setTool,
		updateColor,
		updateStrokeWidth,
		updateFontSize,
		beginPropertyEdit,
		commitPropertyEdit,
		deleteSelection,
		undo,
		redo,
		fitToViewport,
		setZoom,
		setFit,
		exportPng,
		handleKeyboardShortcut,
		isTextEditing,
		resetHistory,
	}
}

function setEditorMetadata(
	object: FabricObject,
	kind: ScreenshotEditorObjectKind,
	sourceRect?: ScreenshotEditorSourceRect,
) {
	const editorObject = object as EditorFabricObject
	editorObject.editorKind = kind
	editorObject.sourceRect = sourceRect
}

function normalizedRect(start: { x: number; y: number }, end: { x: number; y: number }) {
	return {
		left: Math.min(start.x, end.x),
		top: Math.min(start.y, end.y),
		width: Math.abs(end.x - start.x),
		height: Math.abs(end.y - start.y),
	}
}

function loadImage(url: string) {
	return new Promise<HTMLImageElement>((resolve, reject) => {
		const image = new Image()
		image.onload = () => resolve(image)
		image.onerror = () => reject(new Error('Could not load screenshot image'))
		image.src = url
	})
}

function hexToRgba(hex: string, alpha: number) {
	const normalized = hex.replace('#', '')
	const red = Number.parseInt(normalized.slice(0, 2), 16)
	const green = Number.parseInt(normalized.slice(2, 4), 16)
	const blue = Number.parseInt(normalized.slice(4, 6), 16)
	return `rgba(${red}, ${green}, ${blue}, ${alpha})`
}

function colorToHex(color: unknown) {
	if (typeof color !== 'string') return undefined
	if (/^#[\da-f]{6}$/i.test(color)) return color
	if (/^#[\da-f]{3}$/i.test(color)) {
		return `#${color
			.slice(1)
			.split('')
			.map((character) => character.repeat(2))
			.join('')}`
	}
	const components = color
		.match(/[\d.]+/g)
		?.slice(0, 3)
		.map(Number)
	if (!components || components.length !== 3) return undefined
	return `#${components
		.map((component) => Math.round(component).toString(16).padStart(2, '0'))
		.join('')}`
}

function isPropertyKind(
	kind: ScreenshotEditorObjectKind | undefined,
): kind is ScreenshotEditorPropertyKind {
	return (
		kind === 'arrow' ||
		kind === 'censor' ||
		kind === 'ellipse' ||
		kind === 'highlight' ||
		kind === 'pen' ||
		kind === 'rectangle' ||
		kind === 'text'
	)
}

function isTypingTarget(target: EventTarget | null) {
	return (
		target instanceof HTMLInputElement ||
		target instanceof HTMLTextAreaElement ||
		target instanceof HTMLSelectElement ||
		(target instanceof HTMLElement && target.isContentEditable)
	)
}

function serializeSnapshot(entry: EditorHistoryEntry) {
	return JSON.stringify(entry)
}
