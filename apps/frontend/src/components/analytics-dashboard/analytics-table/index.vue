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
			<template #cell-breakdown_project="{ row, value }">
				<span v-if="isGroupedBreakdown('project')" class="mr-2.5 text-primary">{{ value }}</span>
				<ProjectCell
					v-else
					:label="getProjectCellLabel(value)"
					:icon-url="getProjectIconUrl(row.breakdownValues.project)"
					:icon-tooltip="getProjectCellLabel(value)"
					:label-href="getProjectPageHref(row.breakdownValues.project)"
					:organization-href="getProjectOrganizationPageHref(row.breakdownValues.project)"
					:organization-tooltip="getProjectOrganizationName(row.breakdownValues.project)"
				/>
			</template>
			<template #cell-breakdown_country="{ value }">
				<span class="mr-2.5 text-primary">{{ value }}</span>
			</template>
			<template #cell-breakdown_monetization="{ value }">
				<span class="mr-2.5 text-primary">{{ value }}</span>
			</template>
			<template #cell-breakdown_user_agent="{ value }">
				<span class="mr-2.5 text-primary">{{ value }}</span>
			</template>
			<template #cell-breakdown_download_reason="{ value }">
				<span class="mr-2.5 text-primary">{{ value }}</span>
			</template>
			<template #cell-breakdown_user_id="{ row, value }">
				<span v-if="isGroupedBreakdown('user_id')" class="mr-2.5 text-primary">{{ value }}</span>
				<component
					:is="getUserPageHref(row.breakdownValues.user_id) ? 'a' : 'span'"
					v-else
					:href="getUserPageHref(row.breakdownValues.user_id)"
					:target="getUserPageHref(row.breakdownValues.user_id) ? '_blank' : undefined"
					:rel="getUserPageHref(row.breakdownValues.user_id) ? 'noopener noreferrer' : undefined"
					class="mr-2.5 flex min-w-0 items-center gap-2 text-primary"
					:class="{ 'hover:underline': getUserPageHref(row.breakdownValues.user_id) }"
				>
					<span
						v-tooltip="getUserCellLabel(value)"
						class="flex size-6 shrink-0 items-center justify-center overflow-hidden rounded-full text-primary"
					>
						<img
							v-if="getUserAvatarUrl(row.breakdownValues.user_id)"
							:src="getUserAvatarUrl(row.breakdownValues.user_id)"
							:alt="getUserCellLabel(value)"
							class="h-6 w-6 rounded-full object-cover"
						/>
						<UserIcon v-else class="h-full w-full" />
					</span>
					<span class="min-w-0 truncate font-semibold leading-tight text-primary">
						{{ value }}
					</span>
				</component>
			</template>
			<template #cell-breakdown_dependent_project_download="{ row, value }">
				<span v-if="isGroupedBreakdown('dependent_project_download')" class="mr-2.5 text-primary">
					{{ value }}
				</span>
				<ProjectCell
					v-else
					:label="getProjectCellLabel(value)"
					:icon-url="
						isMissingDependentProjectValue(row.breakdownValues.dependent_project_download)
							? undefined
							: getProjectIconUrl(row.breakdownValues.dependent_project_download)
					"
					:icon-tooltip="getProjectCellLabel(value)"
					:hide-icon="
						isMissingDependentProjectValue(row.breakdownValues.dependent_project_download)
					"
					:label-href="
						isMissingDependentProjectValue(row.breakdownValues.dependent_project_download)
							? undefined
							: getProjectPageHref(row.breakdownValues.dependent_project_download)
					"
					:label-tooltip="getDependentProjectTooltip(row)"
				/>
			</template>
			<template #cell-breakdown_version_id="{ row, value }">
				<span v-if="isGroupedBreakdown('version_id')" class="mr-2.5 text-primary">{{ value }}</span>
				<component
					:is="getVersionPageHref(row.projectVersionId) ? 'a' : 'span'"
					v-else
					v-tooltip="getVersionProjectName(row.projectVersionId)"
					:href="getVersionPageHref(row.projectVersionId)"
					:target="getVersionPageHref(row.projectVersionId) ? '_blank' : undefined"
					:rel="getVersionPageHref(row.projectVersionId) ? 'noopener noreferrer' : undefined"
					class="mr-2.5 text-primary"
					:class="{ 'hover:underline': getVersionPageHref(row.projectVersionId) }"
				>
					{{ value }}
				</component>
			</template>
			<template #cell-breakdown_loader="{ value }">
				<span class="mr-2.5 text-primary">{{ value }}</span>
			</template>
			<template #cell-breakdown_game_version="{ value }">
				<span class="mr-2.5 text-primary">{{ value }}</span>
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
import { DownloadIcon, DropdownIcon, SearchIcon, UserIcon } from '@modrinth/assets'
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
	analyticsChartMessages,
	analyticsMessages,
	analyticsTableMessages,
} from '../analytics-messages.ts'
import AnalyticsLoadingBar from '../AnalyticsLoadingBar.vue'
import {
	isNoDependentAnalyticsBreakdownValue,
	isUnknownAnalyticsBreakdownValue,
} from '../breakdown.ts'
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
	AnalyticsTableRow,
	AnalyticsTableSortDirectionValue,
} from './analytics-table-types.ts'
import ProjectCell from './ProjectCell.vue'
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
	resolvedActiveBreakdownGroup,
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
	versionProjectIdsById,
	versionProjectNamesById,
	projectNamesById,
	projectIconUrlsById,
	projectOrganizationIdsById,
	projectOrganizationNamesById,
	userNamesById,
	userAvatarUrlsById,
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
const includeDependentProjectTooltipContext = computed(
	() =>
		selectedBreakdownSet.value.has('dependent_project_download') &&
		!selectedBreakdownSet.value.has('project'),
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
	resolvedActiveBreakdownGroup.value
		? resolvedActiveBreakdownGroup.value.name
		: selectedBreakdowns.value.length === 1
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
		breakdownGroup: resolvedActiveBreakdownGroup.value,
		selectedProjectIds: selectedProjectIdSet.value,
		selectedFilters: selectedFilters.value,
		dependentProjectTypesById: dependentProjectTypesById.value,
		includeDependentProjectTooltipContext: includeDependentProjectTooltipContext.value,
		relevantStats: relevantStats.value,
		projectNamesById: projectNamesById.value,
		userNamesById: userNamesById.value,
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
		breakdownGroup: resolvedActiveBreakdownGroup.value,
		formatMessage,
		getRelevantAnalyticsDashboardStats,
	})
}

function getProjectIconUrl(projectId: string | undefined) {
	return projectId ? projectIconUrlsById.value.get(projectId) : undefined
}

function isGroupedBreakdown(breakdown: string) {
	return resolvedActiveBreakdownGroup.value?.breakdown === breakdown
}

function getProjectOrganizationName(projectId: string | undefined) {
	return projectId ? projectOrganizationNamesById.value.get(projectId) : undefined
}

function getProjectCellLabel(value: unknown) {
	return typeof value === 'string' ? value : String(value ?? '')
}

function getUserAvatarUrl(userId: string | undefined) {
	return userId ? userAvatarUrlsById.value.get(userId) : undefined
}

function getUserCellLabel(value: unknown) {
	return typeof value === 'string' ? value : String(value ?? '')
}

function getUserPageHref(userId: string | undefined) {
	if (!userId) return undefined
	const username = userNamesById.value.get(userId) ?? userId

	return `/user/${encodeURIComponent(username)}`
}

function getProjectPageHref(projectId: string | undefined) {
	return projectId ? `/project/${encodeURIComponent(projectId)}` : undefined
}

function getProjectOrganizationPageHref(projectId: string | undefined) {
	if (!projectId) return undefined
	const organizationId = projectOrganizationIdsById.value.get(projectId)
	if (!organizationId) return undefined

	return `/organization/${encodeURIComponent(organizationId)}`
}

function getVersionPageHref(versionId: string | undefined) {
	if (!versionId) return undefined
	const projectId = versionProjectIdsById.value.get(versionId)
	if (!projectId) return undefined

	return `/project/${encodeURIComponent(projectId)}/version/${encodeURIComponent(versionId)}`
}

function isMissingDependentProjectValue(value: string | undefined) {
	return isUnknownAnalyticsBreakdownValue(value) || isNoDependentAnalyticsBreakdownValue(value)
}

function getDependentProjectTooltip(row: AnalyticsTableRow) {
	if (isNoDependentAnalyticsBreakdownValue(row.breakdownValues.dependent_project_download)) {
		return formatMessage(analyticsMessages.noDependentTooltip)
	}
	if (isUnknownAnalyticsBreakdownValue(row.breakdownValues.dependent_project_download)) {
		return formatMessage(analyticsMessages.unknownDependentTooltip)
	}

	const dependencyProjectIds = new Set(row.dependentOnProjectIds)
	if (row.dependentOnProjectId) {
		dependencyProjectIds.add(row.dependentOnProjectId)
	}

	const dependencyProjectNames = [...dependencyProjectIds]
		.map((projectId) => projectNamesById.value.get(projectId) ?? projectId)
		.sort((left, right) => left.localeCompare(right))

	return dependencyProjectNames.length > 0
		? formatMessage(analyticsChartMessages.dependentOnProjectTooltip, {
				project: dependencyProjectNames.join(', '),
			})
		: undefined
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
		resolvedActiveBreakdownGroup,
		selectedFilters,
		projects,
		dependentProjectTypesById,
		projectNamesById,
		userNamesById,
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
