<template>
	<div class="relative overflow-hidden rounded-2xl">
		<AnalyticsLoadingBar :loading="isDataLoading" />
		<Table
			v-model:selected-ids="tableSelectedGraphDatasetIds"
			:sort-column="displayedSortColumn"
			:sort-direction="displayedSortDirection"
			:columns="columns"
			:data="paginatedRows"
			row-key="id"
			selection-key="graphDatasetId"
			:selection-ids="filteredSelectableGraphDatasetIds"
			:show-selection="showGraphDatasetSelection"
			table-min-width="44rem"
			virtualized
			:virtual-row-height="56"
			@sort="applyRequestedSort"
		>
			<template #header>
				<div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
					<div class="text-xl font-semibold text-contrast">
						{{ formatMessage(analyticsBreakdownMessages.breakdown) }}
					</div>

					<div class="flex w-full flex-wrap items-center gap-2 md:w-auto">
						<StyledInput
							v-model="searchQuery"
							:icon="SearchIcon"
							:placeholder="formatMessage(analyticsTableMessages.searchPlaceholder)"
							clearable
							wrapper-class="w-full sm:w-64"
							@focusin="selectSearchInputText"
						/>
						<ButtonStyled>
							<OverflowMenu
								class="!shadow-none"
								:options="csvExportOptions"
								:disabled="isDataLoading || filteredRows.length === 0"
							>
								<DownloadIcon />
								{{ formatMessage(analyticsTableMessages.exportCsvButton) }}
								<DropdownIcon />
								<template #cumulative-csv>
									{{ formatMessage(analyticsTableMessages.cumulativeCsv) }}
								</template>
								<template #grouped-csv>
									{{ formatMessage(analyticsTableMessages.groupedCsv, { groupBy: groupByLabel }) }}
								</template>
							</OverflowMenu>
						</ButtonStyled>
					</div>
				</div>
			</template>

			<template #cell-date="{ value }">
				<span class="text-primary">{{ value }}</span>
			</template>
			<template #cell-breakdown_project="{ value }">
				<span class="text-primary">{{ value }}</span>
			</template>
			<template #cell-breakdown_country="{ value }">
				<span class="text-primary">{{ value }}</span>
			</template>
			<template #cell-breakdown_monetization="{ value }">
				<span class="text-primary">{{ value }}</span>
			</template>
			<template #cell-breakdown_user_agent="{ value }">
				<span class="text-primary">{{ value }}</span>
			</template>
			<template #cell-breakdown_download_reason="{ value }">
				<span class="text-primary">{{ value }}</span>
			</template>
			<template #cell-breakdown_dependent_project_download="{ value }">
				<span class="text-primary">{{ value }}</span>
			</template>
			<template #cell-breakdown_version_id="{ value }">
				<span class="text-primary">{{ value }}</span>
			</template>
			<template #cell-breakdown_loader="{ value }">
				<span class="text-primary">{{ value }}</span>
			</template>
			<template #cell-breakdown_game_version="{ value }">
				<span class="text-primary">{{ value }}</span>
			</template>
			<template #cell-project="{ value }">
				<span class="text-primary">{{ value }}</span>
			</template>
			<template #cell-dependent_on="{ value }">
				<span class="text-primary">{{ value }}</span>
			</template>
			<template #cell-views="{ row }">
				<span>{{ formatInteger(row.views) }}</span>
			</template>
			<template #cell-downloads="{ row }">
				<span>{{ formatInteger(row.downloads) }}</span>
			</template>
			<template #cell-revenue="{ row }">
				<span>{{ formatRevenue(row.revenue) }}</span>
			</template>
			<template #cell-playtime="{ row }">
				<span v-tooltip="formatFullPlaytime(row.playtime)">
					{{ formatCompactPlaytime(row.playtime) }}
				</span>
			</template>
			<template #empty-state>
				<div class="flex h-64 items-center justify-center text-secondary">
					{{ !isDataLoading ? emptyTableMessage : '' }}
				</div>
			</template>
		</Table>
		<div
			v-if="filteredRows.length > PAGE_SIZE"
			class="mt-3 flex flex-wrap items-center justify-between gap-3 px-1 text-sm text-secondary"
		>
			<span>
				{{
					formatMessage(analyticsTableMessages.paginationSummary, {
						start: visibleRowStart,
						end: visibleRowEnd,
						total: filteredRows.length,
					})
				}}
			</span>
			<Pagination :page="currentPage" :count="pageCount" @switch-page="switchPage" />
		</div>
		<div v-if="isDataLoading" class="absolute inset-0 z-10 overflow-hidden rounded-xl">
			<div class="absolute inset-0 bg-surface-3 opacity-50" />
			<div class="absolute inset-0 backdrop-blur-[4px]" />
			<div class="absolute inset-0 flex h-full max-h-[500px] items-center justify-center pt-10">
				<div class="inline-flex items-center gap-2 text-lg font-semibold text-primary opacity-100">
					<span>{{ formatMessage(analyticsMessages.fetchingResults) }}</span>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { DownloadIcon, DropdownIcon, SearchIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	OverflowMenu,
	type OverflowMenuOption,
	Pagination,
	StyledInput,
	Table,
	useFormatNumber,
	useVIntl,
} from '@modrinth/ui'
import type { LocationQuery } from 'vue-router'

import {
	hasAnalyticsTableSortQuery,
	hasAnalyticsTableSortRouteChange,
	readAnalyticsTableSortState,
} from '~/components/analytics-dashboard/analytics-route-query'
import {
	doesProjectStatusMatchFilters,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

import {
	isTimeRelevantForGroupBy,
	isYearRelevantForTimeRange,
} from '../analytics-chart/analytics-chart-utils.ts'
import {
	analyticsBreakdownMessages,
	analyticsMessages,
	analyticsTableMessages,
} from '../analytics-messages.ts'
import AnalyticsLoadingBar from '../AnalyticsLoadingBar.vue'
import {
	buildAnalyticsTableColumns,
	getAnalyticsTableBreakdownColumnLabel,
} from './analytics-table-columns.ts'
import {
	buildAnalyticsTableCsvContent,
	downloadAnalyticsTableCsv,
	getAnalyticsTableCsvFilename,
} from './analytics-table-csv-export.ts'
import {
	formatAnalyticsTableCompactPlaytime,
	formatAnalyticsTableFullPlaytime,
	formatAnalyticsTableInteger,
	formatAnalyticsTableRevenue,
	getAnalyticsTableGroupByLabel,
} from './analytics-table-formatting.ts'
import { buildAnalyticsTableRows } from './analytics-table-row-builder.ts'
import {
	filterAnalyticsTableRowsBySearch,
	getAnalyticsTableSearchableColumns,
} from './analytics-table-search-filtering.ts'
import {
	areAnalyticsTableSortStatesEqual,
	buildSyncedAnalyticsTableSortRouteQuery,
	getRouteAnalyticsTableSortState,
	toAnalyticsTableSortState,
} from './analytics-table-sort-route.ts'
import { sortAnalyticsTableRows } from './analytics-table-sorting.ts'
import type {
	AnalyticsTableColumnKey,
	AnalyticsTableMode,
	AnalyticsTableSortDirectionValue,
} from './analytics-table-types.ts'
import { useAnalyticsTableGraphSelection } from './use-analytics-table-graph-selection.ts'
import { useAnalyticsTablePagination } from './use-analytics-table-pagination.ts'
import { useAnalyticsTableRowCache } from './use-analytics-table-row-cache.ts'

const {
	hasProjectContext,
	projects,
	selectedProjectIds: currentSelectedProjectIds,
	selectedBreakdowns: currentSelectedBreakdowns,
	displayedSelectedProjectIds: selectedProjectIds,
	displayedSelectedGroupBy: selectedGroupBy,
	displayedSelectedBreakdowns: selectedBreakdowns,
	displayedSelectedFilters: selectedFilters,
	displayedFetchRequest: fetchRequest,
	displayedTimeSlices: timeSlices,
	activeStat,
	hasExplicitGraphDatasetSelection,
	isGraphDatasetSelectionActive,
	selectedGraphDatasetIds,
	defaultGraphDatasetIds,
	topGraphDatasetIds,
	queryResetToken,
	getRelevantAnalyticsDashboardStats,
	isLoading,
	versionNumbersById,
	versionProjectNamesById,
	projectNamesById,
	dependentProjectTypesById,
	getVersionDisplayName,
	getVersionProjectName,
} = injectAnalyticsDashboardContext()
const formatNumber = useFormatNumber()
const { formatMessage } = useVIntl()
const isDataLoading = computed(() => isLoading.value)
const route = useRoute()
const router = useRouter()
const initialTableSortState = readAnalyticsTableSortState(route.query, {
	sortColumn: 'date',
	sortDirection: 'desc',
})

const tableMode = ref<AnalyticsTableMode>('breakdown_only')
const sortColumn = ref<AnalyticsTableColumnKey | undefined>(initialTableSortState.sortColumn)
const sortDirection = ref<AnalyticsTableSortDirectionValue>(initialTableSortState.sortDirection)
const PAGE_SIZE = 500
const GRAPH_DATASET_SELECTION_LIMIT = 8
const INACTIVE_MODE_WARMUP_POINT_LIMIT = 12000
const searchQuery = ref('')
const sortCollator = new Intl.Collator(undefined, { sensitivity: 'base' })

const selectedProjectIdSet = computed(
	() =>
		new Set(
			projects.value
				.filter(
					(project) =>
						selectedProjectIds.value.includes(project.id) &&
						doesProjectStatusMatchFilters(project.status, selectedFilters.value),
				)
				.map((project) => project.id),
		),
)
const selectedBreakdownSet = computed(() => new Set(selectedBreakdowns.value))
const showBreakdownColumn = computed(() => selectedBreakdowns.value.length > 0)
const showGraphDatasetSelection = computed(() =>
	selectedBreakdowns.value.length === 1 && selectedBreakdowns.value[0] === 'project'
		? selectedProjectIdSet.value.size > 1
		: selectedBreakdowns.value.length > 0,
)
const showProjectVersionProjectColumn = computed(
	() =>
		selectedBreakdownSet.value.has('version_id') &&
		!selectedBreakdownSet.value.has('project') &&
		selectedProjectIdSet.value.size > 1,
)
const showDependentOnProjectColumn = computed(
	() =>
		selectedBreakdownSet.value.has('dependent_project_download') &&
		!selectedBreakdownSet.value.has('project') &&
		selectedProjectIdSet.value.size > 1,
)
const includeDateColumn = computed(
	() =>
		selectedBreakdowns.value.length === 0 ||
		(!showGraphDatasetSelection.value && tableMode.value === 'date_breakdown'),
)
const activeTableMode = computed<AnalyticsTableMode>(() =>
	selectedBreakdowns.value.length === 0
		? 'date_breakdown'
		: showGraphDatasetSelection.value
			? 'breakdown_only'
			: tableMode.value,
)
const displayedIncludeDateColumn = computed(() =>
	selectedBreakdowns.value.length === 0
		? true
		: showGraphDatasetSelection.value
			? false
			: displayedTableMode.value === 'date_breakdown',
)
const groupByLabel = computed(() =>
	getAnalyticsTableGroupByLabel(selectedGroupBy.value, formatMessage),
)
const csvExportOptions = computed<OverflowMenuOption[]>(() => {
	if (showGraphDatasetSelection.value) {
		return [
			{
				id: 'cumulative-csv',
				action: () => downloadCsv('breakdown_only'),
			},
			{
				id: 'grouped-csv',
				action: () => downloadCsv('date_breakdown'),
			},
		]
	}

	const mode = displayedTableMode.value

	return [
		{
			id: mode === 'date_breakdown' ? 'grouped-csv' : 'cumulative-csv',
			action: () => downloadCsv(mode),
		},
	]
})
const hasAvailableProjects = computed(() => projects.value.length > 0)
const analyticsPointCount = computed(() =>
	timeSlices.value.reduce((sum, slice) => sum + slice.length, 0),
)
const emptyTableMessage = computed(() => {
	if (trimmedSearchQuery.value && sortedRows.value.length > 0) {
		return formatMessage(analyticsTableMessages.noMatchingRows)
	}

	if (hasProjectContext.value) {
		return formatMessage(analyticsMessages.noDataAvailableForAnalytics)
	}

	return hasAvailableProjects.value
		? formatMessage(analyticsMessages.noDataAvailable)
		: formatMessage(analyticsMessages.noProjectsAvailableForAnalytics)
})

const breakdownColumnLabel = computed(() =>
	selectedBreakdowns.value.length === 1
		? getAnalyticsTableBreakdownColumnLabel(selectedBreakdowns.value[0], formatMessage)
		: formatMessage(analyticsBreakdownMessages.breakdown),
)
const relevantStats = computed(
	() =>
		new Set(getRelevantAnalyticsDashboardStats(selectedBreakdowns.value, selectedFilters.value)),
)

const showTimeInBucketLabel = computed(() => isTimeRelevantForGroupBy(selectedGroupBy.value))
const showYearInBucketLabel = computed(() => {
	const nextFetchRequest = fetchRequest.value
	return nextFetchRequest
		? isYearRelevantForTimeRange(nextFetchRequest.time_range) || selectedGroupBy.value === 'year'
		: false
})

function buildTableRows(mode: AnalyticsTableMode) {
	return buildAnalyticsTableRows({
		mode,
		fetchRequest: fetchRequest.value,
		timeSlices: timeSlices.value,
		selectedBreakdowns: selectedBreakdowns.value,
		selectedProjectIds: selectedProjectIdSet.value,
		selectedFilters: selectedFilters.value,
		dependentProjectTypesById: dependentProjectTypesById.value,
		showDependentOnProjectColumn: showDependentOnProjectColumn.value,
		relevantStats: relevantStats.value,
		projectNamesById: projectNamesById.value,
		getVersionDisplayName,
		getVersionProjectName,
		showTimeInBucketLabel: showTimeInBucketLabel.value,
		showYearInBucketLabel: showYearInBucketLabel.value,
		formatMessage,
	})
}

const columns = computed(() => buildColumns(displayedIncludeDateColumn.value))
const activeColumns = computed(() => buildColumns(includeDateColumn.value))

function buildColumns(includeDate: boolean) {
	return buildAnalyticsTableColumns({
		includeDate,
		selectedBreakdowns: selectedBreakdowns.value,
		selectedFilters: selectedFilters.value,
		showBreakdownColumn: showBreakdownColumn.value,
		showDependentOnProjectColumn: showDependentOnProjectColumn.value,
		showProjectVersionProjectColumn: showProjectVersionProjectColumn.value,
		formatMessage,
		getRelevantAnalyticsDashboardStats,
	})
}

watch(
	activeColumns,
	(nextColumns) => {
		applyRouteOrDefaultSort(nextColumns)
	},
	{ immediate: true },
)

function sortTableRows(rows: ReturnType<typeof buildTableRows>) {
	return sortAnalyticsTableRows(rows, sortColumn.value, sortDirection.value, sortCollator)
}

const {
	displayedTableMode,
	displayedSortColumn,
	displayedSortDirection,
	displayedSortedRows,
	invalidateTableCaches,
	invalidateSortedCaches,
	scheduleRowsForMode,
	scheduleInactiveModeWarmup,
	resortDisplayedRowsForCurrentSort,
	getSortedRowsForMode,
} = useAnalyticsTableRowCache({
	activeTableMode,
	showBreakdownColumn,
	analyticsPointCount,
	sortColumn,
	sortDirection,
	buildRows: buildTableRows,
	sortRows: sortTableRows,
	inactiveModeWarmupPointLimit: INACTIVE_MODE_WARMUP_POINT_LIMIT,
})

const sortedRows = computed(() => {
	return displayedSortedRows.value
})
const trimmedSearchQuery = computed(() => searchQuery.value.trim().toLowerCase())
const searchableColumns = computed(() => getAnalyticsTableSearchableColumns(columns.value))
const filteredRows = computed(() => {
	if (!trimmedSearchQuery.value) {
		return sortedRows.value
	}

	return filterAnalyticsTableRowsBySearch(
		sortedRows.value,
		searchableColumns.value,
		trimmedSearchQuery.value,
	)
})

watch(
	[
		fetchRequest,
		timeSlices,
		selectedProjectIds,
		selectedGroupBy,
		selectedBreakdowns,
		selectedFilters,
		projects,
		dependentProjectTypesById,
		projectNamesById,
		versionNumbersById,
		versionProjectNamesById,
	],
	() => {
		invalidateTableCaches()
		scheduleRowsForMode(activeTableMode.value)
		scheduleInactiveModeWarmup()
	},
	{ immediate: true, flush: 'post' },
)

watch(activeTableMode, () => {
	scheduleRowsForMode(activeTableMode.value)
	scheduleInactiveModeWarmup()
})

watch(
	() => route.query,
	(nextQuery) => {
		const nextSortState = getRouteTableSortState(nextQuery, activeColumns.value)
		if (!areAnalyticsTableSortStatesEqual(getCurrentSortState(), nextSortState)) {
			applyTableSortState(nextSortState)
			return
		}

		syncTableSortRouteQuery()
	},
)

watch([sortColumn, sortDirection], () => {
	syncTableSortRouteQuery()

	if (resortDisplayedRowsForCurrentSort()) {
		scheduleInactiveModeWarmup()
		return
	}

	invalidateSortedCaches()
	scheduleRowsForMode(activeTableMode.value)
	scheduleInactiveModeWarmup()
})

const { filteredSelectableGraphDatasetIds, tableSelectedGraphDatasetIds } =
	useAnalyticsTableGraphSelection({
		sortedRows,
		filteredRows,
		sortColumn,
		showGraphDatasetSelection,
		selectedGraphDatasetIds,
		hasExplicitGraphDatasetSelection,
		isGraphDatasetSelectionActive,
		defaultGraphDatasetIds,
		topGraphDatasetIds,
		queryResetToken,
		currentSelectedBreakdowns,
		currentSelectedProjectIds,
		activeStat,
		sortCollator,
		hasTableSortQuery: () => hasAnalyticsTableSortQuery(route.query),
		applyActiveStatSort,
		graphDatasetSelectionLimit: GRAPH_DATASET_SELECTION_LIMIT,
	})

const { currentPage, pageCount, visibleRowStart, visibleRowEnd, paginatedRows, switchPage } =
	useAnalyticsTablePagination({
		filteredRows,
		pageSize: PAGE_SIZE,
	})

const revenueFormatter = computed(
	() =>
		new Intl.NumberFormat(undefined, {
			minimumFractionDigits: 2,
			maximumFractionDigits: 2,
		}),
)

function formatInteger(value: number): string {
	return formatAnalyticsTableInteger(formatNumber, value)
}

function formatRevenue(value: number): string {
	return formatAnalyticsTableRevenue(revenueFormatter.value, value, formatMessage)
}

function formatCompactPlaytime(value: number): string {
	return formatAnalyticsTableCompactPlaytime(value, formatMessage)
}

function formatFullPlaytime(value: number): string {
	return formatAnalyticsTableFullPlaytime(value, formatMessage)
}

function applyRouteOrDefaultSort(nextColumns = activeColumns.value) {
	const nextSortState = getRouteTableSortState(route.query, nextColumns)
	if (!areAnalyticsTableSortStatesEqual(getCurrentSortState(), nextSortState)) {
		applyTableSortState(nextSortState)
	}

	syncTableSortRouteQuery()
}

function applyTableSortState(state: {
	sortColumn: AnalyticsTableColumnKey | undefined
	sortDirection: AnalyticsTableSortDirectionValue
}) {
	sortColumn.value = state.sortColumn
	sortDirection.value = state.sortDirection
}

function getRouteTableSortState(
	query: LocationQuery,
	nextColumns = activeColumns.value,
): {
	sortColumn: AnalyticsTableColumnKey | undefined
	sortDirection: AnalyticsTableSortDirectionValue
} {
	return getRouteAnalyticsTableSortState(query, nextColumns, getDefaultSortOptions(nextColumns))
}

function getCurrentSortState() {
	return toAnalyticsTableSortState(sortColumn.value, sortDirection.value)
}

function getDefaultSortOptions(nextColumns = activeColumns.value) {
	return {
		columns: nextColumns,
		showGraphDatasetSelection: showGraphDatasetSelection.value,
		activeStat: activeStat.value,
	}
}

function syncTableSortRouteQuery() {
	if (import.meta.server) {
		return
	}

	const nextRouteQuery = buildSyncedAnalyticsTableSortRouteQuery(
		route.query,
		getCurrentSortState(),
		activeColumns.value,
		getDefaultSortOptions(),
	)

	if (!hasAnalyticsTableSortRouteChange(route.query, nextRouteQuery)) {
		return
	}

	router.replace({
		path: route.path,
		query: nextRouteQuery,
	})
}

function applyActiveStatSort() {
	const availableColumns = new Set(activeColumns.value.map((column) => column.key))
	if (!availableColumns.has(activeStat.value)) {
		return
	}

	sortColumn.value = activeStat.value
	sortDirection.value = 'desc'
}

function applyRequestedSort(column: string, direction: AnalyticsTableSortDirectionValue) {
	sortColumn.value = column as AnalyticsTableColumnKey
	sortDirection.value = direction
}

function selectSearchInputText(event: FocusEvent) {
	const target = event.target
	if (target instanceof HTMLInputElement) {
		target.select()
	}
}

function getCsvRows(mode: AnalyticsTableMode) {
	const visibleColumns = getCsvColumns(mode)
	return filterAnalyticsTableRowsBySearch(
		getSortedRowsForMode(mode),
		visibleColumns,
		trimmedSearchQuery.value,
	)
}

function getCsvColumns(mode: AnalyticsTableMode) {
	return buildColumns(mode === 'date_breakdown' || !showBreakdownColumn.value)
}

function getCsvFilename(): string {
	return getAnalyticsTableCsvFilename(breakdownColumnLabel.value, fetchRequest.value, formatMessage)
}

function downloadCsv(mode: AnalyticsTableMode) {
	if (!import.meta.client) {
		return
	}

	const csvRows = getCsvRows(mode)
	if (csvRows.length === 0) {
		return
	}

	const visibleColumns = getCsvColumns(mode)
	const csvContent = buildAnalyticsTableCsvContent(csvRows, visibleColumns, formatMessage)
	downloadAnalyticsTableCsv(getCsvFilename(), csvContent)
}
</script>
