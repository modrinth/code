<template>
	<AnalyticsChartRenderLimitModal
		ref="showAllSelectedGraphDatasetsModal"
		:table-project-count="tableProjectCount"
		@confirm="confirmShowAllSelectedGraphDatasets"
	/>

	<section
		ref="graphSection"
		class="relative flex flex-col rounded-2xl border border-solid border-surface-5 bg-surface-3"
		:style="graphSectionStyle"
	>
		<AnalyticsChartHeader
			v-model:active-graph-view-mode="activeGraphViewMode"
			v-model:ratio-mode="isRatioMode"
			v-model:show-chart-events="showChartEvents"
			v-model:show-project-events="showProjectEvents"
			v-model:show-previous-period="showPreviousPeriod"
			:graph-title="graphTitle"
			:show-table-selection-subheading="showTableSelectionSubheading"
			:table-selection-subheading="tableSelectionSubheading"
			:show-graph-render-limit-button="showGraphRenderLimitButton"
			:graph-render-limit-button-label="graphRenderLimitButtonLabel"
			:show-top-graph-datasets-button="showTopGraphDatasetsButton"
			:can-use-ratio-mode="canUseRatioMode"
			:can-show-previous-period="canShowPreviousPeriodToggle"
			:is-same-day-last-week-comparison="isSameDayLastWeekComparison"
			:has-chart-events="hasChartEvents"
			:has-project-events="hasProjectEvents"
			:small-toggles="!isMobileLayout"
			:default-show-project-events="defaultShowProjectEvents"
			:is-mobile-layout="isMobileLayout"
			@toggle-graph-render-limit="toggleGraphRenderLimit"
			@show-top-graph-datasets="showTopGraphDatasets"
		/>

		<div
			class="flex flex-col gap-6 px-4 pb-6 pt-5"
			:class="['transition-opacity', isDataLoading ? 'pointer-events-none opacity-75' : '']"
		>
			<AnalyticsChartLegend
				:legend-entries="legendEntries"
				:should-capitalize-dataset-labels="shouldCapitalizeDatasetLabels"
				:show-unmonetized-info="showUnmonetizedInfo"
				@entry-hover="setHoveredLegendEntryId"
				@entry-hover-clear="clearHoveredLegendEntryId"
				@entry-click="onLegendEntryClick"
			/>

			<AnalyticsChartPlot
				:chart-type="chartType"
				:is-area="isArea"
				:is-stacked="isStacked"
				:is-ratio-mode="isRatioMode"
				:is-data-loading="isDataLoading"
				:show-empty-chart-state="showEmptyChartState"
				:empty-chart-message="emptyChartMessage"
				:visible-chart-datasets="visibleChartDatasets"
				:chart-labels="chartLabels"
				:x-axis-tick-limit="xAxisTickLimit"
				:active-stat="activeStat"
				:highlighted-chart-dataset-id="highlightedChartDatasetId"
				:has-visible-timeline-events="hasVisibleTimelineEvents"
				:visible-timeline-events="visibleTimelineEvents"
				:selected-group-by="selectedGroupBy"
				:chart-range-bounds="chartRangeBounds"
				:fetch-request="fetchRequest"
				:slice-count="sliceCount"
				:should-show-previous-period="shouldShowPreviousPeriod"
				:is-same-day-last-week-comparison="isSameDayLastWeekComparison"
				:all-chart-datasets="allChartDatasets"
				:current-legend-entries="currentLegendEntries"
				:legend-entries="legendEntries"
				:chart-dataset-by-id="chartDatasetById"
				:hover-ratio-slice-totals="hoverRatioSliceTotals"
				:should-capitalize-dataset-labels="shouldCapitalizeDatasetLabels"
				@range-select="onRangeSelect"
				@entry-click="onTooltipEntryClick"
				@entry-hover="setHoveredLegendEntryId"
				@entry-hover-clear="clearHoveredLegendEntryId"
			/>
		</div>

		<div class="pointer-events-none absolute inset-0 z-[20] overflow-hidden rounded-xl">
			<AnalyticsLoadingBar :loading="isDataLoading" />
		</div>

		<div v-if="isDataLoading" class="absolute inset-0 z-[19] overflow-hidden rounded-xl">
			<div class="absolute inset-0 bg-surface-3 opacity-50" />
			<div class="absolute inset-0 backdrop-blur-[3px]" />
			<div class="absolute inset-0 flex items-center justify-center">
				<div
					class="relative bottom-6 inline-flex items-center gap-2 text-lg font-semibold text-primary"
				>
					<span>{{ formatMessage(analyticsMessages.fetchingResults) }}</span>
				</div>
			</div>
		</div>
	</section>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'

import { getDefaultAnalyticsGraphProjectEventsVisibility } from '~/components/analytics-dashboard/analytics-route-query'
import type { AnalyticsGroupByPreset } from '~/providers/analytics/analytics'
import { injectAnalyticsDashboardContext } from '~/providers/analytics/analytics'

import { analyticsMessages } from '../analytics-messages.ts'
import AnalyticsLoadingBar from '../AnalyticsLoadingBar.vue'
import AnalyticsChartLegend from './analytics-chart-header/AnalyticsChartLegend.vue'
import AnalyticsChartRenderLimitModal from './analytics-chart-header/AnalyticsChartRenderLimitModal.vue'
import AnalyticsChartHeader from './analytics-chart-header/index.vue'
import { useAnalyticsChartLegend } from './analytics-chart-header/use-analytics-chart-legend.ts'
import AnalyticsChartPlot from './analytics-chart-plot/index.vue'
import { useAnalyticsChartEvents } from './analytics-chart-plot/use-analytics-chart-events.ts'
import { useAnalyticsChartLayout } from './analytics-chart-plot/use-analytics-chart-layout.ts'
import { useAnalyticsChartDatasets } from './use-analytics-chart-datasets.ts'
import { useAnalyticsChartProjects } from './use-analytics-chart-projects.ts'

const dashboardContext = injectAnalyticsDashboardContext()
const { formatMessage } = useVIntl()
const {
	activeStat,
	activeGraphViewMode,
	isRatioMode,
	showChartEvents,
	showProjectEvents,
	showPreviousPeriod,
	isMobileLayout,
	hiddenGraphDatasetIds,
	isGraphDatasetSelectionActive,
	selectedProjectIds: currentSelectedProjectIds,
	selectedTimeframeMode,
	selectedTimeframe,
	selectedCustomTimeframeStartDate,
	selectedCustomTimeframeEndDate,
	selectedGroupBy: selectedDashboardGroupBy,
	displayedFetchRequest: fetchRequest,
	displayedSelectedGroupBy: selectedGroupBy,
	displayedSelectedBreakdowns: selectedBreakdowns,
	isLoading,
} = dashboardContext

const isDataLoading = computed(() => isLoading.value)
const isSameDayLastWeekComparison = computed(
	() => selectedTimeframeMode.value === 'preset' && selectedTimeframe.value === 'yesterday',
)
const defaultShowProjectEvents = computed(() =>
	getDefaultAnalyticsGraphProjectEventsVisibility(currentSelectedProjectIds.value),
)

const {
	selectedProjectIdSet,
	hasAvailableProjects,
	selectedProjects,
	selectedProjectNameById,
	selectedProjectEventIdSet,
} = useAnalyticsChartProjects(dashboardContext)
const {
	showAllSelectedGraphDatasets,
	chartRangeBounds,
	tableProjectCount,
	showEmptyChartState,
	emptyChartMessage,
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
	allChartDatasets,
	previousChartDatasets,
	tableSelectionSubheading,
	showGraphRenderLimitButton,
	graphRenderLimitButtonLabel,
	showTopGraphDatasetsButton,
	selectableChartDatasets,
	showTopGraphDatasets,
} = useAnalyticsChartDatasets(dashboardContext, selectedProjects, hasAvailableProjects)
const {
	currentLegendEntries,
	visibleProjectEventIdSet,
	legendEntries,
	chartDatasetById,
	hoverRatioSliceTotals,
	visibleChartDatasets,
	highlightedChartDatasetId,
	setHoveredLegendEntryId,
	clearHoveredLegendEntryId,
	onLegendEntryClick,
	onTooltipEntryClick,
} = useAnalyticsChartLegend({
	selectableChartDatasets,
	allChartDatasets,
	previousChartDatasets,
	shouldShowPreviousPeriod,
	isSameDayLastWeekComparison,
	isRatioMode,
	hiddenGraphDatasetIds,
	selectedBreakdowns,
	isGraphDatasetSelectionActive,
	selectedProjects,
	selectedProjectIdSet,
	selectedProjectEventIdSet,
})
const { hasChartEvents, hasProjectEvents, visibleTimelineEvents, hasVisibleTimelineEvents } =
	useAnalyticsChartEvents(
		dashboardContext,
		chartRangeBounds,
		selectedProjectNameById,
		selectedProjectEventIdSet,
		visibleProjectEventIdSet,
	)
const { graphSection, graphSectionStyle } = useAnalyticsChartLayout(showEmptyChartState)

const showAllSelectedGraphDatasetsModal = ref<InstanceType<
	typeof AnalyticsChartRenderLimitModal
> | null>(null)
const canUseRatioMode = computed(
	() =>
		(activeGraphViewMode.value === 'area' || activeGraphViewMode.value === 'bar') &&
		legendEntries.value.length > 1,
)
const showUnmonetizedInfo = computed(
	() => selectedBreakdowns.value.length === 1 && selectedBreakdowns.value[0] === 'monetization',
)

function toggleGraphRenderLimit(event: MouseEvent) {
	if (showAllSelectedGraphDatasets.value) {
		showAllSelectedGraphDatasets.value = false
		return
	}

	showAllSelectedGraphDatasetsModal.value?.show(event)
}

function confirmShowAllSelectedGraphDatasets() {
	showAllSelectedGraphDatasets.value = true
}

function onRangeSelect(start: Date, end: Date, groupBy: AnalyticsGroupByPreset) {
	selectedTimeframeMode.value = 'custom_datetime_range'
	selectedCustomTimeframeStartDate.value = start.toISOString()
	selectedCustomTimeframeEndDate.value = end.toISOString()
	selectedDashboardGroupBy.value = groupBy
}

watch(canUseRatioMode, (canUse) => {
	if (!canUse) {
		isRatioMode.value = false
	}
})
</script>
