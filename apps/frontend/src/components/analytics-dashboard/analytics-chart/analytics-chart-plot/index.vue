<template>
	<div
		ref="chartContainer"
		class="relative -ml-4 h-[460px] select-none"
		@click="onChartClick"
		@wheel.capture="onChartWheel"
	>
		<div :class="['h-full']">
			<div v-if="showEmptyChartState" class="flex h-full items-center justify-center rounded-xl">
				<div v-if="!isDataLoading" class="relative bottom-6 text-base font-normal text-secondary">
					{{ emptyChartMessage }}
				</div>
			</div>
			<template v-else>
				<ClientOnly>
					<AnalyticsChartClient
						:type="chartType"
						:fill="isArea"
						:stacked="isStacked"
						:ratio-mode="isRatioMode"
						:datasets="visibleChartDatasets"
						:labels="chartLabels"
						:x-axis-tick-limit="xAxisTickLimit"
						:active-stat="activeStat"
						:pinned-slice-index="pinnedSliceIndex"
						:highlighted-dataset-id="highlightedChartDatasetId"
						@hover="onChartHover"
						@geometry="onChartGeometry"
						@pinned-drag="onPinnedDrag"
						@range-select="onRangeSelect"
						@touch-drag="onTouchDragEnd"
					/>
				</ClientOnly>
				<AnalyticsChartEvents
					v-if="hasVisibleTimelineEvents"
					:events="visibleTimelineEvents"
					:active-stat="activeStat"
					:group-by="selectedGroupBy"
					:chart-start="chartRangeBounds?.start ?? null"
					:chart-end="chartRangeBounds?.end ?? null"
					:geometry="chartGeometry"
				/>
				<div
					v-if="showHoverGuide"
					aria-hidden="true"
					class="pointer-events-none absolute bottom-0 left-0 top-0 z-10 mb-[1.8rem] mt-2.5 border-0 border-l border-solid border-contrast opacity-25"
					:style="{ transform: `translate(${hoverState.x}px, 0)` }"
				/>
				<div
					v-if="showPinnedGuide"
					aria-hidden="true"
					class="pointer-events-none absolute bottom-0 left-0 top-0 z-10 mb-[1.8rem] mt-2.5 border-0 border-l border-dashed border-green opacity-75"
					:style="{ transform: `translate(${hoverState.x}px, 0)` }"
				/>
				<AnalyticsChartTooltip
					ref="chartTooltip"
					:visible="hoverState.visible"
					:x="hoverState.x"
					:y="hoverState.y"
					:start="hoverBucketRange?.start ?? null"
					:end="hoverBucketRange?.end ?? null"
					:previous-start="previousHoverBucketRange?.start ?? null"
					:previous-end="previousHoverBucketRange?.end ?? null"
					:chart-start="chartRangeBounds?.start ?? null"
					:chart-end="chartRangeBounds?.end ?? null"
					:formatted-total="hoverFormattedTotal"
					:entries="hoverEntries"
					:container-width="containerSize.width"
					:container-height="containerSize.height"
					:pinned="isHoverPinned"
					:ratio-mode="isRatioMode"
					:capitalize-labels="shouldCapitalizeDatasetLabels"
					:shift-key-pressed="isShiftKeyPressed"
					@entry-click="(datasetId, shiftKey) => emit('entry-click', datasetId, shiftKey)"
					@entry-hover="emit('entry-hover', $event)"
					@entry-hover-clear="emit('entry-hover-clear', $event)"
				/>
			</template>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { useFormatNumber, useVIntl } from '@modrinth/ui'

import type {
	AnalyticsDashboardStat,
	AnalyticsGroupByPreset,
} from '~/providers/analytics/analytics'

import type {
	AnalyticsChartLegendEntry,
	AnalyticsChartRangeBounds,
} from '../analytics-chart-types.ts'
import type { ChartDataset } from '../analytics-chart-utils.ts'
import { formatMetricValue } from '../analytics-chart-utils.ts'
import AnalyticsChartClient from '../AnalyticsChart.client.vue'
import AnalyticsChartEvents, { type AnalyticsChartEvent } from './AnalyticsChartEvents.vue'
import AnalyticsChartTooltip from './AnalyticsChartTooltip.vue'
import { useAnalyticsChartInteractions } from './use-analytics-chart-interactions.ts'

const props = defineProps<{
	chartType: 'line' | 'bar'
	isArea: boolean
	isStacked: boolean
	isRatioMode: boolean
	isDataLoading: boolean
	showEmptyChartState: boolean
	emptyChartMessage: string
	visibleChartDatasets: ChartDataset[]
	chartLabels: string[]
	xAxisTickLimit?: number
	activeStat: AnalyticsDashboardStat
	highlightedChartDatasetId: string | null
	hasVisibleTimelineEvents: boolean
	visibleTimelineEvents: AnalyticsChartEvent[]
	selectedGroupBy: AnalyticsGroupByPreset
	chartRangeBounds: AnalyticsChartRangeBounds | null
	fetchRequest: Labrinth.Analytics.v3.FetchRequest | null
	sliceCount: number
	shouldShowPreviousPeriod: boolean
	allChartDatasets: ChartDataset[]
	currentLegendEntries: AnalyticsChartLegendEntry[]
	legendEntries: AnalyticsChartLegendEntry[]
	chartDatasetById: Map<string, ChartDataset>
	hoverRatioSliceTotals: number[]
	shouldCapitalizeDatasetLabels: boolean
}>()

const emit = defineEmits<{
	'range-select': [start: Date, end: Date, groupBy: AnalyticsGroupByPreset]
	'entry-click': [datasetId: string, shiftKey: boolean]
	'entry-hover': [datasetId: string]
	'entry-hover-clear': [datasetId: string]
}>()

const formatNumber = useFormatNumber()
const { formatMessage } = useVIntl()
const {
	chartContainer,
	chartTooltip,
	chartGeometry,
	containerSize,
	hoverState,
	isHoverPinned,
	isShiftKeyPressed,
	onChartHover,
	onPinnedDrag,
	onTouchDragEnd,
	onChartGeometry,
	onRangeSelect,
	onChartClick,
	onChartWheel,
	pinnedSliceIndex,
	showHoverGuide,
	showPinnedGuide,
	hoverBucketRange,
	previousHoverBucketRange,
} = useAnalyticsChartInteractions({
	isDataLoading: computed(() => props.isDataLoading),
	fetchRequest: computed(() => props.fetchRequest),
	sliceCount: computed(() => props.sliceCount),
	chartLabels: computed(() => props.chartLabels),
	allChartDatasets: computed(() => props.allChartDatasets),
	chartRangeBounds: computed(() => props.chartRangeBounds),
	shouldShowPreviousPeriod: computed(() => props.shouldShowPreviousPeriod),
	onRangeSelected: (start, end, groupBy) => emit('range-select', start, end, groupBy),
})

function getTooltipTotalMetricValue(value: number): number {
	if (props.activeStat === 'revenue' && Math.abs(value) < 1) {
		return Math.round(value * 100) / 100
	}

	return value
}

const hoverTotalValue = computed(() => {
	if (hoverState.sliceIndex === null) return 0
	const sliceIndex = hoverState.sliceIndex
	if (props.isRatioMode) return props.hoverRatioSliceTotals[sliceIndex] ?? 0

	return props.currentLegendEntries.reduce((sum, legendEntry) => {
		if (legendEntry.hidden) return sum
		const dataset = props.chartDatasetById.get(legendEntry.id)
		return sum + getTooltipTotalMetricValue(dataset?.data[sliceIndex] ?? 0)
	}, 0)
})

const hoverFormattedTotal = computed(() => {
	if (props.isRatioMode) {
		return hoverTotalValue.value > 0 ? '100%' : '0%'
	}
	return formatMetricValue(hoverTotalValue.value, props.activeStat, formatNumber, formatMessage)
})

const hoverEntries = computed(() => {
	if (hoverState.sliceIndex === null) return []
	const sliceIndex = hoverState.sliceIndex
	const totalValue = hoverTotalValue.value

	return props.legendEntries.map((legendEntry) => {
		const dataset = props.chartDatasetById.get(legendEntry.id)
		const value = dataset?.data[sliceIndex] ?? 0
		const ratioValue = legendEntry.hidden || totalValue === 0 ? 0 : (value / totalValue) * 100
		return {
			projectId: legendEntry.id,
			name: legendEntry.name,
			projectName: legendEntry.projectName,
			color: legendEntry.color,
			formattedValue: props.isRatioMode
				? `${ratioValue.toFixed(1)}%`
				: formatMetricValue(value, props.activeStat, formatNumber, formatMessage),
			hidden: legendEntry.hidden,
			toggleDisabled: !legendEntry.hidden && isLegendEntryToggleDisabled(legendEntry),
			isPreviousPeriod: legendEntry.isPreviousPeriod,
		}
	})
})

function isLegendEntryToggleDisabled(legendEntry: AnalyticsChartLegendEntry) {
	if (legendEntry.hidden) return false
	const visibleCount = props.legendEntries.filter((entry) => !entry.hidden).length
	return visibleCount <= 1
}
</script>
