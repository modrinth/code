import { useDebugLogger } from '@modrinth/ui'
import { readFile } from '@tauri-apps/plugin-fs'
import { computed, ref, shallowRef, watch } from 'vue'

import type { InstanceScreenshot, ScreenshotEditorData } from '@/helpers/instance'

import { renderCensorRegion } from './censor-object'
import type {
	EditorHistoryEntry,
	ScreenshotCensorMode,
	ScreenshotEditorDocument,
	ScreenshotEditorObjectKind,
	ScreenshotEditorObjectState,
	ScreenshotEditorPropertyKind,
	ScreenshotEditorSourceRect,
	ScreenshotEditorTool,
	ScreenshotEraserMode,
} from './editor-types'

type FabricModule = typeof import('fabric')
type FabricCanvas = import('fabric').Canvas
type FabricObject = import('fabric').FabricObject
type FabricPoint = import('fabric').Point
type FabricPath = import('fabric').Path
type FabricPathData = Exclude<ConstructorParameters<FabricModule['Path']>[0], string>
type FabricImage = import('fabric').FabricImage
type FabricGroup = import('fabric').Group
type FabricImageOptions = ConstructorParameters<FabricModule['FabricImage']>[1]
type EditorFabricObject = FabricObject & {
	editorKind?: ScreenshotEditorObjectKind
	fontSize?: number
	sourceRect?: ScreenshotEditorSourceRect
	censorMode?: ScreenshotCensorMode
	censorColor?: string
}

type FabricPointerEvent = {
	e: MouseEvent | TouchEvent | PointerEvent
}

const MIN_ZOOM = 0.25
const MAX_ZOOM = 4
const CONTROL_SIZE = 14
const CONTROL_TOUCH_SIZE = 24
const SELECTION_COLOR = '#1bd96a'
const CENSOR_REGENERATED_PROPERTIES = new Set([
	'type',
	'version',
	'src',
	'filters',
	'resizeFilter',
	'clipPath',
	'editorKind',
	'sourceRect',
	'censorMode',
	'censorColor',
])
const TOOL_SHORTCUTS: Partial<Record<string, ScreenshotEditorTool>> = {
	a: 'arrow',
	c: 'censor',
	e: 'eraser',
	h: 'highlight',
	o: 'ellipse',
	p: 'pen',
	r: 'rectangle',
	t: 'text',
	v: 'select',
}

type PropertyValueKind = 'size' | 'width'

export function useScreenshotEditor() {
	const debugEraser = useDebugLogger('ScreenshotEditor:Eraser')
	const canvas = shallowRef<FabricCanvas>()
	const loading = ref(false)
	const tool = ref<ScreenshotEditorTool>('select')
	const color = ref('#ffffff')
	const strokeWidth = ref(6)
	const fontSize = ref(30)
	const censorMode = ref<ScreenshotCensorMode>('blur')
	const eraserMode = ref<ScreenshotEraserMode>('element')
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
		if (kind === 'eraser' && eraserMode.value === 'area') return 'width'
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
	const showEraserMode = computed(() => selectionCount.value === 0 && tool.value === 'eraser')
	const canZoomOut = computed(() => zoom.value > MIN_ZOOM)
	const canZoomIn = computed(() => zoom.value < MAX_ZOOM)

	const defaultWidths: Record<'width' | 'highlight' | 'eraser', number> = {
		width: 6,
		highlight: 24,
		eraser: 24,
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
	let persistEditorState = false
	let erasing = false
	let erasedDuringGesture = false

	watch(eraserMode, () => {
		finishErasing()
		if (tool.value === 'eraser') setTool('eraser')
	})

	async function initialize(
		element: HTMLCanvasElement,
		screenshot: InstanceScreenshot,
		editorData: ScreenshotEditorData,
	) {
		dispose()
		loading.value = true
		try {
			fabric = await import('fabric')
			const document = parseEditorDocument(editorData.editor_state)
			persistEditorState = Boolean(document) || !screenshot.original_screenshot_id
			const bytes = await readFile(document ? editorData.background_path : screenshot.path)
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
			if (document) {
				const restored = await enlivenEditorObjects(document.objects)
				for (const object of restored) {
					styleObjectControls(object)
					nextCanvas.add(object)
				}
			}
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
			if (tool.value === 'eraser' && eraserMode.value === 'area') {
				void eraseAreaWithPath(path).catch((error) => {
					debugEraser('area:error', error)
				})
				return
			}
			setEditorMetadata(path, tool.value === 'highlight' ? 'highlight' : 'pen')
			styleObjectControls(path)
			recordHistory()
			syncSelectionProperties()
		})
		editorCanvas.on('object:modified', ({ target }) => {
			if (target) {
				refreshModifiedCensors(target as EditorFabricObject)
				editorCanvas.requestRenderAll()
			}
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
		if (nextTool !== 'eraser') finishErasing()
		tool.value = nextTool
		const editorCanvas = canvas.value
		if (!editorCanvas || !fabric) return
		const activeObject = editorCanvas.getActiveObject() as
			| (EditorFabricObject & { isEditing?: boolean; exitEditing?: () => void })
			| undefined
		if (activeObject?.isEditing && nextTool !== 'text') activeObject.exitEditing?.()

		editorCanvas.isDrawingMode =
			nextTool === 'pen' ||
			nextTool === 'highlight' ||
			(nextTool === 'eraser' && eraserMode.value === 'area')
		editorCanvas.selection = nextTool === 'select'
		for (const object of annotationObjects()) {
			object.selectable = nextTool === 'select'
			object.evented =
				nextTool === 'select' || (nextTool === 'eraser' && eraserMode.value === 'element')
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
		const brush =
			tool.value === 'eraser'
				? createAreaEraserBrush(editorCanvas, fabric)
				: new fabric.PencilBrush(editorCanvas)
		brush.color =
			tool.value === 'eraser'
				? 'rgba(255, 255, 255, 0.45)'
				: tool.value === 'highlight'
					? hexToRgba(color.value, 0.35)
					: color.value
		brush.width = strokeWidth.value
		editorCanvas.freeDrawingBrush = brush
	}

	function createAreaEraserBrush(editorCanvas: FabricCanvas, fabricModule: FabricModule) {
		const editorSourceImage = sourceImage
		return new (class extends fabricModule.PencilBrush {
			override _setBrushStyles(context: CanvasRenderingContext2D) {
				super._setBrushStyles(context)
				const backgroundPattern = editorSourceImage
					? context.createPattern(editorSourceImage, 'no-repeat')
					: null
				if (backgroundPattern) context.strokeStyle = backgroundPattern
			}
		})(editorCanvas)
	}

	function configureCursor(editorCanvas: FabricCanvas, nextTool: ScreenshotEditorTool) {
		const hasBrushCursor =
			nextTool === 'pen' ||
			nextTool === 'highlight' ||
			(nextTool === 'eraser' && eraserMode.value === 'area')
		const cursor = hasBrushCursor
			? 'none'
			: nextTool === 'select'
				? 'default'
				: nextTool === 'text'
					? 'text'
					: 'crosshair'
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
		} else if (tool.value === 'eraser') {
			strokeWidth.value = defaultWidths.eraser
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
			else if (tool.value === 'eraser') defaultWidths.eraser = nextWidth
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

	function beginErasing(point: FabricPoint) {
		erasing = true
		erasedDuringGesture = false
		constructingObject = true
		eraseAtPoint(point)
	}

	function eraseAtPoint(point: FabricPoint) {
		const editorCanvas = canvas.value
		if (!editorCanvas) return
		const { target } = editorCanvas.searchPossibleTargets(annotationObjects(), point)
		if (!target || target === background) return
		editorCanvas.remove(target)
		erasedDuringGesture = true
		editorCanvas.requestRenderAll()
	}

	function finishErasing() {
		if (!erasing) return
		erasing = false
		constructingObject = false
		if (erasedDuringGesture) recordHistory()
		erasedDuringGesture = false
		syncSelectionProperties()
		canvas.value?.requestRenderAll()
	}

	async function eraseAreaWithPath(eraserPath: FabricPath) {
		const editorCanvas = canvas.value
		const fabricModule = fabric
		if (!editorCanvas || !fabricModule) return

		constructingObject = true
		const eraserBounds = eraserPath.getBoundingRect()
		const eraserMask = createAreaEraserMask(eraserPath, fabricModule)
		editorCanvas.remove(eraserPath)
		const targets = annotationObjects().filter((object) =>
			boundingRectsOverlap(eraserBounds, object.getBoundingRect()),
		)

		try {
			await Promise.all(
				targets.map(async (object) => {
					const clipPath = await eraserMask.clone()
					fabricModule.util.sendObjectToPlane(clipPath, undefined, object.calcTransformMatrix())
					const nextClipPath = object.clipPath
						? fabricModule.util.mergeClipPaths(object.clipPath, clipPath)
						: clipPath
					object.set('clipPath', nextClipPath)
				}),
			)
		} finally {
			constructingObject = false
		}

		if (targets.length > 0) recordHistory()
		editorCanvas.requestRenderAll()
	}

	function createAreaEraserMask(eraserPath: FabricPath, fabricModule: FabricModule) {
		const points = samplePathPoints(eraserPath, fabricModule)
		const width = Math.max(1, eraserPath.strokeWidth)
		const radius = width / 2
		const maskPath: FabricPathData = []

		if (points.length === 1) {
			const point = points[0]
			maskPath.push(
				['M', point.x + radius, point.y],
				['A', radius, radius, 0, 1, 0, point.x - radius, point.y],
				['A', radius, radius, 0, 1, 0, point.x + radius, point.y],
				['Z'],
			)
		} else {
			for (let index = 1; index < points.length; index++) {
				const start = points[index - 1]
				const end = points[index]
				const deltaX = end.x - start.x
				const deltaY = end.y - start.y
				const length = Math.hypot(deltaX, deltaY)
				if (length === 0) continue
				const normalX = (-deltaY / length) * radius
				const normalY = (deltaX / length) * radius
				maskPath.push(
					['M', start.x + normalX, start.y + normalY],
					['L', end.x + normalX, end.y + normalY],
					['A', radius, radius, 0, 0, 1, end.x - normalX, end.y - normalY],
					['L', start.x - normalX, start.y - normalY],
					['A', radius, radius, 0, 0, 1, start.x + normalX, start.y + normalY],
					['Z'],
				)
			}
		}

		const brushMask = new fabricModule.Path(maskPath, {
			absolutePositioned: false,
			fill: '#000000',
			inverted: true,
			opacity: 1,
			strokeWidth: 0,
		})
		const outerMask = new fabricModule.Rect({
			left: 0,
			top: 0,
			originX: 'left',
			originY: 'top',
			width: originalWidth.value,
			height: originalHeight.value,
			fill: '#000000',
			strokeWidth: 0,
		})
		fabricModule.util.sendObjectToPlane(brushMask, undefined, outerMask.calcTransformMatrix())
		outerMask.set('clipPath', brushMask)
		return outerMask
	}

	function samplePathPoints(eraserPath: FabricPath, fabricModule: FabricModule) {
		const sampled: FabricPoint[] = []
		const transform = eraserPath.calcTransformMatrix()
		const pathOffset = eraserPath.pathOffset
		const spacing = Math.max(2, eraserPath.strokeWidth / 3)
		let current = { x: 0, y: 0 }

		const addPoint = (x: number, y: number) => {
			const point = new fabricModule.Point(x - pathOffset.x, y - pathOffset.y).transform(transform)
			const previous = sampled.at(-1)
			if (!previous || !previous.eq(point)) sampled.push(point)
		}
		const addLine = (end: { x: number; y: number }) => {
			const steps = Math.max(
				1,
				Math.ceil(Math.hypot(end.x - current.x, end.y - current.y) / spacing),
			)
			for (let step = 1; step <= steps; step++) {
				const progress = step / steps
				addPoint(
					current.x + (end.x - current.x) * progress,
					current.y + (end.y - current.y) * progress,
				)
			}
			current = end
		}

		for (const pathCommand of eraserPath.path) {
			const command = pathCommand[0]
			if (command === 'M') {
				current = { x: Number(pathCommand[1]), y: Number(pathCommand[2]) }
				addPoint(current.x, current.y)
			} else if (command === 'L') {
				addLine({ x: Number(pathCommand[1]), y: Number(pathCommand[2]) })
			} else if (command === 'Q') {
				const start = current
				const control = { x: Number(pathCommand[1]), y: Number(pathCommand[2]) }
				const end = { x: Number(pathCommand[3]), y: Number(pathCommand[4]) }
				const estimatedLength =
					Math.hypot(control.x - start.x, control.y - start.y) +
					Math.hypot(end.x - control.x, end.y - control.y)
				const steps = Math.max(2, Math.ceil(estimatedLength / spacing))
				for (let step = 1; step <= steps; step++) {
					const progress = step / steps
					const inverse = 1 - progress
					addPoint(
						inverse * inverse * start.x +
							2 * inverse * progress * control.x +
							progress * progress * end.x,
						inverse * inverse * start.y +
							2 * inverse * progress * control.y +
							progress * progress * end.y,
					)
				}
				current = end
			}
		}

		if (sampled.length === 0) addPoint(0, 0)
		return sampled
	}

	function handleMouseDown(event: FabricPointerEvent) {
		const editorCanvas = canvas.value
		if (!editorCanvas || !fabric || editorCanvas.isDrawingMode || tool.value === 'select') return
		const point = editorCanvas.getScenePoint(event.e)
		if (tool.value === 'eraser' && eraserMode.value === 'element') {
			beginErasing(point)
			return
		}

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
		if (!editorCanvas) return
		if (tool.value === 'eraser' && eraserMode.value === 'element') {
			if (erasing) eraseAtPoint(editorCanvas.getScenePoint(event.e))
			return
		}
		if (!drawingStart || !drawingObject) return
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
		if (!editorCanvas) return
		if (erasing) {
			eraseAtPoint(editorCanvas.getScenePoint(event.e))
			finishErasing()
			return
		}
		if (!fabric || !drawingStart || !drawingObject) return
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
			const editorCensor = censor as EditorFabricObject
			editorCensor.censorMode = censorMode.value
			editorCensor.censorColor = color.value
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

	function refreshModifiedCensors(target: EditorFabricObject) {
		const targets =
			'getObjects' in target && typeof target.getObjects === 'function'
				? (target.getObjects() as EditorFabricObject[])
				: [target]
		for (const object of targets) refreshCensor(object)
	}

	function refreshCensor(object: EditorFabricObject) {
		if (
			object.editorKind !== 'censor' ||
			object.censorMode !== 'blur' ||
			!sourceImage ||
			object.width <= 0 ||
			object.height <= 0
		) {
			return
		}
		const censorCanvas = renderCensorRegion(
			sourceImage,
			{ left: 0, top: 0, width: object.width, height: object.height },
			'blur',
			object.censorColor ?? '#000000',
			object.calcTransformMatrix(),
		)
		;(object as FabricImage).setElement(censorCanvas, {
			width: object.width,
			height: object.height,
		})
		object.dirty = true
		object.setCoords()
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
			objects: annotationObjects().map((object) => {
				const state = object.toObject([
					'editorKind',
					'sourceRect',
					'censorMode',
					'censorColor',
				]) as ScreenshotEditorObjectState
				if (object.editorKind === 'censor') delete state.src
				return state
			}),
		}
	}

	async function enlivenEditorObjects(states: ScreenshotEditorObjectState[]) {
		const fabricModule = fabric
		const editorSourceImage = sourceImage
		if (!fabricModule || !editorSourceImage) return []
		const enlivenObjects = fabricModule.util.enlivenObjects as unknown as (
			objects: ScreenshotEditorObjectState[],
		) => Promise<EditorFabricObject[]>

		return await Promise.all(
			states.map(async (state) => {
				if (state.editorKind !== 'censor') {
					const [object] = await enlivenObjects([state])
					if (!object) throw new Error('Could not restore screenshot annotation')
					return object
				}

				if (!isSourceRect(state.sourceRect)) {
					throw new Error('Could not restore screenshot censor region')
				}
				const restoredMode: ScreenshotCensorMode = state.censorMode === 'solid' ? 'solid' : 'blur'
				const restoredColor = typeof state.censorColor === 'string' ? state.censorColor : '#000000'
				const serializedClipPath =
					state.clipPath && typeof state.clipPath === 'object'
						? (state.clipPath as ScreenshotEditorObjectState)
						: undefined
				const censorCanvas = renderCensorRegion(
					editorSourceImage,
					state.sourceRect,
					restoredMode,
					restoredColor,
				)
				const objectOptions = Object.fromEntries(
					Object.entries(state).filter(
						([property]) => !CENSOR_REGENERATED_PROPERTIES.has(property),
					),
				) as ScreenshotEditorObjectState
				const censor = new fabricModule.FabricImage(
					censorCanvas,
					objectOptions as unknown as FabricImageOptions,
				) as EditorFabricObject
				setEditorMetadata(censor, 'censor', state.sourceRect)
				censor.censorMode = restoredMode
				censor.censorColor = restoredColor
				refreshCensor(censor)
				if (serializedClipPath) {
					const [clipPath] = await enlivenObjects([serializedClipPath])
					if (clipPath) censor.clipPath = clipPath
				}
				return censor
			}),
		)
	}

	async function restoreHistory(index: number) {
		const editorCanvas = canvas.value
		const entry = history.value[index]
		if (!editorCanvas || !entry || !fabric) return
		restoringHistory = true
		try {
			editorCanvas.discardActiveObject()
			editorCanvas.remove(...annotationObjects())
			const restored = await enlivenEditorObjects(entry.objects)
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

	function exportEditorState() {
		if (!persistEditorState) return null
		const document: ScreenshotEditorDocument = {
			version: 1,
			...snapshot(),
		}
		return JSON.stringify(document)
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
			event.preventDefault()
			deleteSelection()
			return true
		}
		if (event.key === 'Escape') {
			event.preventDefault()
			if (erasing) {
				finishErasing()
				return true
			}
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
		persistEditorState = false
		erasing = false
		erasedDuringGesture = false
	}

	return {
		loading,
		tool,
		color,
		strokeWidth,
		fontSize,
		censorMode,
		eraserMode,
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
		showEraserMode,
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
		exportEditorState,
		handleKeyboardShortcut,
		isTextEditing,
		resetHistory,
	}
}

function parseEditorDocument(value: string | null | undefined) {
	if (!value) return undefined
	try {
		const document = JSON.parse(value) as Partial<ScreenshotEditorDocument>
		if (document.version !== 1 || !Array.isArray(document.objects)) return undefined
		return document as ScreenshotEditorDocument
	} catch {
		return undefined
	}
}

function isSourceRect(value: unknown): value is ScreenshotEditorSourceRect {
	if (!value || typeof value !== 'object') return false
	const rect = value as Partial<ScreenshotEditorSourceRect>
	return (
		typeof rect.left === 'number' &&
		Number.isFinite(rect.left) &&
		typeof rect.top === 'number' &&
		Number.isFinite(rect.top) &&
		typeof rect.width === 'number' &&
		Number.isFinite(rect.width) &&
		rect.width > 0 &&
		typeof rect.height === 'number' &&
		Number.isFinite(rect.height) &&
		rect.height > 0
	)
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

function boundingRectsOverlap(
	left: { left: number; top: number; width: number; height: number },
	right: { left: number; top: number; width: number; height: number },
) {
	return (
		left.left <= right.left + right.width &&
		left.left + left.width >= right.left &&
		left.top <= right.top + right.height &&
		left.top + left.height >= right.top
	)
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
