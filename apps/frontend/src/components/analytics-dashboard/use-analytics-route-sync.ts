import type { Ref } from 'vue'
import type { LocationQuery } from 'vue-router'

import {
	areSelectedFiltersEqual,
	areStringArraysEqual,
	buildAnalyticsQueryBuilderRouteQuery,
	getAnalyticsBreakdownPresetsForProjectSelection,
	hasAnalyticsQueryBuilderRouteChange,
	readAnalyticsGraphState,
	readAnalyticsQueryBuilderState,
} from '~/components/analytics-dashboard/analytics-route-query'
import type {
	AnalyticsBreakdownPreset,
	AnalyticsDashboardStat,
	AnalyticsGraphViewMode,
	AnalyticsGroupByPreset,
	AnalyticsLastTimeframeUnit,
	AnalyticsSelectedBreakdowns,
	AnalyticsSelectedFilters,
	AnalyticsTimeframeMode,
	AnalyticsTimeframePreset,
} from '~/providers/analytics/analytics-types'

export type AnalyticsQueryBuilderRouteNavigationMode = 'push' | 'replace'

export interface AnalyticsQueryBuilderRefs {
	selectedProjectIds: Ref<string[]>
	selectedTimeframeMode: Ref<AnalyticsTimeframeMode>
	selectedTimeframe: Ref<AnalyticsTimeframePreset>
	selectedLastTimeframeAmount: Ref<number>
	selectedLastTimeframeUnit: Ref<AnalyticsLastTimeframeUnit>
	selectedCustomTimeframeStartDate: Ref<string>
	selectedCustomTimeframeEndDate: Ref<string>
	selectedGroupBy: Ref<AnalyticsGroupByPreset>
	selectedBreakdowns: Ref<AnalyticsSelectedBreakdowns>
	selectedFilters: Ref<AnalyticsSelectedFilters>
}

export interface AnalyticsGraphRefs {
	activeStat: Ref<AnalyticsDashboardStat>
	activeGraphViewMode: Ref<AnalyticsGraphViewMode>
	isRatioMode: Ref<boolean>
	showChartEvents: Ref<boolean>
	showProjectEvents: Ref<boolean>
	showPreviousPeriod: Ref<boolean>
	hiddenGraphDatasetIds: Ref<string[]>
	hasExplicitGraphDatasetSelection: Ref<boolean>
	selectedGraphDatasetIds: Ref<string[]>
}

export interface UseAnalyticsRouteSyncOptions {
	queryBuilder: AnalyticsQueryBuilderRefs
	graph: AnalyticsGraphRefs
	availableProjectIds: Ref<string[]>
	defaultProjectIds: Ref<string[]>
	sanitizeSelectedFilters: (
		breakdowns: readonly AnalyticsBreakdownPreset[],
		filters: AnalyticsSelectedFilters,
	) => AnalyticsSelectedFilters
}

export function useAnalyticsRouteSync(options: UseAnalyticsRouteSyncOptions) {
	const { queryBuilder, graph, availableProjectIds, defaultProjectIds, sanitizeSelectedFilters } =
		options
	const route = useRoute()
	const router = useRouter()

	let nextAnalyticsRouteNavigationMode: AnalyticsQueryBuilderRouteNavigationMode = 'replace'

	function replaceNextAnalyticsRouteNavigation() {
		nextAnalyticsRouteNavigationMode = 'replace'
	}

	function consumeAnalyticsRouteNavigationMode(): AnalyticsQueryBuilderRouteNavigationMode {
		const navigationMode = nextAnalyticsRouteNavigationMode
		nextAnalyticsRouteNavigationMode = 'push'
		return navigationMode
	}

	function getSelectedAnalyticsQueryBuilderState() {
		return {
			selectedProjectIds: queryBuilder.selectedProjectIds.value,
			selectedTimeframeMode: queryBuilder.selectedTimeframeMode.value,
			selectedTimeframe: queryBuilder.selectedTimeframe.value,
			selectedLastTimeframeAmount: queryBuilder.selectedLastTimeframeAmount.value,
			selectedLastTimeframeUnit: queryBuilder.selectedLastTimeframeUnit.value,
			selectedCustomTimeframeStartDate: queryBuilder.selectedCustomTimeframeStartDate.value,
			selectedCustomTimeframeEndDate: queryBuilder.selectedCustomTimeframeEndDate.value,
			selectedGroupBy: queryBuilder.selectedGroupBy.value,
			selectedBreakdowns: queryBuilder.selectedBreakdowns.value,
			selectedFilters: queryBuilder.selectedFilters.value,
		}
	}

	function getSelectedAnalyticsGraphState() {
		return {
			activeStat: graph.activeStat.value,
			activeGraphViewMode: graph.activeGraphViewMode.value,
			isRatioMode: graph.isRatioMode.value,
			showChartEvents: graph.showChartEvents.value,
			showProjectEvents: graph.showProjectEvents.value,
			showPreviousPeriod: graph.showPreviousPeriod.value,
			hiddenGraphDatasetIds: graph.hiddenGraphDatasetIds.value,
			selectedGraphDatasetIds: graph.hasExplicitGraphDatasetSelection.value
				? graph.selectedGraphDatasetIds.value
				: null,
		}
	}

	function syncAnalyticsRouteQuery(navigationMode: AnalyticsQueryBuilderRouteNavigationMode) {
		if (import.meta.server) {
			return
		}

		const nextRouteQuery = buildAnalyticsQueryBuilderRouteQuery(
			route.query,
			getSelectedAnalyticsQueryBuilderState(),
			availableProjectIds.value,
			getSelectedAnalyticsGraphState(),
			defaultProjectIds.value,
		)

		const hasAnalyticsQueryChange = hasAnalyticsQueryBuilderRouteChange(route.query, nextRouteQuery)

		if (!hasAnalyticsQueryChange) return

		if (navigationMode === 'replace') {
			router.replace({
				path: route.path,
				query: nextRouteQuery,
			})
		} else {
			router.push({
				path: route.path,
				query: nextRouteQuery,
			})
		}
	}

	function syncQueryBuilderRouteQuery() {
		syncAnalyticsRouteQuery(consumeAnalyticsRouteNavigationMode())
	}

	function syncGraphRouteQuery() {
		syncAnalyticsRouteQuery('replace')
	}

	function applyRouteQueryToState(nextQuery: LocationQuery) {
		const nextQueryState = readAnalyticsQueryBuilderState(
			nextQuery,
			availableProjectIds.value,
			defaultProjectIds.value,
		)
		const availableProjectIdSet = new Set(availableProjectIds.value)
		const nextSelectedProjectIds = nextQueryState.selectedProjectIds.filter((projectId) =>
			availableProjectIdSet.has(projectId),
		)
		const nextGraphState = readAnalyticsGraphState(nextQuery, nextSelectedProjectIds)
		const nextSelectedBreakdowns = getAnalyticsBreakdownPresetsForProjectSelection(
			nextQueryState.selectedBreakdowns,
			nextSelectedProjectIds,
		)
		const nextSelectedFilters = sanitizeSelectedFilters(
			nextSelectedBreakdowns,
			nextQueryState.selectedFilters,
		)
		const shouldUpdateSelectedProjectIds = !areStringArraysEqual(
			queryBuilder.selectedProjectIds.value,
			nextSelectedProjectIds,
		)
		const shouldUpdateSelectedTimeframeMode =
			queryBuilder.selectedTimeframeMode.value !== nextQueryState.selectedTimeframeMode
		const shouldUpdateSelectedTimeframe =
			queryBuilder.selectedTimeframe.value !== nextQueryState.selectedTimeframe
		const shouldUpdateSelectedLastTimeframeAmount =
			queryBuilder.selectedLastTimeframeAmount.value !== nextQueryState.selectedLastTimeframeAmount
		const shouldUpdateSelectedLastTimeframeUnit =
			queryBuilder.selectedLastTimeframeUnit.value !== nextQueryState.selectedLastTimeframeUnit
		const shouldUpdateSelectedCustomTimeframeStartDate =
			queryBuilder.selectedCustomTimeframeStartDate.value !==
			nextQueryState.selectedCustomTimeframeStartDate
		const shouldUpdateSelectedCustomTimeframeEndDate =
			queryBuilder.selectedCustomTimeframeEndDate.value !==
			nextQueryState.selectedCustomTimeframeEndDate
		const shouldUpdateSelectedGroupBy =
			queryBuilder.selectedGroupBy.value !== nextQueryState.selectedGroupBy
		const shouldUpdateSelectedBreakdowns = !areStringArraysEqual(
			queryBuilder.selectedBreakdowns.value,
			nextSelectedBreakdowns,
		)
		const shouldUpdateSelectedFilters = !areSelectedFiltersEqual(
			queryBuilder.selectedFilters.value,
			nextSelectedFilters,
		)
		const shouldUpdateActiveStat = graph.activeStat.value !== nextGraphState.activeStat
		const shouldUpdateActiveGraphViewMode =
			graph.activeGraphViewMode.value !== nextGraphState.activeGraphViewMode
		const shouldUpdateIsRatioMode = graph.isRatioMode.value !== nextGraphState.isRatioMode
		const shouldUpdateShowChartEvents =
			graph.showChartEvents.value !== nextGraphState.showChartEvents
		const shouldUpdateShowProjectEvents =
			graph.showProjectEvents.value !== nextGraphState.showProjectEvents
		const shouldUpdateShowPreviousPeriod =
			graph.showPreviousPeriod.value !== nextGraphState.showPreviousPeriod
		const shouldUpdateHiddenGraphDatasetIds = !areStringArraysEqual(
			graph.hiddenGraphDatasetIds.value,
			nextGraphState.hiddenGraphDatasetIds,
		)
		const nextHasExplicitGraphDatasetSelection = nextGraphState.selectedGraphDatasetIds !== null
		const nextSelectedGraphDatasetIds = nextGraphState.selectedGraphDatasetIds ?? []
		const shouldUpdateHasExplicitGraphDatasetSelection =
			graph.hasExplicitGraphDatasetSelection.value !== nextHasExplicitGraphDatasetSelection
		const shouldUpdateSelectedGraphDatasetIds =
			(nextHasExplicitGraphDatasetSelection || graph.hasExplicitGraphDatasetSelection.value) &&
			!areStringArraysEqual(graph.selectedGraphDatasetIds.value, nextSelectedGraphDatasetIds)
		const hasRouteStateUpdate =
			shouldUpdateSelectedProjectIds ||
			shouldUpdateSelectedTimeframeMode ||
			shouldUpdateSelectedTimeframe ||
			shouldUpdateSelectedLastTimeframeAmount ||
			shouldUpdateSelectedLastTimeframeUnit ||
			shouldUpdateSelectedCustomTimeframeStartDate ||
			shouldUpdateSelectedCustomTimeframeEndDate ||
			shouldUpdateSelectedGroupBy ||
			shouldUpdateSelectedBreakdowns ||
			shouldUpdateSelectedFilters ||
			shouldUpdateActiveStat ||
			shouldUpdateActiveGraphViewMode ||
			shouldUpdateIsRatioMode ||
			shouldUpdateShowChartEvents ||
			shouldUpdateShowProjectEvents ||
			shouldUpdateShowPreviousPeriod ||
			shouldUpdateHiddenGraphDatasetIds ||
			shouldUpdateHasExplicitGraphDatasetSelection ||
			shouldUpdateSelectedGraphDatasetIds

		if (hasRouteStateUpdate) {
			replaceNextAnalyticsRouteNavigation()
		}

		if (shouldUpdateSelectedProjectIds) {
			queryBuilder.selectedProjectIds.value = nextSelectedProjectIds
		}
		if (shouldUpdateSelectedTimeframeMode) {
			queryBuilder.selectedTimeframeMode.value = nextQueryState.selectedTimeframeMode
		}
		if (shouldUpdateSelectedTimeframe) {
			queryBuilder.selectedTimeframe.value = nextQueryState.selectedTimeframe
		}
		if (shouldUpdateSelectedLastTimeframeAmount) {
			queryBuilder.selectedLastTimeframeAmount.value = nextQueryState.selectedLastTimeframeAmount
		}
		if (shouldUpdateSelectedLastTimeframeUnit) {
			queryBuilder.selectedLastTimeframeUnit.value = nextQueryState.selectedLastTimeframeUnit
		}
		if (shouldUpdateSelectedCustomTimeframeStartDate) {
			queryBuilder.selectedCustomTimeframeStartDate.value =
				nextQueryState.selectedCustomTimeframeStartDate
		}
		if (shouldUpdateSelectedCustomTimeframeEndDate) {
			queryBuilder.selectedCustomTimeframeEndDate.value =
				nextQueryState.selectedCustomTimeframeEndDate
		}
		if (shouldUpdateSelectedGroupBy) {
			queryBuilder.selectedGroupBy.value = nextQueryState.selectedGroupBy
		}
		if (shouldUpdateSelectedBreakdowns) {
			queryBuilder.selectedBreakdowns.value = nextSelectedBreakdowns
		}
		if (shouldUpdateSelectedFilters) {
			queryBuilder.selectedFilters.value = nextSelectedFilters
		}
		if (shouldUpdateActiveStat) {
			graph.activeStat.value = nextGraphState.activeStat
		}
		if (shouldUpdateActiveGraphViewMode) {
			graph.activeGraphViewMode.value = nextGraphState.activeGraphViewMode
		}
		if (shouldUpdateIsRatioMode) {
			graph.isRatioMode.value = nextGraphState.isRatioMode
		}
		if (shouldUpdateShowChartEvents) {
			graph.showChartEvents.value = nextGraphState.showChartEvents
		}
		if (shouldUpdateShowProjectEvents) {
			graph.showProjectEvents.value = nextGraphState.showProjectEvents
		}
		if (shouldUpdateShowPreviousPeriod) {
			graph.showPreviousPeriod.value = nextGraphState.showPreviousPeriod
		}
		if (shouldUpdateHiddenGraphDatasetIds) {
			graph.hiddenGraphDatasetIds.value = nextGraphState.hiddenGraphDatasetIds
		}
		if (shouldUpdateHasExplicitGraphDatasetSelection) {
			graph.hasExplicitGraphDatasetSelection.value = nextHasExplicitGraphDatasetSelection
		}
		if (shouldUpdateSelectedGraphDatasetIds) {
			graph.selectedGraphDatasetIds.value = nextSelectedGraphDatasetIds
		}

		if (!hasRouteStateUpdate) {
			syncAnalyticsRouteQuery('replace')
		}
	}

	return {
		replaceNextAnalyticsRouteNavigation,
		syncQueryBuilderRouteQuery,
		syncGraphRouteQuery,
		applyRouteQueryToState,
	}
}
