<template>
	<NewModal
		ref="showAllSelectedGraphDatasetsModal"
		:header="`Show all ${tableProjectCount} lines in graph?`"
		fade="warning"
		width="500px"
		max-width="calc(100vw - 2rem)"
	>
		<p class="m-0 max-w-[32rem] text-primary">
			Showing all selected lines from table may degrade page performance.
		</p>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="transparent">
					<button @click="showAllSelectedGraphDatasetsModal?.hide()">Cancel</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button class="!shadow-none" @click="confirmShowAllSelectedGraphDatasets">
						Show all
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>

	<section
		class="relative flex flex-col rounded-2xl border border-solid border-surface-5 bg-surface-3"
	>
		<div class="flex w-full flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
			<div
				class="flex w-full flex-col items-start gap-3 rounded-t-2xl border-0 border-b border-solid border-surface-5 bg-surface-3 p-4 sm:flex-row sm:flex-wrap sm:items-center sm:justify-between"
			>
				<div class="flex min-w-0 flex-col gap-0.5">
					<div class="text-xl font-semibold text-contrast">
						{{ graphTitle }}
					</div>
					<div
						v-if="showTableSelectionSubheading"
						class="m-0 flex flex-wrap items-center gap-2 text-sm text-secondary"
					>
						<span>{{ tableSelectionSubheading }}</span>

						<button
							v-if="showGraphRenderLimitButton"
							type="button"
							class="font-base border-0 bg-transparent p-0 text-sm underline transition-all hover:brightness-125"
							@click="toggleGraphRenderLimit"
						>
							{{ graphRenderLimitButtonLabel }}
						</button>
						<button
							v-if="showTopGraphDatasetsButton"
							type="button"
							class="font-base border-0 bg-transparent p-0 text-sm underline transition-all hover:brightness-125"
							@click="showTopGraphDatasets"
						>
							Show top 8
						</button>
					</div>
				</div>

				<div class="flex w-full flex-wrap items-center gap-3 sm:w-auto sm:justify-end">
					<div v-if="canUseRatioMode" class="inline-flex items-center gap-2">
						<label for="ratio-mode-toggle" class="cursor-pointer text-sm text-secondary"
							>Ratio</label
						>
						<Toggle id="ratio-mode-toggle" v-model="isRatioMode" small />
					</div>
					<div v-if="canShowPreviousPeriodToggle" class="inline-flex items-center gap-2">
						<label for="previous-period-toggle" class="cursor-pointer text-sm text-secondary"
							>Prev. period</label
						>
						<Toggle id="previous-period-toggle" v-model="showPreviousPeriod" small />
					</div>
					<div v-if="hasChartEvents" class="inline-flex items-center gap-2">
						<label for="events-toggle" class="cursor-pointer text-sm text-secondary">Events</label>
						<Toggle id="events-toggle" v-model="showChartEvents" small />
					</div>
					<Tabs
						:value="activeGraphViewMode"
						:tabs="viewModeTabs"
						@update:value="activeGraphViewMode = $event as AnalyticsGraphViewMode"
					/>
				</div>
			</div>
		</div>

		<div
			class="flex flex-col gap-6 px-4 pb-6 pt-5"
			:class="['transition-opacity', isDataLoading ? 'pointer-events-none opacity-75' : '']"
		>
			<div class="relative">
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-5"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-5"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showLegendTopFade"
						class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-5 bg-gradient-to-b from-surface-3 to-transparent"
					/>
				</Transition>

				<div
					ref="legendContainer"
					class="flex max-h-[160px] flex-wrap items-center gap-y-1 overflow-y-auto px-3"
					@scroll="checkLegendScrollState"
				>
					<div
						v-for="legendEntry in displayedLegendEntries"
						:key="legendEntry.id"
						class="inline-flex items-center"
					>
						<button
							v-tooltip="getLegendEntryTooltip(legendEntry)"
							type="button"
							class="inline-flex items-center gap-1.5 px-2 py-0.5 text-sm !outline-0 transition-all focus-within:!outline-0 focus:!outline-0 focus-visible:!outline-0"
							:class="[
								legendEntry.hidden ? 'text-secondary opacity-70' : 'text-primary',
								isLegendEntryToggleDisabled(legendEntry) && !isShiftKeyPressed
									? 'cursor-default'
									: 'cursor-pointer hover:brightness-125',
							]"
							:aria-pressed="!legendEntry.hidden"
							@mouseenter="setHoveredLegendEntryId(legendEntry.id)"
							@mouseleave="clearHoveredLegendEntryId(legendEntry.id)"
							@focus="setHoveredLegendEntryId(legendEntry.id)"
							@blur="clearHoveredLegendEntryId(legendEntry.id)"
							@click="onLegendEntryClick($event, legendEntry.id)"
						>
							<span
								:class="
									legendEntry.isPreviousPeriod
										? 'h-0 w-2 rounded-none border-0 border-t-2 border-dashed bg-transparent'
										: 'size-2 rounded-full'
								"
								:style="
									legendEntry.isPreviousPeriod
										? { borderColor: legendEntry.color }
										: { backgroundColor: legendEntry.color }
								"
							/>
							<span
								:class="{
									'line-through': legendEntry.hidden,
									capitalize: shouldCapitalizeDatasetLabels,
								}"
							>
								{{ legendEntry.name }}
							</span>
						</button>
						<Dropdown
							v-if="isUnmonetizedLegendEntry(legendEntry)"
							theme="analytics-monetization-popover"
							:triggers="['hover', 'focus']"
							:popper-triggers="['hover', 'focus']"
							:delay="{ show: 0, hide: 250 }"
							placement="top"
							:aria-id="monetizationPopoverId"
							no-auto-focus
						>
							<InfoIcon
								class="-ml-1 mt-px inline-flex size-4 items-center justify-center rounded-full border-0 bg-transparent p-0 text-secondary transition-all hover:text-contrast focus-visible:text-contrast"
								aria-label="View monetized analytics details"
							/>
							<template #popper>
								<div
									role="dialog"
									aria-label="Monetized analytics details"
									class="font-base w-[292px] rounded-xl border border-solid border-surface-5 bg-surface-3 p-3 text-sm leading-snug shadow-2xl"
								>
									Only views and downloads made through Modrinth count toward monetization, and
									downloads require users to be logged in.
								</div>
							</template>
						</Dropdown>
					</div>
					<button
						v-if="canToggleLegendExpansion"
						type="button"
						class="font-base ml-1 text-sm text-secondary underline !transition-all hover:text-contrast"
						:aria-expanded="isLegendExpanded"
						@click="toggleLegendExpansion"
					>
						{{ isLegendExpanded ? 'Show less' : 'Show more' }}
					</button>
				</div>

				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-5"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-5"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showLegendBottomFade"
						class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-5 bg-gradient-to-t from-surface-3 to-transparent"
					/>
				</Transition>
			</div>

			<div
				ref="chartContainer"
				class="relative h-[460px]"
				@click="onChartClick"
				@wheel.capture="onChartWheel"
			>
				<div :class="['h-full']">
					<div
						v-if="showEmptyChartState"
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
								:highlighted-dataset-id="highlightedChartDatasetId"
								@hover="onChartHover"
								@geometry="onChartGeometry"
								@pinned-drag="onPinnedDrag"
								@range-select="onRangeSelect"
							/>
						</ClientOnly>
						<AnalyticsChartEvents
							v-if="showChartEvents"
							:events="localAnalyticsChartEvents"
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
							@entry-click="onTooltipEntryClick"
							@entry-hover="setHoveredLegendEntryId"
							@entry-hover-clear="clearHoveredLegendEntryId"
						/>
					</template>
				</div>
			</div>
		</div>
		<div v-if="isDataLoading" class="absolute inset-0 z-[19] overflow-hidden rounded-xl">
			<AnalyticsLoadingBar :loading="isDataLoading" />
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
import { ChartAreaIcon, ChartColumnBigIcon, ChartSplineIcon, InfoIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	injectModrinthClient,
	NewModal,
	Tabs,
	type TabsTab,
	Toggle,
	useFormatNumber,
	useScrollIndicator,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { Dropdown } from 'floating-vue'

import { isDarkTheme } from '~/plugins/theme/index.ts'
import type {
	AnalyticsDashboardStat,
	AnalyticsGraphViewMode,
} from '~/providers/analytics/analytics'
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
import AnalyticsChartEvents from './AnalyticsChartEvents.vue'
import AnalyticsChartTooltip, { type AnalyticsChartTooltipEntry } from './AnalyticsChartTooltip.vue'
import {
	buildChartDatasets,
	buildTimeAxisLabels,
	type ChartDataset,
	formatMetricValue,
	getShortHourlyAxisTickLimit,
	getSliceBucketRange,
	getSliceCount,
	shouldCapitalizeBreakdownLabel,
} from './utils'

const {
	activeStat,
	activeGraphViewMode,
	isRatioMode,
	showChartEvents,
	showPreviousPeriod,
	hiddenGraphDatasetIds,
	hasExplicitGraphDatasetSelection,
	isGraphDatasetSelectionActive,
	selectedGraphDatasetIds,
	hasPreviousPeriodComparison,
	hasProjectContext,
	selectedTimeframeMode,
	selectedCustomTimeframeStartDate,
	selectedCustomTimeframeEndDate,
	selectedGroupBy: selectedDashboardGroupBy,
	displayedSelectedProjectIds: selectedProjectIds,
	projects,
	displayedFetchRequest: fetchRequest,
	displayedTimeSlices: timeSlices,
	displayedPreviousTimeSlices: previousTimeSlices,
	displayedSelectedGroupBy: selectedGroupBy,
	displayedSelectedBreakdown: selectedBreakdown,
	displayedSelectedFilters: selectedFilters,
	isLoading,
	getVersionDisplayName,
	getVersionProjectName,
} = injectAnalyticsDashboardContext()
const client = injectModrinthClient()
const analyticsEventsQueryKey = ['analytics-events'] as const
const formatNumber = useFormatNumber()
const isDataLoading = computed(() => isLoading.value)
const { data: analyticsEvents } = useQuery({
	queryKey: analyticsEventsQueryKey,
	queryFn: () => client.labrinth.analytics_v3.getEvents(),
	placeholderData: [],
	refetchOnMount: 'always',
	retry: false,
	staleTime: 0,
})

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
const dashboardStats: readonly AnalyticsDashboardStat[] = [
	'views',
	'downloads',
	'revenue',
	'playtime',
]
const TOP_GRAPH_DATASET_LIMIT = 8
const GRAPH_RENDER_DATASET_LIMIT = 250
const PREVIOUS_PERIOD_DATASET_ID_PREFIX = 'previous-period:'
const PREVIOUS_PERIOD_BORDER_DASH = [6, 4]
const MONETIZATION_LEGEND_ENTRY_ORDER = new Map([
	['breakdown:monetized', 0],
	['breakdown:unmonetized', 1],
])

const localAnalyticsChartEvents = computed(() => analyticsEvents.value ?? [])
const hasChartEvents = computed(() => localAnalyticsChartEvents.value.length > 0)
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
const tableProjectCount = computed(() => selectedGraphDatasetIds.value.length)
const isTableGraphSelectionEmpty = computed(
	() =>
		isGraphDatasetSelectionActive.value &&
		hasExplicitGraphDatasetSelection.value &&
		tableProjectCount.value === 0,
)
const showEmptyChartState = computed(
	() => selectedProjects.value.length === 0 || isTableGraphSelectionEmpty.value,
)

const emptyChartMessage = computed(() => {
	if (isTableGraphSelectionEmpty.value) {
		return 'Select items from table below to visualize your data.'
	}

	if (hasProjectContext.value) {
		return 'No data available for analytics'
	}

	return hasAvailableProjects.value
		? 'Select at least one project to view data'
		: 'No projects available for analytics'
})

const lightLegendPalette = [
	'hsl(152, 100%, 34%)',
	'hsl(26, 100%, 42%)',
	'hsl(202, 100%, 35%)',
	'hsl(327, 45%, 64%)',
	'hsl(41, 100%, 45%)',
	'hsl(250, 60%, 33%)',
	'hsl(170, 43%, 47%)',
	'hsl(330, 60%, 33%)',
	'hsl(46, 100%, 36%)',
	'hsl(167, 100%, 30%)',
	'hsl(343, 38%, 45%)',
	'hsl(222, 100%, 28%)',
	'hsl(270, 62%, 60%)',
	'hsl(32, 100%, 37%)',
	'hsl(349, 57%, 51%)',
	'hsl(191, 43%, 37%)',
]

const darkLegendPalette = [
	'hsl(145, 78%, 48%)',
	'hsl(41, 100%, 50%)',
	'hsl(202, 77%, 63%)',
	'hsl(323, 66%, 72%)',
	'hsl(56, 85%, 60%)',
	'hsl(255, 92%, 80%)',
	'hsl(12, 100%, 67%)',
	'hsl(176, 58%, 56%)',
	'hsl(60, 100%, 41%)',
	'hsl(165, 80%, 38%)',
	'hsl(341, 36%, 56%)',
	'hsl(226, 60%, 49%)',
	'hsl(252, 53%, 62%)',
	'hsl(75, 59%, 50%)',
	'hsl(195, 56%, 42%)',
	'hsl(30, 59%, 56%)',
]

const theme = useTheme()
const legendPalette = computed(() =>
	isDarkTheme(theme.active) ? darkLegendPalette : lightLegendPalette,
)

const graphTitle = computed(() => titleByStat[activeStat.value])
const showTableSelectionSubheading = computed(
	() => isGraphDatasetSelectionActive.value && tableProjectCount.value > 0,
)
const tableBreakdownItemLabel = computed(() => {
	const isSingular = tableProjectCount.value === 1

	switch (selectedBreakdown.value) {
		case 'project':
			return isSingular ? 'project' : 'projects'
		case 'country':
			return isSingular ? 'country' : 'countries'
		case 'monetization':
			return isSingular ? 'monetization value' : 'monetization values'
		case 'user_agent':
			return isSingular ? 'download source' : 'download sources'
		case 'download_reason':
			return isSingular ? 'download reason' : 'download reasons'
		case 'version_id':
			return isSingular ? 'project version' : 'project versions'
		case 'loader':
			return isSingular ? 'loader' : 'loaders'
		case 'game_version':
			return isSingular ? 'game version' : 'game versions'
		default:
			return isSingular ? 'item' : 'items'
	}
})
const sortedChartDatasetIds = computed(() =>
	[...allChartDatasets.value]
		.sort((a, b) => {
			const totalDifference = getChartDatasetTotal(b) - getChartDatasetTotal(a)
			return (
				totalDifference || a.label.localeCompare(b.label) || a.projectId.localeCompare(b.projectId)
			)
		})
		.map((dataset) => dataset.projectId),
)
const topGraphDatasetIds = computed(() =>
	sortedChartDatasetIds.value.slice(0, TOP_GRAPH_DATASET_LIMIT),
)
const isShowingAllTableItems = computed(() => {
	if (selectedGraphDatasetIds.value.length !== sortedChartDatasetIds.value.length) return false
	const selectedDatasetIds = new Set(selectedGraphDatasetIds.value)
	return sortedChartDatasetIds.value.every((datasetId) => selectedDatasetIds.has(datasetId))
})
const isShowingTopGraphDatasets = computed(() => {
	if (selectedGraphDatasetIds.value.length !== topGraphDatasetIds.value.length) return false
	const selectedDatasetIds = new Set(selectedGraphDatasetIds.value)
	return topGraphDatasetIds.value.every((datasetId) => selectedDatasetIds.has(datasetId))
})
const isShowingTopTableItems = computed(() => {
	const topDatasetIds = new Set(
		sortedChartDatasetIds.value.slice(0, selectedGraphDatasetIds.value.length),
	)
	return selectedGraphDatasetIds.value.every((datasetId) => topDatasetIds.has(datasetId))
})
const tableSelectionSubheading = computed(() => {
	if (isGraphRenderDatasetLimitActive.value) {
		return `Showing ${GRAPH_RENDER_DATASET_LIMIT} ${tableBreakdownItemLabel.value} from table`
	}

	if (isShowingAllTableItems.value) {
		return `Showing all ${tableBreakdownItemLabel.value} from table`
	}

	if (isShowingTopTableItems.value) {
		if (tableProjectCount.value === 1) {
			return `Showing top ${tableBreakdownItemLabel.value} from table`
		} else {
			return `Showing top ${tableProjectCount.value} ${tableBreakdownItemLabel.value} from table`
		}
	}

	return `Showing ${tableProjectCount.value} ${tableBreakdownItemLabel.value} from table`
})
const shouldCapitalizeDatasetLabels = computed(() =>
	shouldCapitalizeBreakdownLabel(selectedBreakdown.value),
)

const chartType = computed<'line' | 'bar'>(() =>
	activeGraphViewMode.value === 'bar' ? 'bar' : 'line',
)
const canShowPreviousPeriodToggle = computed(
	() => activeGraphViewMode.value === 'line' && hasPreviousPeriodComparison.value,
)
const shouldShowPreviousPeriod = computed(
	() => canShowPreviousPeriodToggle.value && showPreviousPeriod.value,
)
const canUseRatioMode = computed(
	() =>
		(activeGraphViewMode.value === 'area' || activeGraphViewMode.value === 'bar') &&
		legendEntries.value.length > 1,
)
const isArea = computed(() => activeGraphViewMode.value === 'area')
const isStacked = computed(
	() =>
		isRatioMode.value ||
		activeGraphViewMode.value === 'area' ||
		activeGraphViewMode.value === 'bar',
)

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

const chartDatasetsByStat = computed<Record<AnalyticsDashboardStat, ChartDataset[]>>(() => {
	const datasetsByStat = {} as Record<AnalyticsDashboardStat, ChartDataset[]>
	const nextTimeSlices = timeSlices.value
	const nextSelectedProjects = selectedProjects.value
	const nextPalette = legendPalette.value
	const nextSelectedBreakdown = selectedBreakdown.value
	const nextSelectedFilters = selectedFilters.value
	const nextGetVersionProjectName = showProjectVersionNames.value
		? getVersionProjectName
		: undefined
	const nextSliceCount = sliceCount.value

	for (const stat of dashboardStats) {
		datasetsByStat[stat] = buildChartDatasets(
			nextTimeSlices,
			nextSelectedProjects,
			stat,
			nextPalette,
			nextSelectedBreakdown,
			nextSelectedFilters,
			getVersionDisplayName,
			nextGetVersionProjectName,
			nextSliceCount,
		)
	}

	return datasetsByStat
})
const previousChartDatasetsByStat = computed<Record<AnalyticsDashboardStat, ChartDataset[]>>(() => {
	const datasetsByStat = {} as Record<AnalyticsDashboardStat, ChartDataset[]>
	const nextTimeSlices = previousTimeSlices.value
	const nextSelectedProjects = selectedProjects.value
	const nextPalette = legendPalette.value
	const nextSelectedBreakdown = selectedBreakdown.value
	const nextSelectedFilters = selectedFilters.value
	const nextGetVersionProjectName = showProjectVersionNames.value
		? getVersionProjectName
		: undefined
	const nextSliceCount = sliceCount.value

	for (const stat of dashboardStats) {
		datasetsByStat[stat] = buildChartDatasets(
			nextTimeSlices,
			nextSelectedProjects,
			stat,
			nextPalette,
			nextSelectedBreakdown,
			nextSelectedFilters,
			getVersionDisplayName,
			nextGetVersionProjectName,
			nextSliceCount,
		)
	}

	return datasetsByStat
})
const allChartDatasets = computed(() => chartDatasetsByStat.value[activeStat.value])
const previousChartDatasets = computed(() => previousChartDatasetsByStat.value[activeStat.value])
const showAllSelectedGraphDatasets = ref(false)
const shouldUseDefaultGraphDatasetSelection = computed(
	() =>
		isGraphDatasetSelectionActive.value &&
		!hasExplicitGraphDatasetSelection.value &&
		selectedGraphDatasetIds.value.length === 0,
)
const selectedGraphDatasetIdSet = computed(() => {
	if (shouldUseDefaultGraphDatasetSelection.value) {
		return new Set(topGraphDatasetIds.value)
	}

	return new Set(selectedGraphDatasetIds.value)
})
const selectedChartDatasets = computed(() => {
	if (!isGraphDatasetSelectionActive.value) {
		return allChartDatasets.value
	}

	return allChartDatasets.value.filter((dataset) =>
		selectedGraphDatasetIdSet.value.has(dataset.projectId),
	)
})
const sortedSelectedChartDatasetIds = computed(() =>
	[...selectedChartDatasets.value]
		.sort((a, b) => {
			const totalDifference = getChartDatasetTotal(b) - getChartDatasetTotal(a)
			return (
				totalDifference || a.label.localeCompare(b.label) || a.projectId.localeCompare(b.projectId)
			)
		})
		.map((dataset) => dataset.projectId),
)
const isGraphRenderDatasetOverLimit = computed(
	() =>
		isGraphDatasetSelectionActive.value &&
		selectedChartDatasets.value.length > GRAPH_RENDER_DATASET_LIMIT,
)
const showGraphRenderLimitButton = computed(() => isGraphRenderDatasetOverLimit.value)
const graphRenderLimitButtonLabel = computed(() =>
	showAllSelectedGraphDatasets.value ? 'Show limited' : 'Show all',
)
const showTopGraphDatasetsButton = computed(
	() =>
		isGraphDatasetSelectionActive.value &&
		topGraphDatasetIds.value.length > 0 &&
		!isShowingTopGraphDatasets.value,
)
const isGraphRenderDatasetLimitActive = computed(
	() => isGraphRenderDatasetOverLimit.value && !showAllSelectedGraphDatasets.value,
)
const limitedGraphDatasetIds = computed(
	() => new Set(sortedSelectedChartDatasetIds.value.slice(0, GRAPH_RENDER_DATASET_LIMIT)),
)
const selectableChartDatasets = computed(() => {
	if (!isGraphRenderDatasetLimitActive.value) {
		return selectedChartDatasets.value
	}

	return selectedChartDatasets.value.filter((dataset) =>
		limitedGraphDatasetIds.value.has(dataset.projectId),
	)
})

const chartContainer = ref<HTMLElement | null>(null)
const legendContainer = ref<HTMLElement | null>(null)
const chartTooltip = ref<InstanceType<typeof AnalyticsChartTooltip> | null>(null)
const showAllSelectedGraphDatasetsModal = ref<InstanceType<typeof NewModal> | null>(null)
const chartGeometry = ref<AnalyticsChartGeometryPayload | null>(null)
const containerSize = reactive({ width: 0, height: 0 })
const {
	showTopFade: showLegendTopFade,
	showBottomFade: showLegendBottomFade,
	checkScrollState: checkLegendScrollState,
	forceCheck: forceCheckLegendScrollState,
} = useScrollIndicator(legendContainer)
let resizeObserver: ResizeObserver | null = null
let clearIgnoredChartClickTimeout: ReturnType<typeof setTimeout> | null = null

onMounted(() => {
	if (chartContainer.value && typeof ResizeObserver !== 'undefined') {
		resizeObserver = new ResizeObserver((entries) => {
			const entry = entries[0]
			if (!entry) return
			containerSize.width = entry.contentRect.width
			containerSize.height = entry.contentRect.height
		})
		resizeObserver.observe(chartContainer.value)
	}

	window.addEventListener('keydown', updateShiftKeyState)
	window.addEventListener('keyup', updateShiftKeyState)
	window.addEventListener('blur', clearShiftKeyState)
})

onBeforeUnmount(() => {
	resizeObserver?.disconnect()
	resizeObserver = null
	window.removeEventListener('keydown', updateShiftKeyState)
	window.removeEventListener('keyup', updateShiftKeyState)
	window.removeEventListener('blur', clearShiftKeyState)
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
const hoveredLegendEntryId = ref<string | null>(null)
const isLegendExpanded = ref(false)
const isShiftKeyPressed = ref(false)
const monetizationPopoverId = useId()
const promotedCollapsedLegendEntryIds = ref<string[]>([])
const hiddenDatasetIds = computed(() => new Set(hiddenGraphDatasetIds.value))
const promotedCollapsedLegendEntryIdSet = computed(
	() => new Set(promotedCollapsedLegendEntryIds.value),
)
const previousChartDatasetByOriginalId = computed(() => {
	const datasets = new Map<string, ChartDataset>()
	for (const dataset of previousChartDatasets.value) {
		datasets.set(dataset.projectId, dataset)
	}
	return datasets
})

const LEGEND_MAX_ITEMS = 8

type LegendEntry = {
	id: string
	name: string
	projectName?: string
	color: string
	totalValue: number
	hidden: boolean
	isPreviousPeriod?: boolean
}

function getChartDatasetTotal(dataset: ChartDataset) {
	return dataset.data.reduce((sum, value) => sum + value, 0)
}

function getPreviousPeriodDatasetId(datasetId: string) {
	return `${PREVIOUS_PERIOD_DATASET_ID_PREFIX}${datasetId}`
}

function compareLegendEntries(a: LegendEntry, b: LegendEntry) {
	if (selectedBreakdown.value === 'monetization') {
		const aOrder = MONETIZATION_LEGEND_ENTRY_ORDER.get(a.id)
		const bOrder = MONETIZATION_LEGEND_ENTRY_ORDER.get(b.id)

		if (aOrder !== undefined || bOrder !== undefined) {
			return (aOrder ?? Number.MAX_SAFE_INTEGER) - (bOrder ?? Number.MAX_SAFE_INTEGER)
		}
	}

	return b.totalValue - a.totalValue || a.name.localeCompare(b.name)
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

function updateShiftKeyState(event: KeyboardEvent) {
	isShiftKeyPressed.value = event.shiftKey
}

function clearShiftKeyState() {
	isShiftKeyPressed.value = false
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

function onChartWheel(event: WheelEvent) {
	if (!hoverState.visible) return
	chartTooltip.value?.consumeWheel(event)
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

const currentLegendEntries = computed<LegendEntry[]>(() =>
	selectableChartDatasets.value
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
		.sort(compareLegendEntries),
)

const legendEntries = computed<LegendEntry[]>(() => {
	if (!shouldShowPreviousPeriod.value) {
		return currentLegendEntries.value
	}

	return currentLegendEntries.value.flatMap((entry) => {
		const previousDataset = previousChartDatasetByOriginalId.value.get(entry.id)
		const previousEntry: LegendEntry = {
			id: getPreviousPeriodDatasetId(entry.id),
			name: `${entry.name} (Prev.)`,
			projectName: entry.projectName,
			color: entry.color,
			totalValue: previousDataset ? getChartDatasetTotal(previousDataset) : 0,
			hidden: hiddenDatasetIds.value.has(getPreviousPeriodDatasetId(entry.id)),
			isPreviousPeriod: true,
		}

		return [entry, previousEntry]
	})
})

const canToggleLegendExpansion = computed(() => legendEntries.value.length > LEGEND_MAX_ITEMS)

const displayedLegendEntries = computed<LegendEntry[]>(() => {
	if (!isLegendExpanded.value && canToggleLegendExpansion.value) {
		return legendEntries.value.filter(
			(entry, index) =>
				index < LEGEND_MAX_ITEMS || promotedCollapsedLegendEntryIdSet.value.has(entry.id),
		)
	}
	return legendEntries.value
})

const collapsedLegendEntryIds = computed(() => {
	if (isLegendExpanded.value || !canToggleLegendExpansion.value) return new Set<string>()
	return new Set(
		legendEntries.value
			.slice(LEGEND_MAX_ITEMS)
			.filter((entry) => !promotedCollapsedLegendEntryIdSet.value.has(entry.id))
			.map((entry) => entry.id),
	)
})

function toggleLegendExpansion() {
	isLegendExpanded.value = !isLegendExpanded.value
}

function promoteCollapsedLegendEntry(datasetId: string) {
	if (promotedCollapsedLegendEntryIdSet.value.has(datasetId)) return
	promotedCollapsedLegendEntryIds.value = [...promotedCollapsedLegendEntryIds.value, datasetId]
}

function openShowAllSelectedGraphDatasetsModal(event: MouseEvent) {
	showAllSelectedGraphDatasetsModal.value?.show(event)
}

function toggleGraphRenderLimit(event: MouseEvent) {
	if (showAllSelectedGraphDatasets.value) {
		showAllSelectedGraphDatasets.value = false
		return
	}

	openShowAllSelectedGraphDatasetsModal(event)
}

function confirmShowAllSelectedGraphDatasets() {
	showAllSelectedGraphDatasets.value = true
	showAllSelectedGraphDatasetsModal.value?.hide()
}

function showTopGraphDatasets() {
	selectedGraphDatasetIds.value = []
	hasExplicitGraphDatasetSelection.value = false
	showAllSelectedGraphDatasets.value = false
}

watch(
	displayedLegendEntries,
	() => {
		nextTick(() => {
			forceCheckLegendScrollState()
		})
	},
	{ immediate: true, flush: 'post' },
)

watch(canToggleLegendExpansion, (canToggle) => {
	if (!canToggle) {
		isLegendExpanded.value = false
	}
})

watch(canUseRatioMode, (canUse) => {
	if (!canUse) {
		isRatioMode.value = false
	}
})

const chartDatasetById = computed(() => {
	const datasets = new Map<string, ChartDataset>()
	for (const dataset of selectableChartDatasets.value) {
		datasets.set(dataset.projectId, dataset)

		if (!shouldShowPreviousPeriod.value) {
			continue
		}

		const previousDataset = previousChartDatasetByOriginalId.value.get(dataset.projectId)
		const previousData = Array.from(
			{ length: dataset.data.length },
			(_, index) => previousDataset?.data[index] ?? 0,
		)
		datasets.set(getPreviousPeriodDatasetId(dataset.projectId), {
			projectId: getPreviousPeriodDatasetId(dataset.projectId),
			label: `${dataset.label} (Prev.)`,
			projectName: dataset.projectName,
			data: previousData,
			borderColor: dataset.borderColor,
			backgroundColor: dataset.backgroundColor,
			borderDash: PREVIOUS_PERIOD_BORDER_DASH,
		})
	}
	return datasets
})

const hoverRatioSliceTotals = computed(() => {
	const sliceLength = selectableChartDatasets.value.reduce(
		(maxLength, dataset) => Math.max(maxLength, dataset.data.length),
		0,
	)
	const totals = new Array<number>(sliceLength).fill(0)

	for (const legendEntry of legendEntries.value) {
		const dataset = chartDatasetById.value.get(legendEntry.id)
		if (!dataset) continue

		for (let i = 0; i < sliceLength; i++) {
			totals[i] += dataset.data[i] ?? 0
		}
	}

	return totals
})

const baseVisibleChartDatasets = computed(() =>
	legendEntries.value
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

	const sliceLength = datasets.reduce(
		(maxLength, dataset) => Math.max(maxLength, dataset.data.length),
		0,
	)
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

const highlightedChartDatasetId = computed(() => {
	const datasetId = hoveredLegendEntryId.value
	if (!datasetId || !visibleChartDatasetById.value.has(datasetId)) return null
	return datasetId
})

function isLegendEntryToggleDisabled(legendEntry: LegendEntry) {
	if (legendEntry.hidden) return false
	const visibleCount = legendEntries.value.filter((entry) => !entry.hidden).length
	return visibleCount <= 1
}

function getLegendEntryTooltip(legendEntry: LegendEntry) {
	return legendEntry.projectName ?? ''
}

function isUnmonetizedLegendEntry(legendEntry: LegendEntry) {
	return selectedBreakdown.value === 'monetization' && legendEntry.id === 'breakdown:unmonetized'
}

function setHoveredLegendEntryId(datasetId: string) {
	hoveredLegendEntryId.value = datasetId
}

function clearHoveredLegendEntryId(datasetId: string) {
	if (hoveredLegendEntryId.value === datasetId) {
		hoveredLegendEntryId.value = null
	}
}

function clearLegendHoverState() {
	hoveredLegendEntryId.value = null
}

function toggleLegendEntryVisibility(datasetId: string) {
	const nextHiddenDatasetIds = new Set(hiddenDatasetIds.value)
	if (nextHiddenDatasetIds.has(datasetId)) {
		nextHiddenDatasetIds.delete(datasetId)
	} else {
		const visibleCount = legendEntries.value.filter((entry) => !entry.hidden).length
		if (visibleCount <= 1) return
		nextHiddenDatasetIds.add(datasetId)
	}
	hiddenGraphDatasetIds.value = Array.from(nextHiddenDatasetIds)
}

function soloLegendEntry(datasetId: string) {
	const currentLegendEntryIds = new Set(legendEntries.value.map((entry) => entry.id))
	const otherIds = legendEntries.value.map((entry) => entry.id).filter((id) => id !== datasetId)
	const isAlreadySolo =
		!hiddenDatasetIds.value.has(datasetId) && otherIds.every((id) => hiddenDatasetIds.value.has(id))

	if (isAlreadySolo) {
		hiddenGraphDatasetIds.value = hiddenGraphDatasetIds.value.filter(
			(hiddenDatasetId) => !currentLegendEntryIds.has(hiddenDatasetId),
		)
		return
	}

	const nextHiddenDatasetIds = new Set(hiddenDatasetIds.value)
	for (const legendEntry of legendEntries.value) {
		if (legendEntry.id === datasetId) {
			nextHiddenDatasetIds.delete(legendEntry.id)
		} else {
			nextHiddenDatasetIds.add(legendEntry.id)
		}
	}
	hiddenGraphDatasetIds.value = Array.from(nextHiddenDatasetIds)
}

function onLegendEntryClick(event: MouseEvent, datasetId: string) {
	if (event.shiftKey) {
		soloLegendEntry(datasetId)
		clearLegendHoverState()
		return
	}
	toggleLegendEntryVisibility(datasetId)
	clearLegendHoverState()
}

function onTooltipEntryClick(datasetId: string, shiftKey: boolean) {
	if (!chartDatasetById.value.has(datasetId)) return

	if (collapsedLegendEntryIds.value.has(datasetId)) {
		promoteCollapsedLegendEntry(datasetId)
	}

	if (shiftKey) {
		soloLegendEntry(datasetId)
		clearLegendHoverState()
		return
	}
	toggleLegendEntryVisibility(datasetId)
	clearLegendHoverState()
}

function areStringArraysEqual(left: string[], right: string[]) {
	if (left.length !== right.length) return false
	for (let index = 0; index < left.length; index += 1) {
		if (left[index] !== right[index]) return false
	}
	return true
}

watch([chartLabels, allChartDatasets], () => {
	isHoverPinned.value = false
	clearHoverState()
})

watch([() => selectedGraphDatasetIds.value.join('\u0000'), allChartDatasets], () => {
	showAllSelectedGraphDatasets.value = false
})

watch(isDataLoading, (loading) => {
	if (!loading) return
	isHoverPinned.value = false
	clearHoverState()
})

watch(
	[allChartDatasets, legendEntries],
	([datasets]) => {
		if (datasets.length === 0) return

		const availableDatasetIds = new Set(legendEntries.value.map((entry) => entry.id))

		const nextHiddenDatasetIds = hiddenGraphDatasetIds.value.filter((datasetId) =>
			availableDatasetIds.has(datasetId),
		)
		if (
			legendEntries.value.length > 0 &&
			legendEntries.value.every((entry) => nextHiddenDatasetIds.includes(entry.id))
		) {
			const firstLegendEntry = legendEntries.value[0]
			if (firstLegendEntry) {
				const firstLegendEntryIndex = nextHiddenDatasetIds.indexOf(firstLegendEntry.id)
				if (firstLegendEntryIndex !== -1) {
					nextHiddenDatasetIds.splice(firstLegendEntryIndex, 1)
				}
			}
		}

		if (!areStringArraysEqual(hiddenGraphDatasetIds.value, nextHiddenDatasetIds)) {
			hiddenGraphDatasetIds.value = nextHiddenDatasetIds
		}

		const nextPromotedCollapsedLegendEntryIds = promotedCollapsedLegendEntryIds.value.filter(
			(datasetId) => availableDatasetIds.has(datasetId),
		)
		if (
			!areStringArraysEqual(
				promotedCollapsedLegendEntryIds.value,
				nextPromotedCollapsedLegendEntryIds,
			)
		) {
			promotedCollapsedLegendEntryIds.value = nextPromotedCollapsedLegendEntryIds
		}
	},
	{ immediate: true },
)

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
	if (isRatioMode.value) return hoverRatioSliceTotals.value[sliceIndex] ?? 0

	return currentLegendEntries.value.reduce((sum, legendEntry) => {
		if (legendEntry.hidden) return sum
		const dataset = chartDatasetById.value.get(legendEntry.id)
		return sum + (dataset?.data[sliceIndex] ?? 0)
	}, 0)
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
	const totalValue = hoverTotalValue.value

	return legendEntries.value.map((legendEntry) => {
		const dataset = chartDatasetById.value.get(legendEntry.id)
		const value = dataset?.data[sliceIndex] ?? 0
		const ratioValue = totalValue === 0 ? 0 : (value / totalValue) * 100
		return {
			projectId: legendEntry.id,
			name: legendEntry.name,
			projectName: legendEntry.projectName,
			color: legendEntry.color,
			formattedValue: isRatioMode.value
				? `${ratioValue.toFixed(1)}%`
				: formatMetricValue(value, activeStat.value, formatNumber),
			hidden: legendEntry.hidden,
			toggleDisabled: !legendEntry.hidden && isLegendEntryToggleDisabled(legendEntry),
			isPreviousPeriod: legendEntry.isPreviousPeriod,
		}
	})
})
</script>

<style>
.v-popper--theme-analytics-monetization-popover .v-popper__inner {
	overflow: visible !important;
	background: transparent !important;
	padding: 0 !important;
	border: 0 !important;
	box-shadow: none !important;
}

.v-popper--theme-analytics-monetization-popover .v-popper__arrow-container {
	display: none;
}
</style>
