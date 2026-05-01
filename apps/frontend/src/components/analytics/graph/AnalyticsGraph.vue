<template>
	<section
		class="flex flex-col gap-4 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4"
	>
		<div class="flex flex-col gap-3 xl:flex-row xl:items-start xl:justify-between">
			<div class="flex w-full flex-col gap-3">
				<div class="flex items-center justify-between">
					<div class="text-xl font-semibold text-contrast">
						{{ graphTitle }}
					</div>

					<div class="flex items-center gap-2">
						<Tabs
							:value="activeViewMode"
							:tabs="viewModeTabs"
							@update:value="activeViewMode = $event as ViewMode"
						/>
					</div>
				</div>

				<div class="flex flex-wrap items-center gap-x-4 gap-y-2">
					<div
						v-for="legendEntry in displayedLegendEntries"
						:key="legendEntry.id"
						class="inline-flex items-center"
					>
						<button
							type="button"
							class="inline-flex cursor-pointer items-center gap-1.5 text-sm transition-all hover:brightness-125"
							:class="legendEntry.hidden ? 'text-secondary opacity-70' : 'text-primary'"
							:aria-pressed="!legendEntry.hidden"
							@click="toggleLegendEntryVisibility(legendEntry.id)"
						>
							<span class="size-2 rounded-full" :style="{ backgroundColor: legendEntry.color }" />
							<span :class="{ 'line-through': legendEntry.hidden }">{{ legendEntry.name }}</span>
						</button>
					</div>

					<button
						v-if="canShowMoreLegendEntries"
						type="button"
						class="text-sm font-normal text-primary underline transition-all hover:brightness-125"
						@click="showMoreLegendEntries"
					>
						Show more
					</button>
					<button
						v-else-if="canShowLessLegendEntries"
						type="button"
						class="text-sm font-normal text-primary underline transition-all hover:brightness-125"
						@click="showLessLegendEntries"
					>
						Show less
					</button>
				</div>
			</div>
		</div>

		<div ref="chartContainer" class="relative h-96" @click="onChartClick">
			<div
				v-if="isDataLoading"
				class="flex h-full items-center justify-center rounded-xl bg-surface-3"
			>
				<div
					class="relative bottom-6 inline-flex items-center gap-2 text-sm font-medium text-secondary"
				>
					<SpinnerIcon class="size-5 animate-spin" />
					<span>Loading data</span>
				</div>
			</div>
			<div
				v-else-if="selectedProjects.length === 0"
				class="flex h-full items-center justify-center rounded-xl"
			>
				<div class="relative bottom-6 text-sm font-medium text-secondary">
					Select at least one project to view data
				</div>
			</div>
			<template v-else>
				<ClientOnly>
					<AnalyticsChart
						:type="chartType"
						:fill="isArea"
						:stacked="isStacked"
						:datasets="visibleChartDatasets"
						:labels="chartLabels"
						:active-stat="activeStat"
						:pinned-slice-index="pinnedSliceIndex"
						@hover="onChartHover"
						@pinned-drag="onPinnedDrag"
					/>
				</ClientOnly>
				<div
					v-if="showPinnedGuide"
					aria-hidden="true"
					class="pointer-events-none absolute bottom-0 top-0 z-10 mb-8 mt-2 border-0 border-l-[2px] border-dashed border-green opacity-75"
					:style="{ transform: `translate(${hoverState.x}px, 0)` }"
				/>
				<AnalyticsChartTooltip
					:visible="hoverState.visible"
					:x="hoverState.x"
					:y="hoverState.y"
					:range-label="hoverRangeLabel"
					:entries="hoverEntries"
					:container-width="containerSize.width"
					:container-height="containerSize.height"
					:pinned="isHoverPinned"
				/>
			</template>
		</div>
	</section>
</template>

<script setup lang="ts">
import { ChartAreaIcon, ChartColumnBigIcon, ChartSplineIcon, SpinnerIcon } from '@modrinth/assets'
import { Tabs, type TabsTab, useFormatNumber } from '@modrinth/ui'

import type { AnalyticsDashboardStat } from '~/providers/analytics/analytics'
import { injectAnalyticsDashboardContext } from '~/providers/analytics/analytics'

import AnalyticsChart from './AnalyticsChart.client.vue'
import AnalyticsChartTooltip, { type AnalyticsChartTooltipEntry } from './AnalyticsChartTooltip.vue'
import {
	buildChartDatasets,
	buildTimeAxisLabels,
	type ChartDataset,
	formatBucketEndLabel,
	formatMetricValue,
	getSliceBucketRange,
	getSliceCount,
	isTimeRelevantForGroupBy,
} from './utils'

type ViewMode = 'line' | 'area' | 'bar'

const {
	activeStat,
	selectedProjectIds,
	projects,
	fetchRequest,
	timeSlices,
	selectedGroupBy,
	selectedBreakdown,
	selectedFilters,
	isLoading,
	getVersionDisplayName,
} = injectAnalyticsDashboardContext()
const formatNumber = useFormatNumber()
const isDataLoading = computed(() => isLoading.value)

const activeViewMode = ref<ViewMode>('line')

const viewModeTabs: TabsTab[] = [
	{ value: 'line', label: 'Line', icon: ChartSplineIcon },
	{ value: 'area', label: 'Area', icon: ChartAreaIcon },
	{ value: 'bar', label: 'Bar', icon: ChartColumnBigIcon },
]

const titleByStat: Record<AnalyticsDashboardStat, string> = {
	views: 'Views Over Time',
	downloads: 'Downloads Over Time',
	revenue: 'Revenue Over Time',
	playtime: 'Playtime Over Time',
}

const selectedProjectIdSet = computed(() => new Set(selectedProjectIds.value))

const selectedProjects = computed(() =>
	projects.value.filter((project) => selectedProjectIdSet.value.has(project.id)),
)

const legendPalette = ['#00D084', '#A78BFA', '#F59E0B', '#38BDF8', '#FB7185', '#34D399']

const graphTitle = computed(() => titleByStat[activeStat.value])

const chartType = computed<'line' | 'bar'>(() => (activeViewMode.value === 'bar' ? 'bar' : 'line'))
const isArea = computed(() => activeViewMode.value === 'area')
const isStacked = computed(() => activeViewMode.value === 'area' || activeViewMode.value === 'bar')

const sliceCount = computed(() => {
	const nextFetchRequest = fetchRequest.value
	const fallback = timeSlices.value.length
	if (!nextFetchRequest) return Math.max(1, fallback)
	return getSliceCount(nextFetchRequest.time_range, fallback)
})

const showTimeInBucketLabel = computed(() => isTimeRelevantForGroupBy(selectedGroupBy.value))

const chartLabels = computed(() => {
	const nextFetchRequest = fetchRequest.value
	if (!nextFetchRequest) return []
	return buildTimeAxisLabels(
		nextFetchRequest.time_range,
		sliceCount.value,
		selectedGroupBy.value,
	)
})

const allChartDatasets = computed(() =>
	buildChartDatasets(
		timeSlices.value,
		selectedProjects.value,
		activeStat.value,
		legendPalette,
		selectedBreakdown.value,
		selectedFilters.value,
		getVersionDisplayName,
	),
)

const chartContainer = ref<HTMLElement | null>(null)
const containerSize = reactive({ width: 0, height: 0 })
let resizeObserver: ResizeObserver | null = null
let clearIgnoredChartClickTimeout: ReturnType<typeof setTimeout> | null = null

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
	if (clearIgnoredChartClickTimeout) {
		clearTimeout(clearIgnoredChartClickTimeout)
		clearIgnoredChartClickTimeout = null
	}
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

const isHoverPinned = ref(false)
const ignoreNextChartClick = ref(false)
const hiddenDatasetIds = ref<Set<string>>(new Set())
const showAllLegendEntries = ref(false)

const LEGEND_MAX_ITEMS = 8

type LegendEntry = {
	id: string
	name: string
	color: string
	totalValue: number
	hidden: boolean
}

function setHoverState(payload: HoverState) {
	hoverState.visible = payload.visible
	hoverState.x = payload.x
	hoverState.y = payload.y
	hoverState.sliceIndex = payload.sliceIndex
}

function clearHoverState() {
	hoverState.visible = false
	hoverState.sliceIndex = null
}

function onChartHover(payload: HoverState) {
	if (isDataLoading.value) return
	if (isHoverPinned.value) return
	setHoverState(payload)
}

function onPinnedDrag(payload: HoverState) {
	if (isDataLoading.value || !isHoverPinned.value) return
	ignoreNextChartClick.value = true
	if (clearIgnoredChartClickTimeout) {
		clearTimeout(clearIgnoredChartClickTimeout)
	}
	clearIgnoredChartClickTimeout = setTimeout(() => {
		ignoreNextChartClick.value = false
		clearIgnoredChartClickTimeout = null
	}, 350)
	setHoverState(payload)
}

function onChartClick() {
	if (isDataLoading.value) return
	if (ignoreNextChartClick.value) {
		ignoreNextChartClick.value = false
		return
	}

	if (!hoverState.visible || hoverState.sliceIndex === null) {
		if (isHoverPinned.value) {
			isHoverPinned.value = false
			clearHoverState()
		}
		return
	}

	if (isHoverPinned.value) {
		isHoverPinned.value = false
		clearHoverState()
		return
	}

	isHoverPinned.value = true
}

const pinnedSliceIndex = computed(() => (isHoverPinned.value ? hoverState.sliceIndex : null))
const showPinnedGuide = computed(
	() =>
		!isDataLoading.value &&
		isHoverPinned.value &&
		hoverState.visible &&
		hoverState.sliceIndex !== null,
)

const legendEntries = computed<LegendEntry[]>(() =>
	allChartDatasets.value
		.map((dataset) => {
			const totalValue = dataset.data.reduce((sum, value) => sum + value, 0)

			return {
				id: dataset.projectId,
				name: dataset.label,
				color: dataset.borderColor,
				totalValue,
				hidden: hiddenDatasetIds.value.has(dataset.projectId),
			}
		})
		.sort((a, b) => b.totalValue - a.totalValue || a.name.localeCompare(b.name)),
)

const displayedLegendEntries = computed(() =>
	showAllLegendEntries.value ? legendEntries.value : legendEntries.value.slice(0, LEGEND_MAX_ITEMS),
)

const canShowMoreLegendEntries = computed(
	() => !showAllLegendEntries.value && legendEntries.value.length > LEGEND_MAX_ITEMS,
)
const canShowLessLegendEntries = computed(
	() => showAllLegendEntries.value && legendEntries.value.length > LEGEND_MAX_ITEMS,
)

const chartDatasetById = computed(() => {
	const datasets = new Map<string, ChartDataset>()
	for (const dataset of allChartDatasets.value) {
		datasets.set(dataset.projectId, dataset)
	}
	return datasets
})

const visibleChartDatasets = computed(() =>
	displayedLegendEntries.value
		.filter((legendEntry) => !legendEntry.hidden)
		.map((legendEntry) => chartDatasetById.value.get(legendEntry.id))
		.filter((dataset): dataset is ChartDataset => Boolean(dataset)),
)

const visibleChartDatasetById = computed(() => {
	const datasets = new Map<string, ChartDataset>()
	for (const dataset of visibleChartDatasets.value) {
		datasets.set(dataset.projectId, dataset)
	}
	return datasets
})

function toggleLegendEntryVisibility(datasetId: string) {
	const nextHiddenDatasetIds = new Set(hiddenDatasetIds.value)
	if (nextHiddenDatasetIds.has(datasetId)) {
		nextHiddenDatasetIds.delete(datasetId)
	} else {
		nextHiddenDatasetIds.add(datasetId)
	}
	hiddenDatasetIds.value = nextHiddenDatasetIds
}

function showMoreLegendEntries() {
	showAllLegendEntries.value = true
}

function showLessLegendEntries() {
	showAllLegendEntries.value = false
}

watch([chartLabels, allChartDatasets], () => {
	isHoverPinned.value = false
	clearHoverState()
})

watch(isDataLoading, (loading) => {
	if (!loading) return
	isHoverPinned.value = false
	clearHoverState()
})

watch(allChartDatasets, (datasets) => {
	const availableDatasetIds = new Set(datasets.map((dataset) => dataset.projectId))
	if (hiddenDatasetIds.value.size > 0) {
		const nextHiddenDatasetIds = new Set(
			Array.from(hiddenDatasetIds.value).filter((datasetId) => availableDatasetIds.has(datasetId)),
		)

		if (nextHiddenDatasetIds.size !== hiddenDatasetIds.value.size) {
			hiddenDatasetIds.value = nextHiddenDatasetIds
		}
	}

	if (datasets.length <= LEGEND_MAX_ITEMS) {
		showAllLegendEntries.value = false
	}
})

const hoverBucketRange = computed(() => {
	const nextFetchRequest = fetchRequest.value
	if (!nextFetchRequest || hoverState.sliceIndex === null) return null
	return getSliceBucketRange(nextFetchRequest.time_range, sliceCount.value, hoverState.sliceIndex)
})

const hoverRangeLabel = computed(() => {
	if (!hoverBucketRange.value) return ''
	return formatBucketEndLabel(hoverBucketRange.value.end, showTimeInBucketLabel.value)
})

const hoverEntries = computed<AnalyticsChartTooltipEntry[]>(() => {
	if (hoverState.sliceIndex === null) return []
	const sliceIndex = hoverState.sliceIndex
	return displayedLegendEntries.value
		.filter((legendEntry) => !legendEntry.hidden)
		.map((legendEntry) => {
			const dataset = visibleChartDatasetById.value.get(legendEntry.id)
			const value = dataset?.data[sliceIndex] ?? 0

			return {
				projectId: legendEntry.id,
				name: legendEntry.name,
				color: legendEntry.color,
				formattedValue: formatMetricValue(value, activeStat.value, formatNumber),
			}
		})
})
</script>
