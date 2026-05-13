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
}>()

const emit = defineEmits<{
	(event: 'hover' | 'pinned-drag', payload: AnalyticsChartHoverPayload): void
	(event: 'range-select', payload: AnalyticsChartRangeSelectPayload): void
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
const CSS_VARIABLE_COLOR_PATTERN = /^var\(\s*(--[a-z0-9-_]+)\s*\)$/i

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
	const match = /^#([0-9a-f]{6})$/i.exec(color)
	if (!match) return color
	const r = Number.parseInt(match[1].slice(0, 2), 16)
	const g = Number.parseInt(match[1].slice(2, 4), 16)
	const b = Number.parseInt(match[1].slice(4, 6), 16)
	return `rgba(${r}, ${g}, ${b}, ${alpha})`
}

function resolveCssColor(color: string): string {
	const match = CSS_VARIABLE_COLOR_PATTERN.exec(color)
	if (!match || typeof document === 'undefined') {
		return color
	}

	const resolvedColor = getComputedStyle(document.documentElement).getPropertyValue(match[1]).trim()
	return resolvedColor || color
}

function buildDatasets() {
	return props.datasets.map((dataset, index) => {
		const borderColor = resolveCssColor(dataset.borderColor)
		const backgroundColor = resolveCssColor(dataset.backgroundColor)
		const common = {
			label: dataset.label,
			data: dataset.data,
			borderColor,
			borderWidth: 2,
		}

		if (props.type === 'bar') {
			return {
				...common,
				backgroundColor: withAlpha(backgroundColor, 0.85),
				borderWidth: 0,
				stack: props.stacked ? 'analytics' : undefined,
			}
		}

		const lineFill: 'origin' | '-1' | false = props.fill ? (index === 0 ? 'origin' : '-1') : false

		return {
			...common,
			backgroundColor: props.fill ? withAlpha(backgroundColor, 0.3) : backgroundColor,
			fill: lineFill,
			tension: 0.35,
			pointRadius: 0,
			pointBackgroundColor: borderColor,
			pointBorderWidth: 0,
			pointHoverRadius: 4,
			pointHoverBackgroundColor: borderColor,
			pointHoverBorderWidth: 0,
			pointHitRadius: 16,
			stack: props.stacked ? 'analytics' : undefined,
		}
	})
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
		data: {
			labels: props.labels,
			datasets: buildDatasets() as ChartConfiguration['data']['datasets'],
		},
		options: {
			responsive: true,
			maintainAspectRatio: false,
			animation: false,
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
	chartInstance = new Chart(canvasRef.value, buildConfig())
}

function refreshChart() {
	if (!chartInstance) return
	const config = buildConfig()
	chartInstance.data = config.data
	chartInstance.options = config.options ?? {}
	chartInstance.update('none')
	applyPinnedSliceState()
}

function applyPinnedSliceState() {
	if (!chartInstance) return

	chartInstance.options.events = getChartEvents()
	chartInstance.setActiveElements(
		props.pinnedSliceIndex === null ? [] : getPinnedActiveElements(props.pinnedSliceIndex),
	)
	chartInstance.update('none')
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
	() => [props.datasets, props.labels, props.xAxisTickLimit, props.activeStat],
	() => {
		refreshChart()
	},
	{ deep: true },
)

watch(
	() => props.pinnedSliceIndex,
	() => {
		applyPinnedSliceState()
	},
)
</script>
