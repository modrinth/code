<template>
	<canvas ref="canvasRef" :style="{ touchAction: props.pinnedSliceIndex === null ? undefined : 'none' }" />
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

import { type ChartDataset, formatAxisValue } from './utils'

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

const props = defineProps<{
	type: 'line' | 'bar'
	fill: boolean
	stacked: boolean
	datasets: ChartDataset[]
	labels: string[]
	activeStat: AnalyticsDashboardStat
	pinnedSliceIndex: number | null
}>()

const emit = defineEmits<{
	(event: 'hover', payload: AnalyticsChartHoverPayload): void
	(event: 'pinned-drag', payload: AnalyticsChartHoverPayload): void
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

let pinnedDragPointerId: number | null = null
let pinnedDragStartX = 0
let pinnedDragStartY = 0
let isPinnedDragging = false

function getChartEvents(): ChartEvents {
	return props.pinnedSliceIndex === null ? [...chartInteractionEvents] : []
}

function getPinnedPointBorderWidth() {
	return props.pinnedSliceIndex === null ? 0 : 2
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

function withAlpha(color: string, alpha: number): string {
	const match = /^#([0-9a-f]{6})$/i.exec(color)
	if (!match) return color
	const r = Number.parseInt(match[1].slice(0, 2), 16)
	const g = Number.parseInt(match[1].slice(2, 4), 16)
	const b = Number.parseInt(match[1].slice(4, 6), 16)
	return `rgba(${r}, ${g}, ${b}, ${alpha})`
}

function darkenColor(color: string, amount: number): string {
	const match = /^#([0-9a-f]{6})$/i.exec(color)
	if (!match) return color
	const r = Math.round(Number.parseInt(match[1].slice(0, 2), 16) * (1 - amount))
	const g = Math.round(Number.parseInt(match[1].slice(2, 4), 16) * (1 - amount))
	const b = Math.round(Number.parseInt(match[1].slice(4, 6), 16) * (1 - amount))
	return `#${[r, g, b].map((value) => value.toString(16).padStart(2, '0')).join('')}`
}

function buildDatasets() {
	return props.datasets.map((dataset, index) => {
		const pointBorderColor = darkenColor(dataset.borderColor, 0.4)
		const common = {
			label: dataset.label,
			data: dataset.data,
			borderColor: dataset.borderColor,
			borderWidth: 2,
		}

		if (props.type === 'bar') {
			return {
				...common,
				backgroundColor: withAlpha(dataset.backgroundColor, 0.85),
				borderWidth: 0,
				stack: props.stacked ? 'analytics' : undefined,
			}
		}

		const lineFill: 'origin' | '-1' | false = props.fill ? (index === 0 ? 'origin' : '-1') : false

		return {
			...common,
			backgroundColor: props.fill
				? withAlpha(dataset.backgroundColor, 0.3)
				: dataset.backgroundColor,
			fill: lineFill,
			tension: 0.35,
			pointRadius: 0,
			pointBackgroundColor: dataset.borderColor,
			pointBorderColor,
			pointBorderWidth: 0,
			pointHoverRadius: 4,
			pointHoverBackgroundColor: dataset.borderColor,
			pointHoverBorderColor: pointBorderColor,
			pointHoverBorderWidth: getPinnedPointBorderWidth,
			pointHitRadius: 16,
			stack: props.stacked ? 'analytics' : undefined,
		}
	})
}

function buildConfig(): ChartConfiguration {
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
					grid: { display: false },
					ticks: {
						maxTicksLimit: 8,
						autoSkip: true,
						color: 'rgba(148, 163, 184, 0.9)',
					},
					border: { color: 'rgba(148, 163, 184, 0.35)' },
				},
				y: {
					stacked: props.stacked,
					beginAtZero: true,
					grid: {
						color: 'rgba(148, 163, 184, 0.15)',
					},
					border: { display: false },
					ticks: {
						color: 'rgba(148, 163, 184, 0.9)',
						callback: (tickValue) => {
							const numeric =
								typeof tickValue === 'number' ? tickValue : Number.parseFloat(String(tickValue))
							if (!Number.isFinite(numeric)) return String(tickValue)
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

	if (props.pinnedSliceIndex === null) {
		chartInstance.setActiveElements([])
		chartInstance.update('none')
		return
	}

	chartInstance.setActiveElements(getPinnedActiveElements(props.pinnedSliceIndex))
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

onMounted(() => {
	createChart()
	canvasRef.value?.addEventListener('mouseleave', handleCanvasLeave)
	canvasRef.value?.addEventListener('pointerdown', handlePinnedPointerDown)
	canvasRef.value?.addEventListener('pointermove', handlePinnedPointerMove)
	canvasRef.value?.addEventListener('pointerup', handlePinnedPointerEnd)
	canvasRef.value?.addEventListener('pointercancel', handlePinnedPointerEnd)
})

onBeforeUnmount(() => {
	canvasRef.value?.removeEventListener('mouseleave', handleCanvasLeave)
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
	() => [props.datasets, props.labels, props.activeStat],
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
