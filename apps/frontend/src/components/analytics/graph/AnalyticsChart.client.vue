<template>
	<canvas ref="canvasRef" />
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
	type TooltipModel,
} from 'chart.js'

import type { AnalyticsDashboardStat } from '~/providers/analytics'

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
}>()

const emit = defineEmits<{
	(event: 'hover', payload: AnalyticsChartHoverPayload): void
}>()

const canvasRef = ref<HTMLCanvasElement | null>(null)
let chartInstance: Chart | null = null

const { formatCompactNumber } = useCompactNumber()

function withAlpha(color: string, alpha: number): string {
	const match = /^#([0-9a-f]{6})$/i.exec(color)
	if (!match) return color
	const r = Number.parseInt(match[1].slice(0, 2), 16)
	const g = Number.parseInt(match[1].slice(2, 4), 16)
	const b = Number.parseInt(match[1].slice(4, 6), 16)
	return `rgba(${r}, ${g}, ${b}, ${alpha})`
}

function buildDatasets() {
	return props.datasets.map((dataset, index) => {
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

		const lineFill: 'origin' | '-1' | false = props.fill
			? index === 0
				? 'origin'
				: '-1'
			: false

		return {
			...common,
			backgroundColor: props.fill
				? withAlpha(dataset.backgroundColor, 0.3)
				: dataset.backgroundColor,
			fill: lineFill,
			tension: 0.35,
			pointRadius: 0,
			pointHoverRadius: 4,
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

function handleExternalTooltip(context: { tooltip: TooltipModel<'line' | 'bar'> }) {
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
}

function handleCanvasLeave() {
	emit('hover', { visible: false, x: 0, y: 0, sliceIndex: null })
}

onMounted(() => {
	createChart()
	canvasRef.value?.addEventListener('mouseleave', handleCanvasLeave)
})

onBeforeUnmount(() => {
	canvasRef.value?.removeEventListener('mouseleave', handleCanvasLeave)
	chartInstance?.destroy()
	chartInstance = null
})

watch(
	() => [props.type, props.fill, props.stacked],
	() => {
		chartInstance?.destroy()
		chartInstance = null
		nextTick(() => createChart())
	},
)

watch(
	() => [props.datasets, props.labels, props.activeStat],
	() => {
		refreshChart()
	},
	{ deep: true },
)
</script>
