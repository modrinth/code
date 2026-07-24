import type { Labrinth } from '@modrinth/api-client'
import { useVIntl } from '@modrinth/ui'
import { computed, type ComputedRef, ref, watch } from 'vue'

import { useTheme } from '~/composables/nuxt-accessors'
import { isDarkTheme } from '~/plugins/theme/index.ts'
import type {
	AnalyticsBreakdownGroup,
	AnalyticsBreakdownPreset,
	AnalyticsDashboardContextValue,
	AnalyticsDashboardProject,
	AnalyticsDashboardStat,
	AnalyticsSelectedFilters,
} from '~/providers/analytics/analytics'

import {
	analyticsChartMessages,
	analyticsMessages,
	formatAnalyticsGraphTitle,
	type FormatMessage,
	getAnalyticsBreakdownItemType,
} from '../analytics-messages'
import {
	ANALYTICS_DASHBOARD_STATS,
	DARK_LEGEND_PALETTE,
	GRAPH_RENDER_DATASET_LIMIT,
	LIGHT_LEGEND_PALETTE,
	TOP_GRAPH_DATASET_LIMIT,
} from './analytics-chart-constants'
import {
	buildChartDatasets,
	buildTimeAxisLabels,
	type ChartDataset,
	getChartDatasetTotal,
	getShortHourlyAxisTickLimit,
	getSliceCount,
	shouldCapitalizeBreakdownLabel,
} from './analytics-chart-utils'

export function useAnalyticsChartDatasets(
	context: Pick<
		AnalyticsDashboardContextValue,
		| 'activeStat'
		| 'activeGraphViewMode'
		| 'isRatioMode'
		| 'showPreviousPeriod'
		| 'hasPreviousPeriodComparison'
		| 'hasProjectContext'
		| 'displayedFetchRequest'
		| 'displayedTimeSlices'
		| 'displayedPreviousTimeSlices'
		| 'displayedSelectedGroupBy'
		| 'displayedSelectedBreakdowns'
		| 'displayedSelectedFilters'
		| 'resolvedActiveBreakdownGroup'
		| 'hiddenGraphDatasetIds'
		| 'hasExplicitGraphDatasetSelection'
		| 'isGraphDatasetSelectionActive'
		| 'selectedGraphDatasetIds'
		| 'defaultGraphDatasetIds'
		| 'topGraphDatasetIds'
		| 'projectNamesById'
		| 'userNamesById'
		| 'dependentProjectTypesById'
		| 'getVersionDisplayName'
		| 'getVersionProjectName'
	>,
	selectedProjects: ComputedRef<AnalyticsDashboardProject[]>,
	hasAvailableProjects: ComputedRef<boolean>,
) {
	const theme = useTheme()
	const { formatMessage } = useVIntl()
	const showAllSelectedGraphDatasets = ref(false)

	const chartRangeBounds = computed(() => {
		const nextFetchRequest = context.displayedFetchRequest.value
		if (!nextFetchRequest) return null
		return {
			start: new Date(nextFetchRequest.time_range.start),
			end: new Date(nextFetchRequest.time_range.end),
		}
	})
	const showProjectVersionNames = computed(
		() =>
			context.displayedSelectedBreakdowns.value.includes('version_id') &&
			selectedProjects.value.length > 1,
	)
	const tableProjectCount = computed(() => context.selectedGraphDatasetIds.value.length)
	const isTableGraphSelectionEmpty = computed(
		() =>
			context.isGraphDatasetSelectionActive.value &&
			context.hasExplicitGraphDatasetSelection.value &&
			tableProjectCount.value === 0,
	)
	const showEmptyChartState = computed(
		() => selectedProjects.value.length === 0 || isTableGraphSelectionEmpty.value,
	)
	const emptyChartMessage = computed(() => {
		if (isTableGraphSelectionEmpty.value) {
			return formatMessage(analyticsChartMessages.selectTableItemsEmpty)
		}

		if (context.hasProjectContext.value) {
			return formatMessage(analyticsMessages.noDataAvailableForAnalytics)
		}

		return hasAvailableProjects.value
			? formatMessage(analyticsMessages.noDataAvailable)
			: formatMessage(analyticsMessages.noProjectsAvailableForAnalytics)
	})
	const legendPalette = computed(() =>
		isDarkTheme(theme.active) ? DARK_LEGEND_PALETTE : LIGHT_LEGEND_PALETTE,
	)
	const graphTitle = computed(() =>
		formatAnalyticsGraphTitle(context.activeStat.value, formatMessage),
	)
	const showTableSelectionSubheading = computed(
		() => context.isGraphDatasetSelectionActive.value && tableProjectCount.value > 0,
	)
	const tableBreakdownItemType = computed(() =>
		getAnalyticsBreakdownItemType(context.displayedSelectedBreakdowns.value),
	)
	const shouldCapitalizeDatasetLabels = computed(() =>
		shouldCapitalizeBreakdownLabel(context.displayedSelectedBreakdowns.value),
	)
	const chartType = computed<'line' | 'bar'>(() =>
		context.activeGraphViewMode.value === 'bar' ? 'bar' : 'line',
	)
	const canShowPreviousPeriodToggle = computed(
		() => context.activeGraphViewMode.value === 'line' && context.hasPreviousPeriodComparison.value,
	)
	const shouldShowPreviousPeriod = computed(
		() => canShowPreviousPeriodToggle.value && context.showPreviousPeriod.value,
	)
	const isArea = computed(() => context.activeGraphViewMode.value === 'area')
	const isStacked = computed(
		() =>
			context.isRatioMode.value ||
			context.activeGraphViewMode.value === 'area' ||
			context.activeGraphViewMode.value === 'bar',
	)
	const sliceCount = computed(() => {
		const nextFetchRequest = context.displayedFetchRequest.value
		const fallback = context.displayedTimeSlices.value.length
		if (!nextFetchRequest) return Math.max(1, fallback)
		return getSliceCount(nextFetchRequest.time_range, fallback)
	})
	const chartLabels = computed(() => {
		const nextFetchRequest = context.displayedFetchRequest.value
		if (!nextFetchRequest) return []
		return buildTimeAxisLabels(
			nextFetchRequest.time_range,
			sliceCount.value,
			context.displayedSelectedGroupBy.value,
		)
	})
	const xAxisTickLimit = computed(() => {
		const nextFetchRequest = context.displayedFetchRequest.value
		return nextFetchRequest
			? getShortHourlyAxisTickLimit(
					nextFetchRequest.time_range,
					context.displayedSelectedGroupBy.value,
				)
			: undefined
	})
	const chartDatasetsByStat = computed<Record<AnalyticsDashboardStat, ChartDataset[]>>(() =>
		buildDatasetsByStat(
			context.displayedTimeSlices.value,
			selectedProjects.value,
			legendPalette.value,
			context.displayedSelectedBreakdowns.value,
			context.displayedSelectedFilters.value,
			context.dependentProjectTypesById.value,
			context.projectNamesById.value,
			context.userNamesById.value,
			context.getVersionDisplayName,
			showProjectVersionNames.value ? context.getVersionProjectName : undefined,
			formatMessage,
			sliceCount.value,
			context.resolvedActiveBreakdownGroup.value,
		),
	)
	const previousChartDatasetsByStat = computed<Record<AnalyticsDashboardStat, ChartDataset[]>>(() =>
		buildDatasetsByStat(
			context.displayedPreviousTimeSlices.value,
			selectedProjects.value,
			legendPalette.value,
			context.displayedSelectedBreakdowns.value,
			context.displayedSelectedFilters.value,
			context.dependentProjectTypesById.value,
			context.projectNamesById.value,
			context.userNamesById.value,
			context.getVersionDisplayName,
			showProjectVersionNames.value ? context.getVersionProjectName : undefined,
			formatMessage,
			sliceCount.value,
			context.resolvedActiveBreakdownGroup.value,
		),
	)
	const allChartDatasets = computed(() => chartDatasetsByStat.value[context.activeStat.value])
	const previousChartDatasets = computed(
		() => previousChartDatasetsByStat.value[context.activeStat.value],
	)
	const sortedChartDatasetIds = computed(() => sortDatasetsByTotal(allChartDatasets.value))
	const chartTopGraphDatasetIds = computed(() =>
		sortedChartDatasetIds.value.slice(0, TOP_GRAPH_DATASET_LIMIT),
	)
	const fallbackDefaultGraphDatasetIds = computed(() =>
		context.defaultGraphDatasetIds.value.length > 0
			? context.defaultGraphDatasetIds.value
			: chartTopGraphDatasetIds.value,
	)
	const isShowingAllTableItems = computed(() => {
		if (context.selectedGraphDatasetIds.value.length !== sortedChartDatasetIds.value.length) {
			return false
		}
		const selectedDatasetIds = new Set(context.selectedGraphDatasetIds.value)
		return sortedChartDatasetIds.value.every((datasetId) => selectedDatasetIds.has(datasetId))
	})
	const isShowingTopGraphDatasets = computed(() => {
		if (
			context.selectedGraphDatasetIds.value.length !== fallbackDefaultGraphDatasetIds.value.length
		) {
			return false
		}
		const selectedDatasetIds = new Set(context.selectedGraphDatasetIds.value)
		return fallbackDefaultGraphDatasetIds.value.every((datasetId) =>
			selectedDatasetIds.has(datasetId),
		)
	})
	const isShowingTopTableItems = computed(() => {
		const topDatasetIds = new Set(
			context.topGraphDatasetIds.value.slice(0, context.selectedGraphDatasetIds.value.length),
		)
		return context.selectedGraphDatasetIds.value.every((datasetId) => topDatasetIds.has(datasetId))
	})
	const isGraphRenderDatasetOverLimit = computed(
		() =>
			context.isGraphDatasetSelectionActive.value &&
			selectedChartDatasets.value.length > GRAPH_RENDER_DATASET_LIMIT,
	)
	const isGraphRenderDatasetLimitActive = computed(
		() => isGraphRenderDatasetOverLimit.value && !showAllSelectedGraphDatasets.value,
	)
	const tableSelectionSubheading = computed(() => {
		if (isGraphRenderDatasetLimitActive.value) {
			return formatMessage(analyticsChartMessages.tableSelectionLimited, {
				limit: GRAPH_RENDER_DATASET_LIMIT,
				itemType: tableBreakdownItemType.value,
			})
		}

		if (isShowingAllTableItems.value) {
			return formatMessage(analyticsChartMessages.tableSelectionAll, {
				count: tableProjectCount.value,
				itemType: tableBreakdownItemType.value,
			})
		}

		if (isShowingTopTableItems.value) {
			return formatMessage(analyticsChartMessages.tableSelectionTop, {
				count: tableProjectCount.value,
				itemType: tableBreakdownItemType.value,
			})
		}

		return formatMessage(analyticsChartMessages.tableSelectionCount, {
			count: tableProjectCount.value,
			itemType: tableBreakdownItemType.value,
		})
	})
	const shouldUseDefaultGraphDatasetSelection = computed(
		() =>
			context.isGraphDatasetSelectionActive.value &&
			!context.hasExplicitGraphDatasetSelection.value &&
			context.selectedGraphDatasetIds.value.length === 0,
	)
	const selectedGraphDatasetIdSet = computed(() => {
		if (shouldUseDefaultGraphDatasetSelection.value) {
			return new Set(fallbackDefaultGraphDatasetIds.value)
		}

		return new Set(context.selectedGraphDatasetIds.value)
	})
	const selectedChartDatasets = computed(() => {
		if (!context.isGraphDatasetSelectionActive.value) {
			return allChartDatasets.value
		}

		return allChartDatasets.value.filter((dataset) =>
			selectedGraphDatasetIdSet.value.has(dataset.projectId),
		)
	})
	const sortedSelectedChartDatasetIds = computed(() =>
		sortDatasetsByTotal(selectedChartDatasets.value),
	)
	const showGraphRenderLimitButton = computed(() => isGraphRenderDatasetOverLimit.value)
	const graphRenderLimitButtonLabel = computed(() =>
		showAllSelectedGraphDatasets.value
			? formatMessage(analyticsChartMessages.showLimited)
			: formatMessage(analyticsChartMessages.showAll),
	)
	const showTopGraphDatasetsButton = computed(
		() =>
			context.isGraphDatasetSelectionActive.value &&
			context.topGraphDatasetIds.value.length > 0 &&
			!isShowingTopGraphDatasets.value,
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

	function showTopGraphDatasets() {
		context.selectedGraphDatasetIds.value = []
		context.hasExplicitGraphDatasetSelection.value = false
		showAllSelectedGraphDatasets.value = false
	}

	watch([() => context.selectedGraphDatasetIds.value.join('\u0000'), allChartDatasets], () => {
		showAllSelectedGraphDatasets.value = false
	})

	return {
		showAllSelectedGraphDatasets,
		chartRangeBounds,
		showProjectVersionNames,
		tableProjectCount,
		isTableGraphSelectionEmpty,
		showEmptyChartState,
		emptyChartMessage,
		legendPalette,
		graphTitle,
		showTableSelectionSubheading,
		shouldCapitalizeDatasetLabels,
		chartType,
		canShowPreviousPeriodToggle,
		shouldShowPreviousPeriod,
		isArea,
		isStacked,
		sliceCount,
		chartLabels,
		xAxisTickLimit,
		chartDatasetsByStat,
		previousChartDatasetsByStat,
		allChartDatasets,
		previousChartDatasets,
		sortedChartDatasetIds,
		chartTopGraphDatasetIds,
		fallbackDefaultGraphDatasetIds,
		isShowingAllTableItems,
		isShowingTopGraphDatasets,
		isShowingTopTableItems,
		tableSelectionSubheading,
		shouldUseDefaultGraphDatasetSelection,
		selectedGraphDatasetIdSet,
		selectedChartDatasets,
		sortedSelectedChartDatasetIds,
		isGraphRenderDatasetOverLimit,
		showGraphRenderLimitButton,
		graphRenderLimitButtonLabel,
		showTopGraphDatasetsButton,
		isGraphRenderDatasetLimitActive,
		limitedGraphDatasetIds,
		selectableChartDatasets,
		showTopGraphDatasets,
	}
}

function buildDatasetsByStat(
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
	selectedProjects: AnalyticsDashboardProject[],
	palette: string[],
	selectedBreakdowns: readonly AnalyticsBreakdownPreset[],
	selectedFilters: AnalyticsSelectedFilters,
	dependentProjectTypesById: ReadonlyMap<string, readonly string[]>,
	projectNamesById: ReadonlyMap<string, string>,
	userNamesById: ReadonlyMap<string, string>,
	getVersionDisplayName: (versionId: string) => string,
	getVersionProjectName: ((versionId: string) => string | undefined) | undefined,
	formatMessage: FormatMessage,
	sliceCount: number,
	breakdownGroup: AnalyticsBreakdownGroup | null,
) {
	const datasetsByStat = {} as Record<AnalyticsDashboardStat, ChartDataset[]>
	for (const stat of ANALYTICS_DASHBOARD_STATS) {
		datasetsByStat[stat] = buildChartDatasets(
			timeSlices,
			selectedProjects,
			stat,
			palette,
			selectedBreakdowns,
			selectedFilters,
			dependentProjectTypesById,
			projectNamesById,
			userNamesById,
			getVersionDisplayName,
			getVersionProjectName,
			formatMessage,
			sliceCount,
			breakdownGroup,
		)
	}
	return datasetsByStat
}

function sortDatasetsByTotal(datasets: ChartDataset[]) {
	return [...datasets]
		.sort((a, b) => {
			const totalDifference = getChartDatasetTotal(b) - getChartDatasetTotal(a)
			return (
				totalDifference || a.label.localeCompare(b.label) || a.projectId.localeCompare(b.projectId)
			)
		})
		.map((dataset) => dataset.projectId)
}
