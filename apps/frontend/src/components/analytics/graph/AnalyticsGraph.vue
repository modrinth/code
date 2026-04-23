<template>
	<section
		class="flex flex-col gap-4 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4"
	>
		<div class="flex flex-col gap-3 xl:flex-row xl:items-start xl:justify-between">
			<div class="flex w-full flex-col gap-4">
				<div class="flex items-center justify-between">
					<div class="text-xl font-semibold text-contrast">
						{{ graphTitle }}
					</div>

					<div class="flex items-center gap-2">
						<div class="w-52">
							<Chips
								v-model="activeViewMode"
								:items="viewModeValues"
								:format-label="formatViewModeLabel"
								size="small"
								:capitalize="false"
								aria-label="Chart view mode"
							/>
						</div>
					</div>
				</div>

				<div class="flex flex-wrap items-center gap-x-4 gap-y-2">
					<div
						v-for="legendEntry in legendEntries"
						:key="legendEntry.id"
						class="inline-flex items-center gap-1.5 text-sm text-primary"
					>
						<span class="size-2 rounded-full" :style="{ backgroundColor: legendEntry.color }" />
						<span>{{ legendEntry.name }}</span>
					</div>

					<span v-if="legendEntries.length === 0" class="text-sm text-secondary">
						No projects selected
					</span>
				</div>
			</div>
		</div>

		<div ref="chartContainer" class="relative h-80">
			<div
				v-if="selectedProjects.length === 0"
				class="flex h-full items-center justify-center rounded-xl border border-dashed border-surface-5 bg-surface-3"
			>
				<div class="text-sm font-medium text-secondary">
					Select at least one project to view data
				</div>
			</div>
			<template v-else>
				<ClientOnly>
					<AnalyticsChart
						:type="chartType"
						:fill="isArea"
						:stacked="isStacked"
						:datasets="chartDatasets"
						:labels="chartLabels"
						:active-stat="analyticsDashboardContext.activeStat.value"
						@hover="onChartHover"
					/>
				</ClientOnly>
				<AnalyticsChartTooltip
					:visible="hoverState.visible"
					:x="hoverState.x"
					:y="hoverState.y"
					:range-label="hoverRangeLabel"
					:entries="hoverEntries"
					:container-width="containerSize.width"
					:container-height="containerSize.height"
				/>
			</template>
		</div>
	</section>
</template>

<script setup lang="ts">
import { Chips, useFormatNumber } from '@modrinth/ui'

import type { AnalyticsDashboardStat } from '~/providers/analytics/analytics'
import { injectAnalyticsDashboardContext } from '~/providers/analytics/analytics'

import AnalyticsChart from './AnalyticsChart.client.vue'
import AnalyticsChartTooltip, { type AnalyticsChartTooltipEntry } from './AnalyticsChartTooltip.vue'
import {
	buildChartDatasets,
	buildTimeAxisLabels,
	formatBucketEndLabel,
	formatMetricValue,
	getSliceBucketRange,
	getSliceCount,
	isTimeRelevantForGroupBy,
} from './utils'

type ViewMode = 'line' | 'area' | 'bar'

const analyticsDashboardContext = injectAnalyticsDashboardContext()
const formatNumber = useFormatNumber()

const activeViewMode = ref<ViewMode>('line')

const viewModeLabels: Record<ViewMode, string> = {
	line: 'Line',
	area: 'Area',
	bar: 'Bar',
}

const viewModeValues: ViewMode[] = ['line', 'area', 'bar']

function formatViewModeLabel(value: ViewMode): string {
	return viewModeLabels[value]
}

const titleByStat: Record<AnalyticsDashboardStat, string> = {
	views: 'Views Over Time',
	downloads: 'Downloads Over Time',
	revenue: 'Revenue Over Time',
	playtime: 'Playtime Over Time',
}

const selectedProjectIdSet = computed(
	() => new Set(analyticsDashboardContext.selectedProjectIds.value),
)

const selectedProjects = computed(() =>
	analyticsDashboardContext.projects.value.filter((project) =>
		selectedProjectIdSet.value.has(project.id),
	),
)

const legendPalette = ['#00D084', '#A78BFA', '#F59E0B', '#38BDF8', '#FB7185', '#34D399']

const graphTitle = computed(() => titleByStat[analyticsDashboardContext.activeStat.value])

const chartType = computed<'line' | 'bar'>(() => (activeViewMode.value === 'bar' ? 'bar' : 'line'))
const isArea = computed(() => activeViewMode.value === 'area')
const isStacked = computed(() => activeViewMode.value === 'area' || activeViewMode.value === 'bar')

const sliceCount = computed(() => {
	const fetchRequest = analyticsDashboardContext.fetchRequest.value
	const fallback = analyticsDashboardContext.timeSlices.value.length
	if (!fetchRequest) return Math.max(1, fallback)
	return getSliceCount(fetchRequest.time_range, fallback)
})

const showTimeInBucketLabel = computed(() =>
	isTimeRelevantForGroupBy(analyticsDashboardContext.selectedGroupBy.value),
)

const chartLabels = computed(() => {
	const fetchRequest = analyticsDashboardContext.fetchRequest.value
	if (!fetchRequest) return []
	return buildTimeAxisLabels(fetchRequest.time_range, sliceCount.value, showTimeInBucketLabel.value)
})

const chartDatasets = computed(() =>
	buildChartDatasets(
		analyticsDashboardContext.timeSlices.value,
		selectedProjects.value,
		analyticsDashboardContext.activeStat.value,
		legendPalette,
		analyticsDashboardContext.selectedBreakdown.value,
	),
)

const legendEntries = computed(() =>
	chartDatasets.value.map((dataset) => ({
		id: dataset.projectId,
		name: dataset.label,
		color: dataset.borderColor,
	})),
)

const chartContainer = ref<HTMLElement | null>(null)
const containerSize = reactive({ width: 0, height: 0 })
let resizeObserver: ResizeObserver | null = null

onMounted(() => {
	if (!chartContainer.value || typeof ResizeObserver === 'undefined') return
	resizeObserver = new ResizeObserver((entries) => {
		const entry = entries[0]
		if (!entry) return
		containerSize.width = entry.contentRect.width
		containerSize.height = entry.contentRect.height
	})
	resizeObserver.observe(chartContainer.value)
})

onBeforeUnmount(() => {
	resizeObserver?.disconnect()
	resizeObserver = null
})

type HoverState = {
	visible: boolean
	x: number
	y: number
	sliceIndex: number | null
}

const hoverState = reactive<HoverState>({
	visible: false,
	x: 0,
	y: 0,
	sliceIndex: null,
})

function onChartHover(payload: HoverState) {
	hoverState.visible = payload.visible
	hoverState.x = payload.x
	hoverState.y = payload.y
	hoverState.sliceIndex = payload.sliceIndex
}

watch([chartLabels, chartDatasets], () => {
	hoverState.visible = false
	hoverState.sliceIndex = null
})

const hoverBucketRange = computed(() => {
	const fetchRequest = analyticsDashboardContext.fetchRequest.value
	if (!fetchRequest || hoverState.sliceIndex === null) return null
	return getSliceBucketRange(fetchRequest.time_range, sliceCount.value, hoverState.sliceIndex)
})

const hoverRangeLabel = computed(() => {
	if (!hoverBucketRange.value) return ''
	return formatBucketEndLabel(hoverBucketRange.value.end, showTimeInBucketLabel.value)
})

const hoverEntries = computed<AnalyticsChartTooltipEntry[]>(() => {
	if (hoverState.sliceIndex === null) return []
	const sliceIndex = hoverState.sliceIndex
	return chartDatasets.value.map((dataset) => ({
		projectId: dataset.projectId,
		name: dataset.label,
		color: dataset.borderColor,
		formattedValue: formatMetricValue(
			dataset.data[sliceIndex] ?? 0,
			analyticsDashboardContext.activeStat.value,
			formatNumber,
		),
	}))
})
</script>
