<template>
	<div class="relative h-full">
		<canvas
			ref="canvasRef"
			class="h-full w-full"
			:style="{ touchAction: props.pinnedSliceIndex === null ? 'pan-y' : 'none' }"
		/>
		<div
			v-if="rangeSelection.visible"
			aria-hidden="true"
			class="pointer-events-none absolute z-10 rounded-sm border border-dashed border-brand bg-brand-highlight opacity-20"
			:style="rangeSelectionStyle"
		/>
	</div>
</template>

<script setup lang="ts">
import { useCompactNumber } from '@modrinth/ui'
import {
	BarController,
	BarElement,
	CategoryScale,
	Chart,
	type ChartConfiguration,
	Filler,
	LinearScale,
	LineController,
	LineElement,
	PointElement,
	Tooltip,
} from 'chart.js'

import type { AnalyticsDashboardStat } from '~/providers/analytics/analytics'

import { type ChartDataset, DEFAULT_X_AXIS_TICK_LIMIT, formatAxisValue } from './utils'

Chart.register(
	LineController,
	BarController,
	LineElement,
	BarElement,
	PointElement,
	CategoryScale,
	LinearScale,
	Filler,
	Tooltip,
)

export type AnalyticsChartHoverPayload = {
	visible: boolean
	x: number
	y: number
	sliceIndex: number | null
}

export type AnalyticsChartRangeSelectPayload = {
	startSliceIndex: number
	endSliceIndex: number
}

export type AnalyticsChartGeometryPayload = {
	left: number
	right: number
	top: number
	bottom: number
	width: number
	height: number
	xPositions: number[]
}

const props = defineProps<{
	type: 'line' | 'bar'
	fill: boolean
	stacked: boolean
	ratioMode: boolean
	datasets: ChartDataset[]
	labels: string[]
	xAxisTickLimit?: number
	activeStat: AnalyticsDashboardStat
	pinnedSliceIndex: number | null
	highlightedDatasetId: string | null
}>()

const emit = defineEmits<{
	(event: 'hover' | 'pinned-drag', payload: AnalyticsChartHoverPayload): void
	(event: 'range-select', payload: AnalyticsChartRangeSelectPayload): void
	(event: 'geometry', payload: AnalyticsChartGeometryPayload): void
}>()

const canvasRef = ref<HTMLCanvasElement | null>(null)
let chartInstance: Chart | null = null

const { formatCompactNumber } = useCompactNumber()

type ExternalTooltipHandler = NonNullable<
	NonNullable<NonNullable<ChartConfiguration['options']>['plugins']>['tooltip']
>['external']
type ExternalTooltipContext = Parameters<Exclude<ExternalTooltipHandler, undefined>>[0]
type ChartEvents = NonNullable<NonNullable<ChartConfiguration['options']>['events']>

const chartInteractionEvents: ChartEvents = [
	'mousemove',
	'mouseout',
	'click',
	'touchstart',
	'touchmove',
]
const PINNED_DRAG_THRESHOLD_PX = 6
const RANGE_SELECT_THRESHOLD_PX = 8
const EMPTY_DATA_Y_AXIS_MAX = 10
const EMPTY_DATA_Y_AXIS_STEP = 2
const Y_AXIS_WIDTH = 40
const SECONDS_PER_HOUR = 60 * 60
const DIMMED_SERIES_OPACITY = 0.5
const BAR_BACKGROUND_OPACITY = 0.85
const AREA_BACKGROUND_OPACITY = 0.3
const SERIES_OPACITY_TRANSITION_MS = 150
const CSS_VARIABLE_COLOR_PATTERN = /^var\(\s*(--[a-z0-9-_]+)\s*\)$/i
const HSL_COLOR_PATTERN = /^hsl\(\s*([0-9.]+)(?:deg)?\s*,\s*([0-9.]+)%\s*,\s*([0-9.]+)%\s*\)$/i

let pinnedDragPointerId: number | null = null
let pinnedDragStartX = 0
let pinnedDragStartY = 0
let isPinnedDragging = false
let rangeSelectPointerId: number | null = null
let rangeSelectStartX = 0
let rangeSelectStartY = 0
let rangeSelectStartSliceIndex: number | null = null
let rangeSelectLastSliceIndex: number | null = null
let isRangeSelecting = false
let seriesOpacityAnimationFrame: number | null = null
let currentDatasetOpacities: number[] = []
let suppressGeometryEmit = false
let lastGeometryPayload: AnalyticsChartGeometryPayload | null = null

const geometryPlugin = {
	id: 'analytics-chart-geometry',
	afterLayout(chart: Chart) {
		if (suppressGeometryEmit) return
		emitChartGeometry(chart)
	},
}

const rangeSelection = reactive({
	visible: false,
	startX: 0,
	currentX: 0,
	top: 0,
	bottom: 0,
})

const rangeSelectionStyle = computed(() => {
	const left = Math.min(rangeSelection.startX, rangeSelection.currentX)
	const width = Math.max(1, Math.abs(rangeSelection.currentX - rangeSelection.startX))

	return {
		top: `${rangeSelection.top}px`,
		bottom: `${rangeSelection.bottom}px`,
		transform: `translate(${left}px, 0)`,
		width: `${width}px`,
	}
})

function getChartEvents(): ChartEvents {
	return props.pinnedSliceIndex === null ? [...chartInteractionEvents] : []
}

function emitChartGeometry(chart: Chart | null = chartInstance) {
	if (!chart || !canvasRef.value) return

	const chartArea = chart.chartArea
	const rect = canvasRef.value.getBoundingClientRect()
	if (
		!Number.isFinite(chartArea.left) ||
		!Number.isFinite(chartArea.right) ||
		!Number.isFinite(chartArea.top) ||
		!Number.isFinite(chartArea.bottom) ||
		chartArea.right <= chartArea.left ||
		chartArea.bottom <= chartArea.top
	) {
		return
	}

	const payload = {
		left: chartArea.left,
		right: chartArea.right,
		top: chartArea.top,
		bottom: chartArea.bottom,
		width: rect.width,
		height: rect.height,
		xPositions: props.labels
			.map((_, index) => chart.scales.x.getPixelForValue(index))
			.filter((x) => Number.isFinite(x)),
	}
	if (areChartGeometryPayloadsEqual(lastGeometryPayload, payload)) {
		return
	}

	lastGeometryPayload = payload
	emit('geometry', payload)
}

function getPinnedActiveElements(sliceIndex: number) {
	if (!chartInstance) return []

	const activeElements: { datasetIndex: number; index: number }[] = []
	for (let datasetIndex = 0; datasetIndex < chartInstance.data.datasets.length; datasetIndex++) {
		const dataset = chartInstance.data.datasets[datasetIndex]
		if (!dataset) continue

		const dataLength = Array.isArray(dataset.data) ? dataset.data.length : 0
		if (sliceIndex >= dataLength) continue

		activeElements.push({
			datasetIndex,
			index: sliceIndex,
		})
	}

	return activeElements
}

function getNearestSliceIndex(clientX: number) {
	if (!chartInstance || !canvasRef.value || props.labels.length === 0) return null

	const rect = canvasRef.value.getBoundingClientRect()
	const x = clientX - rect.left
	const xScale = chartInstance.scales.x
	const rawIndex = xScale.getValueForPixel(x)
	if (typeof rawIndex !== 'number' || !Number.isFinite(rawIndex)) return null

	return Math.min(props.labels.length - 1, Math.max(0, Math.round(rawIndex)))
}

function getSliceChartPosition(sliceIndex: number) {
	if (!chartInstance || !canvasRef.value) return null

	const rect = canvasRef.value.getBoundingClientRect()
	const chartArea = chartInstance.chartArea
	const xScale = chartInstance.scales.x
	const x = xScale.getPixelForValue(sliceIndex)
	if (!Number.isFinite(x)) return null

	return {
		x: Math.min(chartArea.right, Math.max(chartArea.left, x)),
		top: chartArea.top,
		bottom: rect.height - chartArea.bottom,
	}
}

function updateRangeSelection(sliceIndex: number) {
	const chartPosition = getSliceChartPosition(sliceIndex)
	if (!chartPosition) return

	rangeSelection.visible = true
	rangeSelection.currentX = chartPosition.x
	rangeSelection.top = chartPosition.top
	rangeSelection.bottom = chartPosition.bottom
}

function clearRangeSelection() {
	rangeSelection.visible = false
}

function areChartGeometryPayloadsEqual(
	left: AnalyticsChartGeometryPayload | null,
	right: AnalyticsChartGeometryPayload,
): boolean {
	if (!left) return false
	if (
		left.left !== right.left ||
		left.right !== right.right ||
		left.top !== right.top ||
		left.bottom !== right.bottom ||
		left.width !== right.width ||
		left.height !== right.height ||
		left.xPositions.length !== right.xPositions.length
	) {
		return false
	}

	for (let index = 0; index < left.xPositions.length; index++) {
		if (left.xPositions[index] !== right.xPositions[index]) {
			return false
		}
	}

	return true
}

function getPinnedTooltipPosition(sliceIndex: number) {
	if (!chartInstance) return null

	const activeElements = getPinnedActiveElements(sliceIndex)
	if (activeElements.length === 0) return null

	const positions = activeElements
		.map(({ datasetIndex, index }) => {
			const element = chartInstance?.getDatasetMeta(datasetIndex).data[index]
			if (!element) return null
			const point = element.getProps(['x', 'y'], true) as { x: number; y: number }
			if (!Number.isFinite(point.x) || !Number.isFinite(point.y)) return null
			return point
		})
		.filter((position): position is { x: number; y: number } => Boolean(position))

	if (positions.length === 0) return null

	const x = positions.reduce((sum, position) => sum + position.x, 0) / positions.length
	const y = positions.reduce((sum, position) => sum + position.y, 0) / positions.length

	return { x, y }
}

function emitPinnedDragHover(sliceIndex: number) {
	const position = getPinnedTooltipPosition(sliceIndex)
	if (!position) return

	emit('pinned-drag', {
		visible: true,
		x: position.x,
		y: position.y,
		sliceIndex,
	})
}

function emitRangeDragHover(sliceIndex: number) {
	const position = getPinnedTooltipPosition(sliceIndex)
	const fallbackPosition = getSliceChartPosition(sliceIndex)
	if (!position && !fallbackPosition) return

	emit('hover', {
		visible: true,
		x: position?.x ?? fallbackPosition?.x ?? 0,
		y: position?.y ?? fallbackPosition?.top ?? 0,
		sliceIndex,
	})
}

function withAlpha(color: string, alpha: number): string {
	const hexMatch = /^#([0-9a-f]{6})$/i.exec(color)
	if (hexMatch) {
		const r = Number.parseInt(hexMatch[1].slice(0, 2), 16)
		const g = Number.parseInt(hexMatch[1].slice(2, 4), 16)
		const b = Number.parseInt(hexMatch[1].slice(4, 6), 16)
		return `rgba(${r}, ${g}, ${b}, ${alpha})`
	}

	const hslMatch = HSL_COLOR_PATTERN.exec(color)
	if (!hslMatch) return color

	const hue = Number.parseFloat(hslMatch[1])
	const saturation = Number.parseFloat(hslMatch[2])
	const lightness = Number.parseFloat(hslMatch[3])
	if (![hue, saturation, lightness].every(Number.isFinite)) return color

	return `hsla(${hue}, ${saturation}%, ${lightness}%, ${alpha})`
}

function resolveCssColor(color: string): string {
	const match = CSS_VARIABLE_COLOR_PATTERN.exec(color)
	if (!match || typeof document === 'undefined') {
		return color
	}

	const resolvedColor = getComputedStyle(document.documentElement).getPropertyValue(match[1]).trim()
	return resolvedColor || color
}

function getTargetDatasetOpacity(index: number) {
	const dataset = props.datasets[index]
	return props.highlightedDatasetId !== null && dataset?.projectId !== props.highlightedDatasetId
		? DIMMED_SERIES_OPACITY
		: 1
}

function getDatasetOpacity(index: number) {
	return currentDatasetOpacities[index] ?? getTargetDatasetOpacity(index)
}

function getDatasetColors(dataset: ChartDataset, index: number) {
	const opacity = getDatasetOpacity(index)
	return {
		borderColor: withAlpha(resolveCssColor(dataset.borderColor), opacity),
		backgroundColor: resolveCssColor(dataset.backgroundColor),
		opacity,
	}
}

function buildDatasets() {
	return props.datasets.map((dataset, index) => {
		const colors = getDatasetColors(dataset, index)
		const common = {
			label: dataset.label,
			data: dataset.data,
			borderColor: colors.borderColor,
			borderWidth: 2,
		}

		if (props.type === 'bar') {
			return {
				...common,
				backgroundColor: withAlpha(colors.backgroundColor, BAR_BACKGROUND_OPACITY * colors.opacity),
				borderWidth: 0,
				stack: props.stacked ? 'analytics' : undefined,
			}
		}

		const lineFill: 'origin' | '-1' | false = props.fill ? (index === 0 ? 'origin' : '-1') : false

		return {
			...common,
			backgroundColor: props.fill
				? withAlpha(colors.backgroundColor, AREA_BACKGROUND_OPACITY * colors.opacity)
				: withAlpha(colors.backgroundColor, colors.opacity),
			fill: lineFill,
			tension: 0.35,
			pointRadius: 0,
			pointBackgroundColor: colors.borderColor,
			pointBorderWidth: 0,
			pointHoverRadius: 4,
			pointHoverBackgroundColor: colors.borderColor,
			pointHoverBorderWidth: 0,
			pointHitRadius: 16,
			stack: props.stacked ? 'analytics' : undefined,
		}
	})
}

function cancelSeriesOpacityAnimation() {
	if (seriesOpacityAnimationFrame === null) return

	cancelAnimationFrame(seriesOpacityAnimationFrame)
	seriesOpacityAnimationFrame = null
}

function getTargetDatasetOpacities() {
	return props.datasets.map((_, index) => getTargetDatasetOpacity(index))
}

function syncDatasetOpacitiesToTargets() {
	cancelSeriesOpacityAnimation()
	currentDatasetOpacities = getTargetDatasetOpacities()
}

function updateChartWithoutGeometry() {
	if (!chartInstance) return

	suppressGeometryEmit = true
	try {
		chartInstance.update('none')
	} finally {
		suppressGeometryEmit = false
	}
}

function applySeriesHoverState() {
	if (!chartInstance) return

	chartInstance.data.datasets.forEach((chartDataset, index) => {
		const dataset = props.datasets[index]
		if (!dataset) return

		const colors = getDatasetColors(dataset, index)
		chartDataset.borderColor = colors.borderColor
		chartDataset.backgroundColor =
			props.type === 'bar'
				? withAlpha(colors.backgroundColor, BAR_BACKGROUND_OPACITY * colors.opacity)
				: props.fill
					? withAlpha(colors.backgroundColor, AREA_BACKGROUND_OPACITY * colors.opacity)
					: withAlpha(colors.backgroundColor, colors.opacity)
		Object.assign(chartDataset, {
			pointBackgroundColor: colors.borderColor,
			pointHoverBackgroundColor: colors.borderColor,
		})
	})

	updateChartWithoutGeometry()
}

function easeSeriesOpacityTransition(progress: number) {
	return 1 - Math.pow(1 - progress, 3)
}

function animateSeriesHoverState() {
	if (!chartInstance) return
	if (typeof requestAnimationFrame === 'undefined') {
		syncDatasetOpacitiesToTargets()
		applySeriesHoverState()
		return
	}

	cancelSeriesOpacityAnimation()

	const from = props.datasets.map((_, index) => getDatasetOpacity(index))
	const to = getTargetDatasetOpacities()
	if (from.every((opacity, index) => Math.abs(opacity - (to[index] ?? 1)) < 0.001)) {
		currentDatasetOpacities = to
		applySeriesHoverState()
		return
	}

	const start = performance.now()
	const tick = (now: number) => {
		const progress = Math.min(1, (now - start) / SERIES_OPACITY_TRANSITION_MS)
		const easedProgress = easeSeriesOpacityTransition(progress)
		currentDatasetOpacities = to.map(
			(targetOpacity, index) =>
				(from[index] ?? targetOpacity) +
				(targetOpacity - (from[index] ?? targetOpacity)) * easedProgress,
		)
		applySeriesHoverState()

		if (progress < 1) {
			seriesOpacityAnimationFrame = requestAnimationFrame(tick)
			return
		}

		seriesOpacityAnimationFrame = null
		currentDatasetOpacities = to
		applySeriesHoverState()
	}

	seriesOpacityAnimationFrame = requestAnimationFrame(tick)
}

function getVisibleXAxisLabelIndexes(labelCount: number, limit: number): Set<number> {
	if (limit <= 0 || labelCount <= limit) {
		return new Set(Array.from({ length: labelCount }, (_, index) => index))
	}

	const indexes = new Set<number>()
	for (let i = 0; i < limit; i++) {
		indexes.add(Math.floor((i * labelCount) / limit))
	}

	return indexes
}

function hasMetricData() {
	return props.datasets.some((dataset) =>
		dataset.data.some((value) => Number.isFinite(value) && value > 0),
	)
}

function getEmptyDataYAxisMax() {
	return props.activeStat === 'playtime'
		? EMPTY_DATA_Y_AXIS_MAX * SECONDS_PER_HOUR
		: EMPTY_DATA_Y_AXIS_MAX
}

function getEmptyDataYAxisStepSize() {
	return props.activeStat === 'playtime'
		? EMPTY_DATA_Y_AXIS_STEP * SECONDS_PER_HOUR
		: EMPTY_DATA_Y_AXIS_STEP
}

function buildConfig(): ChartConfiguration {
	const hasData = hasMetricData()
	const visibleXAxisLabelIndexes =
		props.xAxisTickLimit === undefined
			? null
			: getVisibleXAxisLabelIndexes(props.labels.length, props.xAxisTickLimit)

	return {
		type: props.type,
		plugins: [geometryPlugin],
		data: {
			labels: props.labels,
			datasets: buildDatasets() as ChartConfiguration['data']['datasets'],
		},
		options: {
			responsive: true,
			maintainAspectRatio: false,
			animation: false,
			normalized: true,
			events: getChartEvents(),
			interaction: {
				mode: 'index',
				intersect: false,
			},
			plugins: {
				legend: { display: false },
				tooltip: {
					enabled: false,
					external: handleExternalTooltip,
				},
			},
			scales: {
				x: {
					stacked: props.stacked && props.type === 'bar',
					offset: props.type === 'bar',
					grid: { display: false },
					ticks: {
						align: 'inner',
						maxTicksLimit: props.xAxisTickLimit ?? DEFAULT_X_AXIS_TICK_LIMIT,
						autoSkip: !props.xAxisTickLimit,
						color: 'rgba(148, 163, 184, 0.9)',
						callback: (tickValue, index) => {
							if (visibleXAxisLabelIndexes && !visibleXAxisLabelIndexes.has(index)) {
								return ''
							}

							return props.labels[Number(tickValue)] ?? ''
						},
					},
					border: { color: 'rgba(148, 163, 184, 0.35)' },
				},
				y: {
					stacked: props.stacked,
					beginAtZero: true,
					afterFit: (scale) => {
						scale.width = Y_AXIS_WIDTH
					},
					...(props.ratioMode
						? { max: 100, min: 0 }
						: hasData
							? {}
							: { max: getEmptyDataYAxisMax(), min: 0 }),
					grid: {
						color: 'rgba(148, 163, 184, 0.15)',
					},
					border: { display: false },
					ticks: {
						color: 'rgba(148, 163, 184, 0.9)',
						...(props.ratioMode
							? { stepSize: 25 }
							: hasData
								? {}
								: { stepSize: getEmptyDataYAxisStepSize() }),
						callback: (tickValue) => {
							const numeric =
								typeof tickValue === 'number' ? tickValue : Number.parseFloat(String(tickValue))
							if (!Number.isFinite(numeric)) return String(tickValue)
							if (props.ratioMode) return `${numeric}%`
							return formatAxisValue(numeric, props.activeStat, formatCompactNumber)
						},
					},
				},
			},
		},
	}
}

function handleExternalTooltip(context: ExternalTooltipContext) {
	const tooltip = context.tooltip
	if (!tooltip || tooltip.opacity === 0) {
		emit('hover', { visible: false, x: 0, y: 0, sliceIndex: null })
		return
	}
	const sliceIndex = tooltip.dataPoints?.[0]?.dataIndex ?? null
	emit('hover', {
		visible: true,
		x: tooltip.caretX,
		y: tooltip.caretY,
		sliceIndex,
	})
}

function createChart() {
	if (!canvasRef.value) return
	syncDatasetOpacitiesToTargets()
	chartInstance = new Chart(canvasRef.value, buildConfig())
	emitChartGeometry()
}

function refreshChart() {
	if (!chartInstance) return
	syncDatasetOpacitiesToTargets()
	const config = buildConfig()
	chartInstance.data = config.data
	chartInstance.options = config.options ?? {}
	chartInstance.update('none')
	applyPinnedSliceState()
	emitChartGeometry()
}

function applyPinnedSliceState() {
	if (!chartInstance) return

	chartInstance.options.events = getChartEvents()
	chartInstance.setActiveElements(
		props.pinnedSliceIndex === null ? [] : getPinnedActiveElements(props.pinnedSliceIndex),
	)
	updateChartWithoutGeometry()
}

function handleCanvasLeave() {
	emit('hover', { visible: false, x: 0, y: 0, sliceIndex: null })
	if (props.pinnedSliceIndex !== null) {
		requestAnimationFrame(() => applyPinnedSliceState())
	}
}

function handlePinnedPointerDown(event: PointerEvent) {
	if (props.pinnedSliceIndex === null || event.pointerType !== 'touch' || !canvasRef.value) return

	pinnedDragPointerId = event.pointerId
	pinnedDragStartX = event.clientX
	pinnedDragStartY = event.clientY
	isPinnedDragging = false
	canvasRef.value.setPointerCapture(event.pointerId)
}

function handlePinnedPointerMove(event: PointerEvent) {
	if (props.pinnedSliceIndex === null || event.pointerId !== pinnedDragPointerId) return

	const distance = Math.hypot(event.clientX - pinnedDragStartX, event.clientY - pinnedDragStartY)
	if (!isPinnedDragging && distance < PINNED_DRAG_THRESHOLD_PX) return

	const sliceIndex = getNearestSliceIndex(event.clientX)
	if (sliceIndex === null) return

	isPinnedDragging = true
	event.preventDefault()
	emitPinnedDragHover(sliceIndex)
}

function handlePinnedPointerEnd(event: PointerEvent) {
	if (event.pointerId !== pinnedDragPointerId) return

	canvasRef.value?.releasePointerCapture(event.pointerId)
	pinnedDragPointerId = null
	isPinnedDragging = false
}

function handleRangePointerDown(event: PointerEvent) {
	if (rangeSelectPointerId !== null) return
	if (!canvasRef.value || props.labels.length === 0) return
	if (event.pointerType === 'mouse' && event.button !== 0) return
	if (props.pinnedSliceIndex !== null && event.pointerType === 'touch') return

	const sliceIndex = getNearestSliceIndex(event.clientX)
	if (sliceIndex === null) return

	const chartPosition = getSliceChartPosition(sliceIndex)
	if (!chartPosition) return

	rangeSelectPointerId = event.pointerId
	rangeSelectStartX = event.clientX
	rangeSelectStartY = event.clientY
	rangeSelectStartSliceIndex = sliceIndex
	rangeSelectLastSliceIndex = sliceIndex
	isRangeSelecting = false
	rangeSelection.startX = chartPosition.x
	rangeSelection.currentX = chartPosition.x
	rangeSelection.top = chartPosition.top
	rangeSelection.bottom = chartPosition.bottom
	canvasRef.value.setPointerCapture(event.pointerId)
}

function handleRangePointerMove(event: PointerEvent) {
	if (event.pointerId !== rangeSelectPointerId) return

	const distance = Math.hypot(event.clientX - rangeSelectStartX, event.clientY - rangeSelectStartY)
	if (!isRangeSelecting && distance < RANGE_SELECT_THRESHOLD_PX) return

	const sliceIndex = getNearestSliceIndex(event.clientX)
	if (sliceIndex === null) return

	isRangeSelecting = true
	rangeSelectLastSliceIndex = sliceIndex
	event.preventDefault()
	updateRangeSelection(sliceIndex)
	emitRangeDragHover(sliceIndex)
}

function handleRangePointerEnd(event: PointerEvent) {
	if (event.pointerId !== rangeSelectPointerId) return

	canvasRef.value?.releasePointerCapture(event.pointerId)
	const startSliceIndex = rangeSelectStartSliceIndex
	const endSliceIndex = rangeSelectLastSliceIndex

	if (isRangeSelecting && startSliceIndex !== null && endSliceIndex !== null) {
		event.preventDefault()
		emit('range-select', { startSliceIndex, endSliceIndex })
	}

	rangeSelectPointerId = null
	rangeSelectStartSliceIndex = null
	rangeSelectLastSliceIndex = null
	isRangeSelecting = false
	clearRangeSelection()
}

function handleRangePointerCancel(event: PointerEvent) {
	if (event.pointerId !== rangeSelectPointerId) return

	canvasRef.value?.releasePointerCapture(event.pointerId)
	rangeSelectPointerId = null
	rangeSelectStartSliceIndex = null
	rangeSelectLastSliceIndex = null
	isRangeSelecting = false
	clearRangeSelection()
}

onMounted(() => {
	createChart()
	canvasRef.value?.addEventListener('mouseleave', handleCanvasLeave)
	canvasRef.value?.addEventListener('pointerdown', handleRangePointerDown)
	canvasRef.value?.addEventListener('pointermove', handleRangePointerMove)
	canvasRef.value?.addEventListener('pointerup', handleRangePointerEnd)
	canvasRef.value?.addEventListener('pointercancel', handleRangePointerCancel)
	canvasRef.value?.addEventListener('pointerdown', handlePinnedPointerDown)
	canvasRef.value?.addEventListener('pointermove', handlePinnedPointerMove)
	canvasRef.value?.addEventListener('pointerup', handlePinnedPointerEnd)
	canvasRef.value?.addEventListener('pointercancel', handlePinnedPointerEnd)
})

onBeforeUnmount(() => {
	canvasRef.value?.removeEventListener('mouseleave', handleCanvasLeave)
	canvasRef.value?.removeEventListener('pointerdown', handleRangePointerDown)
	canvasRef.value?.removeEventListener('pointermove', handleRangePointerMove)
	canvasRef.value?.removeEventListener('pointerup', handleRangePointerEnd)
	canvasRef.value?.removeEventListener('pointercancel', handleRangePointerCancel)
	canvasRef.value?.removeEventListener('pointerdown', handlePinnedPointerDown)
	canvasRef.value?.removeEventListener('pointermove', handlePinnedPointerMove)
	canvasRef.value?.removeEventListener('pointerup', handlePinnedPointerEnd)
	canvasRef.value?.removeEventListener('pointercancel', handlePinnedPointerEnd)
	cancelSeriesOpacityAnimation()
	chartInstance?.destroy()
	chartInstance = null
})

watch(
	() => [props.type, props.fill, props.stacked],
	() => {
		chartInstance?.destroy()
		chartInstance = null
		nextTick(() => {
			createChart()
			applyPinnedSliceState()
		})
	},
)

watch(
	() => [props.datasets, props.labels, props.xAxisTickLimit, props.activeStat, props.ratioMode],
	() => {
		refreshChart()
	},
)

watch(
	() => props.pinnedSliceIndex,
	() => {
		applyPinnedSliceState()
	},
)

watch(
	() => props.highlightedDatasetId,
	() => {
		animateSeriesHoverState()
	},
)
</script>
