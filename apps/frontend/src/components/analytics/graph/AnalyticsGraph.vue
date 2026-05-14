<template>
	<section
		class="relative flex flex-col !overflow-hidden rounded-2xl border border-solid border-surface-5 bg-surface-3"
	>
		<AnalyticsLoadingBar :loading="isDataLoading" />
		<div class="flex w-full flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
			<div
				class="flex w-full items-center justify-between rounded-t-2xl border-0 border-b border-solid border-surface-5 bg-surface-3 p-4"
			>
				<div class="text-xl font-semibold text-contrast">
					{{ graphTitle }}
				</div>

				<div class="flex items-center gap-3">
					<div v-if="hasChartEvents" class="inline-flex items-center gap-2">
						<label for="events-toggle" class="cursor-pointer text-sm text-secondary">Events</label>
						<Toggle id="events-toggle" v-model="showChartEvents" small />
					</div>
					<div v-if="canUseRatioMode" class="inline-flex items-center gap-2">
						<label for="ratio-mode-toggle" class="cursor-pointer text-sm text-secondary"
							>Ratio</label
						>
						<Toggle id="ratio-mode-toggle" v-model="isRatioMode" small />
					</div>
					<Tabs
						:value="activeViewMode"
						:tabs="viewModeTabs"
						@update:value="activeViewMode = $event as ViewMode"
					/>
				</div>
			</div>
		</div>

		<div
			class="flex flex-col gap-6 px-4 pb-6 pt-5"
			:class="['transition-opacity', isDataLoading ? 'pointer-events-none opacity-75' : '']"
		>
			<div class="flex flex-wrap items-center gap-x-4 gap-y-2 px-3">
				<div
					v-for="legendEntry in displayedLegendEntries"
					:key="legendEntry.id"
					class="inline-flex items-center"
				>
					<button
						type="button"
						class="inline-flex items-center gap-1.5 text-sm !outline-0 transition-all focus-within:!outline-0 focus:!outline-0 focus-visible:!outline-0"
						:class="[
							legendEntry.hidden ? 'text-secondary opacity-70' : 'text-primary',
							isLegendEntryToggleDisabled(legendEntry)
								? 'cursor-default'
								: 'cursor-pointer hover:brightness-125',
						]"
						:aria-pressed="!legendEntry.hidden"
						@click="onLegendEntryClick($event, legendEntry.id)"
					>
						<span class="size-2 rounded-full" :style="{ backgroundColor: legendEntry.color }" />
						<span
							v-tooltip="legendEntry.projectName ?? ''"
							:class="{ 'line-through': legendEntry.hidden }"
						>
							{{ legendEntry.name }}
						</span>
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

			<div ref="chartContainer" class="relative h-[460px]" @click="onChartClick">
				<div :class="['h-full']">
					<div
						v-if="selectedProjects.length === 0"
						class="flex h-full items-center justify-center rounded-xl"
					>
						<div
							v-if="!isDataLoading"
							class="relative bottom-6 text-base font-normal text-secondary"
						>
							{{ emptyChartMessage }}
						</div>
					</div>
					<template v-else>
						<ClientOnly>
							<AnalyticsChart
								:type="chartType"
								:fill="isArea"
								:stacked="isStacked"
								:ratio-mode="isRatioMode"
								:datasets="visibleChartDatasets"
								:labels="chartLabels"
								:x-axis-tick-limit="xAxisTickLimit"
								:active-stat="activeStat"
								:pinned-slice-index="pinnedSliceIndex"
								@hover="onChartHover"
								@geometry="onChartGeometry"
								@pinned-drag="onPinnedDrag"
								@range-select="onRangeSelect"
							/>
						</ClientOnly>
						<AnalyticsChartEvents
							v-if="showChartEvents"
							:events="dummyAnalyticsChartEvents"
							:active-stat="activeStat"
							:group-by="selectedGroupBy"
							:chart-start="chartRangeBounds?.start ?? null"
							:chart-end="chartRangeBounds?.end ?? null"
							:geometry="chartGeometry"
							:container-width="containerSize.width"
							:container-height="containerSize.height"
						/>
						<div
							v-if="showHoverGuide"
							aria-hidden="true"
							class="pointer-events-none absolute bottom-0 left-0 top-0 z-10 mb-8 mt-2 border-0 border-l border-solid border-contrast opacity-25"
							:style="{ transform: `translate(${hoverState.x}px, 0)` }"
						/>
						<div
							v-if="showPinnedGuide"
							aria-hidden="true"
							class="pointer-events-none absolute bottom-0 left-0 top-0 z-10 mb-8 mt-2 border-0 border-l-[2px] border-dashed border-green opacity-75"
							:style="{ transform: `translate(${hoverState.x}px, 0)` }"
						/>
						<AnalyticsChartTooltip
							:visible="hoverState.visible"
							:x="hoverState.x"
							:y="hoverState.y"
							:start="hoverBucketRange?.start ?? null"
							:end="hoverBucketRange?.end ?? null"
							:chart-start="chartRangeBounds?.start ?? null"
							:chart-end="chartRangeBounds?.end ?? null"
							:formatted-total="hoverFormattedTotal"
							:entries="hoverEntries"
							:container-width="containerSize.width"
							:container-height="containerSize.height"
							:pinned="isHoverPinned"
							:ratio-mode="isRatioMode"
						/>
					</template>
				</div>
			</div>
		</div>
		<div v-if="isDataLoading" class="absolute inset-0 z-[19] overflow-hidden rounded-xl">
			<div class="absolute inset-0 bg-surface-3 opacity-50" />
			<div class="absolute inset-0 backdrop-blur-[3px]" />
			<div class="absolute inset-0 flex items-center justify-center">
				<div
					class="relative bottom-6 inline-flex items-center gap-2 text-lg font-semibold text-primary"
				>
					<span>Fetching results...</span>
				</div>
			</div>
		</div>
	</section>
</template>

<script setup lang="ts">
import { ChartAreaIcon, ChartColumnBigIcon, ChartSplineIcon } from '@modrinth/assets'
import { Tabs, type TabsTab, Toggle, useFormatNumber } from '@modrinth/ui'

import { isDarkTheme } from '~/plugins/theme/index.ts'
import type { AnalyticsDashboardStat } from '~/providers/analytics/analytics'
import {
	doesProjectStatusMatchFilters,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

import AnalyticsLoadingBar from '../AnalyticsLoadingBar.vue'
import {
	ensureMinimumTimeRange,
	getDefaultAnalyticsGroupByForDurationMinutes,
} from '../query-builder/timeframe-picker/timeframe'
import AnalyticsChart, {
	type AnalyticsChartGeometryPayload,
	type AnalyticsChartRangeSelectPayload,
} from './AnalyticsChart.client.vue'
import AnalyticsChartEvents, { type AnalyticsChartEvent } from './AnalyticsChartEvents.vue'
import AnalyticsChartTooltip, { type AnalyticsChartTooltipEntry } from './AnalyticsChartTooltip.vue'
import {
	buildChartDatasets,
	buildTimeAxisLabels,
	type ChartDataset,
	formatMetricValue,
	getShortHourlyAxisTickLimit,
	getSliceBucketRange,
	getSliceCount,
} from './utils'

type ViewMode = 'line' | 'area' | 'bar'

const {
	activeStat,
	hasProjectContext,
	selectedTimeframeMode,
	selectedCustomTimeframeStartDate,
	selectedCustomTimeframeEndDate,
	selectedGroupBy: selectedDashboardGroupBy,
	displayedSelectedProjectIds: selectedProjectIds,
	projects,
	displayedFetchRequest: fetchRequest,
	displayedTimeSlices: timeSlices,
	displayedSelectedGroupBy: selectedGroupBy,
	displayedSelectedBreakdown: selectedBreakdown,
	displayedSelectedFilters: selectedFilters,
	isLoading,
	getVersionDisplayName,
	getVersionProjectName,
} = injectAnalyticsDashboardContext()
const formatNumber = useFormatNumber()
const isDataLoading = computed(() => isLoading.value)

const activeViewMode = ref<ViewMode>('line')
const isRatioMode = ref(false)
const showChartEvents = ref(true)

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

const dummyAnalyticsChartEvents: AnalyticsChartEvent[] = [
	{
		title: 'Analytics outage',
		startDate: '2026-04-25T00:00:00.000Z',
		endDate: '2026-04-27T00:00:00.000Z',
	},
	{
		title: 'Ad revenue over reported, resulting in a potential spike.',
		announcementUrl: 'https://modrinth.com/news',
		startDate: '2026-05-04T00:00:00.000Z',
		endDate: '2026-05-04T00:00:00.000Z',
		forMetricType: 'revenue',
	},
	{
		title: 'China CDN ingest outage',
		announcementUrl: 'https://modrinth.com/news',
		startDate: '2026-05-01T00:00:00.000Z',
		endDate: '2026-05-07T00:00:00.000Z',
		forMetricType: 'downloads',
	},
	{
		title: 'Modrinth App release',
		announcementUrl: 'https://modrinth.com/news',
		startDate: '2023-08-07T00:00:00.000Z',
		endDate: '2023-08-07T00:00:00.000Z',
	},
]

const hasChartEvents = computed(() => dummyAnalyticsChartEvents.length > 0)
const selectedProjectIdSet = computed(() => new Set(selectedProjectIds.value))
const hasAvailableProjects = computed(() => projects.value.length > 0)

const selectedProjects = computed(() =>
	projects.value.filter(
		(project) =>
			selectedProjectIdSet.value.has(project.id) &&
			doesProjectStatusMatchFilters(project.status, selectedFilters.value),
	),
)
const showProjectVersionNames = computed(
	() => selectedBreakdown.value === 'version_id' && selectedProjects.value.length > 1,
)

const emptyChartMessage = computed(() => {
	if (hasProjectContext.value) {
		return 'No data available for analytics'
	}

	return hasAvailableProjects.value
		? 'Select at least one project to view data'
		: 'No projects available for analytics'
})

const lightLegendPalette = [
	'#00AF5C',
	'#D55E00',
	'#0072B2',
	'#CC79A7',
	'#E69F00',
	'#332288',
	'#44AA99',
	'#882255',
	'#A04500',
	'#007F3A',
	'#8E4070',
	'#004A74',
	'#5C45D3',
	'#A37100',
	'#B83377',
	'#2F756A',
]

const darkLegendPalette = [
	'#1BD96A',
	'#FFB000',
	'#56B4E9',
	'#E78AC3',
	'#F0E442',
	'#A78BFA',
	'#FF7A59',
	'#4DD0C8',
	'#B87C00',
	'#119547',
	'#B05690',
	'#2D7AB5',
	'#6F58CC',
	'#B8AD30',
	'#2A938B',
	'#CC5239',
]

const theme = useTheme()
const legendPalette = computed(() =>
	isDarkTheme(theme.active) ? darkLegendPalette : lightLegendPalette,
)

const graphTitle = computed(() => titleByStat[activeStat.value])

const chartType = computed<'line' | 'bar'>(() => (activeViewMode.value === 'bar' ? 'bar' : 'line'))
const canUseRatioMode = computed(
	() =>
		(activeViewMode.value === 'area' || activeViewMode.value === 'bar') &&
		displayedLegendEntries.value.length > 1,
)
const isArea = computed(() => activeViewMode.value === 'area')
const isStacked = computed(
	() => isRatioMode.value || activeViewMode.value === 'area' || activeViewMode.value === 'bar',
)

watch(canUseRatioMode, (canUse) => {
	if (!canUse) {
		isRatioMode.value = false
	}
})

const sliceCount = computed(() => {
	const nextFetchRequest = fetchRequest.value
	const fallback = timeSlices.value.length
	if (!nextFetchRequest) return Math.max(1, fallback)
	return getSliceCount(nextFetchRequest.time_range, fallback)
})

const chartLabels = computed(() => {
	const nextFetchRequest = fetchRequest.value
	if (!nextFetchRequest) return []
	return buildTimeAxisLabels(nextFetchRequest.time_range, sliceCount.value, selectedGroupBy.value)
})

const xAxisTickLimit = computed(() => {
	const nextFetchRequest = fetchRequest.value
	return nextFetchRequest
		? getShortHourlyAxisTickLimit(nextFetchRequest.time_range, selectedGroupBy.value)
		: undefined
})

const allChartDatasets = computed(() =>
	buildChartDatasets(
		timeSlices.value,
		selectedProjects.value,
		activeStat.value,
		legendPalette.value,
		selectedBreakdown.value,
		selectedFilters.value,
		getVersionDisplayName,
		showProjectVersionNames.value ? getVersionProjectName : undefined,
		sliceCount.value,
	),
)

const chartContainer = ref<HTMLElement | null>(null)
const chartGeometry = ref<AnalyticsChartGeometryPayload | null>(null)
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
const LEGEND_EXPANDED_MAX_ITEMS = 24
const OTHER_LEGEND_ENTRY_ID = '__analytics_other__'
const OTHER_LEGEND_ENTRY_COLOR = '#9ca3af'

type LegendEntry = {
	id: string
	name: string
	projectName?: string
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

function ignoreUpcomingChartClick() {
	ignoreNextChartClick.value = true
	if (clearIgnoredChartClickTimeout) {
		clearTimeout(clearIgnoredChartClickTimeout)
	}
	clearIgnoredChartClickTimeout = setTimeout(() => {
		ignoreNextChartClick.value = false
		clearIgnoredChartClickTimeout = null
	}, 350)
}

function onPinnedDrag(payload: HoverState) {
	if (isDataLoading.value || !isHoverPinned.value) return
	ignoreUpcomingChartClick()
	setHoverState(payload)
}

function onChartGeometry(payload: AnalyticsChartGeometryPayload) {
	chartGeometry.value = payload
}

function getDefaultGroupByForRange(start: Date, end: Date) {
	const ensuredRange = ensureMinimumTimeRange(start, end)
	const durationMinutes = Math.max(
		1,
		Math.floor((ensuredRange.end.getTime() - ensuredRange.start.getTime()) / 60000),
	)

	return getDefaultAnalyticsGroupByForDurationMinutes(durationMinutes)
}

function onRangeSelect(payload: AnalyticsChartRangeSelectPayload) {
	if (isDataLoading.value) return

	const nextFetchRequest = fetchRequest.value
	if (!nextFetchRequest) return

	if (payload.startSliceIndex === payload.endSliceIndex) {
		ignoreUpcomingChartClick()
		return
	}

	const startSliceIndex = Math.min(payload.startSliceIndex, payload.endSliceIndex)
	const endSliceIndex = Math.max(payload.startSliceIndex, payload.endSliceIndex)
	const startBucketRange = getSliceBucketRange(
		nextFetchRequest.time_range,
		sliceCount.value,
		startSliceIndex,
	)
	const endBucketRange = getSliceBucketRange(
		nextFetchRequest.time_range,
		sliceCount.value,
		endSliceIndex,
	)
	const start = startBucketRange.start
	const end = endBucketRange.end

	if (
		!Number.isFinite(start.getTime()) ||
		!Number.isFinite(end.getTime()) ||
		end.getTime() <= start.getTime()
	) {
		return
	}

	ignoreUpcomingChartClick()
	isHoverPinned.value = false
	clearHoverState()
	selectedTimeframeMode.value = 'custom_datetime_range'
	selectedCustomTimeframeStartDate.value = start.toISOString()
	selectedCustomTimeframeEndDate.value = end.toISOString()
	selectedDashboardGroupBy.value = getDefaultGroupByForRange(start, end)
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
const showHoverGuide = computed(
	() =>
		!isDataLoading.value &&
		!isHoverPinned.value &&
		hoverState.visible &&
		hoverState.sliceIndex !== null,
)
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
				projectName: dataset.projectName,
				color: dataset.borderColor,
				totalValue,
				hidden: hiddenDatasetIds.value.has(dataset.projectId),
			}
		})
		.sort((a, b) => b.totalValue - a.totalValue || a.name.localeCompare(b.name))
		.map((entry, index) => ({
			...entry,
			color:
				selectedBreakdown.value === 'loader'
					? entry.color
					: legendPalette.value[index % legendPalette.value.length],
		})),
)

const otherBundledLegendIds = computed<string[]>(() => {
	if (!showAllLegendEntries.value) return []
	if (legendEntries.value.length <= LEGEND_EXPANDED_MAX_ITEMS) return []
	return legendEntries.value.slice(LEGEND_EXPANDED_MAX_ITEMS - 1).map((entry) => entry.id)
})

const otherLegendEntry = computed<LegendEntry | null>(() => {
	const bundledIds = otherBundledLegendIds.value
	if (bundledIds.length === 0) return null
	const idSet = new Set(bundledIds)
	const totalValue = legendEntries.value
		.filter((entry) => idSet.has(entry.id))
		.reduce((sum, entry) => sum + entry.totalValue, 0)
	return {
		id: OTHER_LEGEND_ENTRY_ID,
		name: 'Other',
		color: OTHER_LEGEND_ENTRY_COLOR,
		totalValue,
		hidden: hiddenDatasetIds.value.has(OTHER_LEGEND_ENTRY_ID),
	}
})

const displayedLegendEntries = computed<LegendEntry[]>(() => {
	if (!showAllLegendEntries.value) {
		return legendEntries.value.slice(0, LEGEND_MAX_ITEMS)
	}
	const other = otherLegendEntry.value
	if (!other) return legendEntries.value
	return [...legendEntries.value.slice(0, LEGEND_EXPANDED_MAX_ITEMS - 1), other]
})

const canShowMoreLegendEntries = computed(
	() => !showAllLegendEntries.value && legendEntries.value.length > LEGEND_MAX_ITEMS,
)
const canShowLessLegendEntries = computed(
	() => showAllLegendEntries.value && legendEntries.value.length > LEGEND_MAX_ITEMS,
)

const otherChartDataset = computed<ChartDataset | null>(() => {
	const bundledIds = otherBundledLegendIds.value
	if (bundledIds.length === 0) return null
	const idSet = new Set(bundledIds)
	const datasets = allChartDatasets.value.filter((dataset) => idSet.has(dataset.projectId))
	if (datasets.length === 0) return null

	const sliceLength = datasets[0].data.length
	const data = new Array<number>(sliceLength).fill(0)
	for (const dataset of datasets) {
		for (let i = 0; i < sliceLength; i++) {
			data[i] += dataset.data[i] ?? 0
		}
	}

	return {
		...datasets[0],
		projectId: OTHER_LEGEND_ENTRY_ID,
		label: 'Other',
		projectName: undefined,
		data,
		borderColor: OTHER_LEGEND_ENTRY_COLOR,
		backgroundColor: OTHER_LEGEND_ENTRY_COLOR,
	}
})

const chartDatasetById = computed(() => {
	const datasets = new Map<string, ChartDataset>()
	for (const dataset of allChartDatasets.value) {
		datasets.set(dataset.projectId, dataset)
	}
	if (otherChartDataset.value) {
		datasets.set(OTHER_LEGEND_ENTRY_ID, otherChartDataset.value)
	}
	return datasets
})

const baseVisibleChartDatasets = computed(() =>
	displayedLegendEntries.value
		.filter((legendEntry) => !legendEntry.hidden)
		.map((legendEntry) => {
			const dataset = chartDatasetById.value.get(legendEntry.id)
			if (!dataset) return null

			return {
				...dataset,
				borderColor: legendEntry.color,
				backgroundColor: legendEntry.color,
			}
		})
		.filter((dataset): dataset is ChartDataset => Boolean(dataset)),
)

const visibleChartDatasets = computed<ChartDataset[]>(() => {
	const datasets = baseVisibleChartDatasets.value
	if (!isRatioMode.value || datasets.length === 0) return datasets

	const sliceLength = datasets[0]?.data.length ?? 0
	const totals = new Array<number>(sliceLength).fill(0)
	for (const dataset of datasets) {
		for (let i = 0; i < sliceLength; i++) {
			totals[i] += dataset.data[i] ?? 0
		}
	}

	return datasets.map((dataset) => ({
		...dataset,
		data: dataset.data.map((value, i) => (totals[i] === 0 ? 0 : (value / totals[i]) * 100)),
	}))
})

const visibleChartDatasetById = computed(() => {
	const datasets = new Map<string, ChartDataset>()
	for (const dataset of visibleChartDatasets.value) {
		datasets.set(dataset.projectId, dataset)
	}
	return datasets
})

function isLegendEntryToggleDisabled(legendEntry: LegendEntry) {
	if (legendEntry.hidden) return false
	const visibleCount = displayedLegendEntries.value.filter((entry) => !entry.hidden).length
	return visibleCount <= 1
}

function toggleLegendEntryVisibility(datasetId: string) {
	const nextHiddenDatasetIds = new Set(hiddenDatasetIds.value)
	if (nextHiddenDatasetIds.has(datasetId)) {
		nextHiddenDatasetIds.delete(datasetId)
	} else {
		const visibleCount = displayedLegendEntries.value.filter((entry) => !entry.hidden).length
		if (visibleCount <= 1) return
		nextHiddenDatasetIds.add(datasetId)
	}
	hiddenDatasetIds.value = nextHiddenDatasetIds
}

function soloLegendEntry(datasetId: string) {
	const otherIds = displayedLegendEntries.value
		.map((entry) => entry.id)
		.filter((id) => id !== datasetId)
	const isAlreadySolo =
		!hiddenDatasetIds.value.has(datasetId) && otherIds.every((id) => hiddenDatasetIds.value.has(id))
	hiddenDatasetIds.value = isAlreadySolo ? new Set() : new Set(otherIds)
}

function onLegendEntryClick(event: MouseEvent, datasetId: string) {
	if (event.shiftKey) {
		soloLegendEntry(datasetId)
		return
	}
	toggleLegendEntryVisibility(datasetId)
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
	availableDatasetIds.add(OTHER_LEGEND_ENTRY_ID)
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

const chartRangeBounds = computed(() => {
	const nextFetchRequest = fetchRequest.value
	if (!nextFetchRequest) return null
	return {
		start: new Date(nextFetchRequest.time_range.start),
		end: new Date(nextFetchRequest.time_range.end),
	}
})

const hoverTotalValue = computed(() => {
	if (hoverState.sliceIndex === null) return 0
	const sliceIndex = hoverState.sliceIndex
	return visibleChartDatasets.value.reduce(
		(sum, dataset) => sum + (dataset.data[sliceIndex] ?? 0),
		0,
	)
})

const hoverFormattedTotal = computed(() => {
	if (isRatioMode.value) {
		return hoverTotalValue.value > 0 ? '100%' : '0%'
	}
	return formatMetricValue(hoverTotalValue.value, activeStat.value, formatNumber)
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
				projectName: legendEntry.projectName,
				color: legendEntry.color,
				formattedValue: isRatioMode.value
					? `${value.toFixed(1)}%`
					: formatMetricValue(value, activeStat.value, formatNumber),
			}
		})
})
</script>
