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
					<div class="text-xl font-semibold text-contrast">Breakdown</div>

					<div class="flex w-full flex-wrap items-center gap-3 md:w-auto">
						<StyledInput
							v-model="searchQuery"
							:icon="SearchIcon"
							placeholder="Search..."
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
								Export CSV
								<DropdownIcon />
								<template #cumulative-csv> Cumulative </template>
								<template #grouped-csv> Grouped by {{ groupByLabel.toLowerCase() }} </template>
							</OverflowMenu>
						</ButtonStyled>
					</div>
				</div>
			</template>

			<template #cell-date="{ value }">
				<span class="text-primary">{{ value }}</span>
			</template>
			<template #cell-breakdown="{ row }">
				<span
					class="text-primary"
					:class="{
						capitalize: shouldCapitalizeBreakdownDisplay,
					}"
					>{{ row.breakdownDisplay }}</span
				>
			</template>
			<template #cell-project="{ value }">
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
				Showing {{ visibleRowStart }} to {{ visibleRowEnd }} of {{ filteredRows.length }}
			</span>
			<Pagination :page="currentPage" :count="pageCount" @switch-page="switchPage" />
		</div>
		<div v-if="isDataLoading" class="absolute inset-0 z-10 overflow-hidden rounded-xl">
			<div class="absolute inset-0 bg-surface-3 opacity-50" />
			<div class="absolute inset-0 backdrop-blur-[4px]" />
			<div class="absolute inset-0 flex h-full max-h-[500px] items-center justify-center pt-10">
				<div class="inline-flex items-center gap-2 text-lg font-semibold text-primary opacity-100">
					<span>Fetching results...</span>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, DropdownIcon, SearchIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	OverflowMenu,
	type OverflowMenuOption,
	Pagination,
	StyledInput,
	Table,
	type TableColumn,
	useFormatNumber,
} from '@modrinth/ui'
import type { LocationQuery } from 'vue-router'

import {
	type AnalyticsBreakdownPreset,
	type AnalyticsDashboardStat,
	doesAnalyticsPointMatchNormalizedFilters,
	doesProjectStatusMatchFilters,
	injectAnalyticsDashboardContext,
	normalizeAnalyticsSelectedFilters,
} from '~/providers/analytics/analytics'
import {
	type AnalyticsTableSortColumn as TableColumnKey,
	type AnalyticsTableSortDirection as SortDirection,
	areStringArraysEqual,
	buildAnalyticsTableSortRouteQuery,
	hasAnalyticsTableSortQuery,
	hasAnalyticsTableSortRouteChange,
	readAnalyticsTableSortState,
} from '~/providers/analytics/query-builder-url'

import AnalyticsLoadingBar from '../AnalyticsLoadingBar.vue'
import { ALL_BREAKDOWN_VALUE, getAnalyticsBreakdownValue } from '../breakdown'
import {
	formatBreakdownLabel,
	formatBucketEndLabel,
	getSliceBucketRange,
	getSliceCount,
	isTimeRelevantForGroupBy,
	isYearRelevantForTimeRange,
	shouldCapitalizeBreakdownLabel,
} from '../graph/utils'

type TableMode = 'date_breakdown' | 'breakdown_only'

type AnalyticsTableRow = {
	id: string
	date: string
	dateMs: number
	project: string
	breakdown: string
	graphDatasetId: string
	breakdownDisplay: string
	views: number
	downloads: number
	revenue: number
	playtime: number
}

type DisplayedRowsCache = {
	generation: number
	mode: TableMode
	sortColumn: TableColumnKey | undefined
	sortDirection: SortDirection
	rows: AnalyticsTableRow[]
}

const {
	hasProjectContext,
	projects,
	displayedSelectedProjectIds: selectedProjectIds,
	displayedSelectedGroupBy: selectedGroupBy,
	displayedSelectedBreakdown: selectedBreakdown,
	displayedSelectedFilters: selectedFilters,
	displayedFetchRequest: fetchRequest,
	displayedTimeSlices: timeSlices,
	activeStat,
	hasExplicitGraphDatasetSelection,
	isGraphDatasetSelectionActive,
	selectedGraphDatasetIds,
	getRelevantAnalyticsDashboardStats,
	isLoading,
	versionNumbersById,
	versionProjectNamesById,
	getVersionDisplayName,
	getVersionProjectName,
} = injectAnalyticsDashboardContext()
const formatNumber = useFormatNumber()
const isDataLoading = computed(() => isLoading.value)
const route = useRoute()
const router = useRouter()
const initialTableSortState = readAnalyticsTableSortState(route.query, {
	sortColumn: 'date',
	sortDirection: 'desc',
})

const tableMode = ref<TableMode>('breakdown_only')
const sortColumn = ref<TableColumnKey | undefined>(initialTableSortState.sortColumn)
const sortDirection = ref<SortDirection>(initialTableSortState.sortDirection)
const displayedSortColumn = ref<TableColumnKey | undefined>(initialTableSortState.sortColumn)
const displayedSortDirection = ref<SortDirection>(initialTableSortState.sortDirection)
const PAGE_SIZE = 500
const GRAPH_DATASET_SELECTION_LIMIT = 8
const INACTIVE_MODE_WARMUP_POINT_LIMIT = 12000
const ALL_PROJECTS_DATASET_ID = 'all'
const ALL_PROJECTS_BREAKDOWN_VALUE = 'all'
const SECONDS_PER_MINUTE = 60
const SECONDS_PER_HOUR = 60 * SECONDS_PER_MINUTE
const SEARCHABLE_COLUMN_KEYS = new Set<TableColumnKey>(['date', 'project', 'breakdown'])
const currentPage = ref(1)
const searchQuery = ref('')
const sortCollator = new Intl.Collator(undefined, { sensitivity: 'base' })
const modeBuildRequestIds: Record<TableMode, number> = {
	date_breakdown: 0,
	breakdown_only: 0,
}
const selectDefaultGraphDatasetsWhenSelectionEmpties = ref(false)
let tableCacheGeneration = 0
let displayedSortedRowsGeneration = 0
const displayedTableMode = ref<TableMode>('breakdown_only')
const displayedSortedRows = shallowRef<AnalyticsTableRow[]>([])
const displayedRowsCache = shallowRef<DisplayedRowsCache | null>(null)

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
const showBreakdownColumn = computed(() => selectedBreakdown.value !== 'none')
const showGraphDatasetSelection = computed(() =>
	selectedBreakdown.value === 'project'
		? selectedProjectIdSet.value.size > 1
		: selectedBreakdown.value !== 'none',
)
const showProjectVersionProjectColumn = computed(
	() => selectedBreakdown.value === 'version_id' && selectedProjectIdSet.value.size > 1,
)
const includeDateColumn = computed(
	() =>
		selectedBreakdown.value === 'none' ||
		(!showGraphDatasetSelection.value && tableMode.value === 'date_breakdown'),
)
const activeTableMode = computed<TableMode>(() =>
	selectedBreakdown.value === 'none'
		? 'date_breakdown'
		: showGraphDatasetSelection.value
			? 'breakdown_only'
			: tableMode.value,
)
const displayedIncludeDateColumn = computed(() =>
	selectedBreakdown.value === 'none'
		? true
		: showGraphDatasetSelection.value
			? false
			: displayedTableMode.value === 'date_breakdown',
)
const groupByLabel = computed(() => getGroupByLabel(selectedGroupBy.value))
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
const projectNamesById = computed(
	() => new Map(projects.value.map((project) => [project.id, project.name])),
)
const hasAvailableProjects = computed(() => projects.value.length > 0)
const analyticsPointCount = computed(() =>
	timeSlices.value.reduce((sum, slice) => sum + slice.length, 0),
)
const emptyTableMessage = computed(() => {
	if (trimmedSearchQuery.value && sortedRows.value.length > 0) {
		return 'No matching analytics rows'
	}

	if (hasProjectContext.value) {
		return 'No data available for analytics'
	}

	return hasAvailableProjects.value ? 'No data available' : 'No projects available for analytics'
})

const breakdownColumnLabel = computed(() => {
	switch (selectedBreakdown.value) {
		case 'none':
			return 'Project'
		case 'project':
			return 'Project'
		case 'country':
			return 'Country'
		case 'monetization':
			return 'Monetization'
		case 'user_agent':
			return 'Download source'
		case 'download_reason':
			return 'Download reason'
		case 'version_id':
			return 'Project version'
		case 'loader':
			return 'Loader'
		case 'game_version':
			return 'Game version'
		default:
			return 'Breakdown'
	}
})
const relevantStats = computed(
	() => new Set(getRelevantAnalyticsDashboardStats(selectedBreakdown.value, selectedFilters.value)),
)
const shouldCapitalizeBreakdownDisplay = computed(() =>
	shouldCapitalizeBreakdownLabel(selectedBreakdown.value),
)

const showTimeInBucketLabel = computed(() => isTimeRelevantForGroupBy(selectedGroupBy.value))
const showYearInBucketLabel = computed(() => {
	const nextFetchRequest = fetchRequest.value
	return nextFetchRequest
		? isYearRelevantForTimeRange(nextFetchRequest.time_range) || selectedGroupBy.value === 'year'
		: false
})

function buildTableRows(mode: TableMode): AnalyticsTableRow[] {
	const nextFetchRequest = fetchRequest.value
	const nextTimeSlices = timeSlices.value
	const nextSelectedBreakdown = selectedBreakdown.value
	const nextSelectedProjectIds = selectedProjectIdSet.value
	const nextRelevantStats = relevantStats.value
	const normalizedFilters = normalizeAnalyticsSelectedFilters(selectedFilters.value)

	if (!nextFetchRequest || nextSelectedProjectIds.size === 0) {
		return []
	}

	const timeRange = nextFetchRequest.time_range
	const sliceCount = getSliceCount(timeRange, nextTimeSlices.length)
	const includeDate = mode === 'date_breakdown'
	const breakdownDisplayValues = new Map<string, string>()
	const projectDisplayValues = new Map<string, string>()
	const nextRows = new Map<string, AnalyticsTableRow>()
	const bucketLabelsBySliceIndex = new Map<number, { date: string; dateMs: number }>()

	function getBreakdownDisplayValue(breakdown: string) {
		let displayValue = breakdownDisplayValues.get(breakdown)
		if (displayValue === undefined) {
			displayValue = formatBreakdownDisplayValue(breakdown)
			breakdownDisplayValues.set(breakdown, displayValue)
		}
		return displayValue
	}

	function getProjectDisplayValueForBreakdown(breakdown: string) {
		let displayValue = projectDisplayValues.get(breakdown)
		if (displayValue === undefined) {
			displayValue = getProjectDisplayValue(breakdown, nextSelectedBreakdown)
			projectDisplayValues.set(breakdown, displayValue)
		}
		return displayValue
	}

	function getBucketLabel(sliceIndex: number) {
		let bucketLabel = bucketLabelsBySliceIndex.get(sliceIndex)
		if (!bucketLabel) {
			const bucketRange = getSliceBucketRange(timeRange, sliceCount, sliceIndex)
			bucketLabel = {
				date: formatBucketEndLabel(
					bucketRange.end,
					showTimeInBucketLabel.value,
					showYearInBucketLabel.value,
				),
				dateMs: bucketRange.end.getTime(),
			}
			bucketLabelsBySliceIndex.set(sliceIndex, bucketLabel)
		}
		return bucketLabel
	}

	function createRow(
		rowId: string,
		breakdown: string,
		bucketLabel?: { date: string; dateMs: number },
	) {
		const row = {
			id: rowId,
			date: bucketLabel?.date ?? '',
			dateMs: bucketLabel?.dateMs ?? 0,
			project: getProjectDisplayValueForBreakdown(breakdown),
			breakdown,
			graphDatasetId: getGraphDatasetId(breakdown, nextSelectedBreakdown),
			breakdownDisplay: getBreakdownDisplayValue(breakdown),
			views: 0,
			downloads: 0,
			revenue: 0,
			playtime: 0,
		}
		nextRows.set(rowId, row)
		return row
	}

	if (!includeDate && nextSelectedBreakdown === 'none') {
		createRow(ALL_PROJECTS_BREAKDOWN_VALUE, ALL_PROJECTS_BREAKDOWN_VALUE)
	}

	if (!includeDate && nextSelectedBreakdown === 'project') {
		for (const projectId of nextSelectedProjectIds) {
			createRow(projectId, projectId)
		}
	}

	nextTimeSlices.forEach((slice, sliceIndex) => {
		const bucketLabel = includeDate ? getBucketLabel(sliceIndex) : undefined

		for (const point of slice) {
			if (!isProjectAnalyticsPoint(point)) {
				continue
			}

			if (!nextSelectedProjectIds.has(point.source_project)) {
				continue
			}

			if (!doesAnalyticsPointMatchNormalizedFilters(point, normalizedFilters)) {
				continue
			}

			const pointStat = getStatForMetric(point.metric_kind)
			if (!pointStat || !nextRelevantStats.has(pointStat)) {
				continue
			}

			const breakdown =
				nextSelectedBreakdown === 'none'
					? ALL_PROJECTS_BREAKDOWN_VALUE
					: nextSelectedBreakdown === 'project'
						? point.source_project
						: getBreakdownValue(point, nextSelectedBreakdown)
			if (nextSelectedBreakdown !== 'none' && breakdown === ALL_BREAKDOWN_VALUE) {
				continue
			}

			const nextBucketLabel = includeDate ? (bucketLabel ?? getBucketLabel(sliceIndex)) : undefined
			const rowId = includeDate ? `${nextBucketLabel?.dateMs ?? 0}::${breakdown}` : breakdown
			const row = nextRows.get(rowId) ?? createRow(rowId, breakdown, nextBucketLabel)
			addMetricToRow(row, point)
		}
	})

	return Array.from(nextRows.values())
}

function isProjectAnalyticsPoint(
	point: Labrinth.Analytics.v3.AnalyticsData,
): point is Labrinth.Analytics.v3.ProjectAnalytics {
	return 'source_project' in point
}

const columns = computed<TableColumn<TableColumnKey>[]>(() =>
	buildColumns(displayedIncludeDateColumn.value),
)
const activeColumns = computed<TableColumn<TableColumnKey>[]>(() =>
	buildColumns(includeDateColumn.value),
)

function buildColumns(includeDate: boolean): TableColumn<TableColumnKey>[] {
	const nextColumns: TableColumn<TableColumnKey>[] = []

	const stats = getRelevantAnalyticsDashboardStats(selectedBreakdown.value, selectedFilters.value)

	if (includeDate) {
		nextColumns.push({
			key: 'date',
			label: 'Date',
			enableSorting: true,
			defaultSortDirection: 'desc',
			width: stats.length > 2 ? '20%' : '',
		})
	}

	if (showBreakdownColumn.value) {
		nextColumns.push({
			key: 'breakdown',
			label: breakdownColumnLabel.value,
			enableSorting: true,
		})
	}

	if (showProjectVersionProjectColumn.value) {
		nextColumns.push({
			key: 'project',
			label: 'Project',
			enableSorting: true,
		})
	}

	for (const stat of stats) {
		const column = getMetricColumnForStat(stat)
		if (column) {
			nextColumns.push(column)
		}
	}

	return nextColumns
}

watch(
	activeColumns,
	(nextColumns) => {
		applyRouteOrDefaultSort(nextColumns)
	},
	{ immediate: true },
)

const sortedRows = computed<AnalyticsTableRow[]>(() => {
	return displayedSortedRows.value
})
const trimmedSearchQuery = computed(() => searchQuery.value.trim().toLowerCase())
const searchableColumns = computed(() =>
	columns.value.filter((column) => SEARCHABLE_COLUMN_KEYS.has(column.key)),
)
const filteredRows = computed<AnalyticsTableRow[]>(() => {
	if (!trimmedSearchQuery.value) {
		return sortedRows.value
	}

	return filterRowsBySearch(sortedRows.value, searchableColumns.value)
})
const selectableGraphDatasetIds = computed(() => getSelectableGraphDatasetIds(sortedRows.value))
const filteredSelectableGraphDatasetIds = computed(() =>
	getSelectableGraphDatasetIds(filteredRows.value),
)
const tableSelectedGraphDatasetIds = computed({
	get: () => selectedGraphDatasetIds.value,
	set: (ids: string[]) => {
		selectedGraphDatasetIds.value = ids
		if (showGraphDatasetSelection.value) {
			hasExplicitGraphDatasetSelection.value = true
		}
	},
})

function setSelectedGraphDatasetIds(ids: string[], explicit: boolean) {
	selectedGraphDatasetIds.value = ids
	hasExplicitGraphDatasetSelection.value = explicit
}

watch(
	showGraphDatasetSelection,
	(nextShowSelection) => {
		isGraphDatasetSelectionActive.value = nextShowSelection
		if (!nextShowSelection) {
			setSelectedGraphDatasetIds([], false)
		}
	},
	{ immediate: true },
)

watch(activeStat, () => {
	if (!showGraphDatasetSelection.value) {
		return
	}
	if (hasAnalyticsTableSortQuery(route.query)) {
		return
	}

	applyActiveStatSort()
})

watch(selectedBreakdown, () => {
	setSelectedGraphDatasetIds([], false)
})

watch(
	[selectedProjectIds, selectedFilters],
	() => {
		selectDefaultGraphDatasetsWhenSelectionEmpties.value = true
	},
	{ deep: true },
)

watch(
	[selectableGraphDatasetIds, showGraphDatasetSelection, hasExplicitGraphDatasetSelection],
	(
		[nextSelectableGraphDatasetIds, nextShowGraphDatasetSelection, nextHasExplicitSelection],
		previousValues,
	) => {
		if (!nextShowGraphDatasetSelection) {
			selectDefaultGraphDatasetsWhenSelectionEmpties.value = false
			return
		}

		const previousSelectableGraphDatasetIds = previousValues?.[0] ?? []
		const didSelectableGraphDatasetsChange = !areStringArraysEqual(
			previousSelectableGraphDatasetIds,
			nextSelectableGraphDatasetIds,
		)
		const nextSelectableGraphDatasetIdSet = new Set(nextSelectableGraphDatasetIds)
		const nextSelectedGraphDatasetIds = selectedGraphDatasetIds.value.filter((datasetId) =>
			nextSelectableGraphDatasetIdSet.has(datasetId),
		)

		if (nextHasExplicitSelection) {
			if (
				selectDefaultGraphDatasetsWhenSelectionEmpties.value &&
				didSelectableGraphDatasetsChange &&
				nextSelectedGraphDatasetIds.length === 0 &&
				nextSelectableGraphDatasetIds.length > 0
			) {
				selectDefaultGraphDatasetsWhenSelectionEmpties.value = false
				setSelectedGraphDatasetIds(
					getDefaultSelectedGraphDatasetIds(nextSelectableGraphDatasetIds),
					false,
				)
				return
			}

			if (didSelectableGraphDatasetsChange) {
				selectDefaultGraphDatasetsWhenSelectionEmpties.value = false
			}

			if (!areStringArraysEqual(selectedGraphDatasetIds.value, nextSelectedGraphDatasetIds)) {
				setSelectedGraphDatasetIds(nextSelectedGraphDatasetIds, true)
			}
			return
		}

		selectDefaultGraphDatasetsWhenSelectionEmpties.value = false
		const defaultSelectedGraphDatasetIds = getDefaultSelectedGraphDatasetIds(
			nextSelectableGraphDatasetIds,
		)
		if (!areStringArraysEqual(selectedGraphDatasetIds.value, defaultSelectedGraphDatasetIds)) {
			setSelectedGraphDatasetIds(defaultSelectedGraphDatasetIds, false)
		}
	},
	{ immediate: true },
)

watch(
	[
		fetchRequest,
		timeSlices,
		selectedProjectIds,
		selectedGroupBy,
		selectedBreakdown,
		selectedFilters,
		projects,
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
	currentPage.value = 1
	scheduleRowsForMode(activeTableMode.value)
	scheduleInactiveModeWarmup()
})

watch(
	() => route.query,
	(nextQuery) => {
		const nextSortState = getRouteTableSortState(nextQuery, activeColumns.value)
		if (!isCurrentSortState(nextSortState)) {
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

const pageCount = computed(() => Math.max(Math.ceil(filteredRows.value.length / PAGE_SIZE), 1))
const visibleRowStart = computed(() =>
	filteredRows.value.length === 0 ? 0 : (currentPage.value - 1) * PAGE_SIZE + 1,
)
const visibleRowEnd = computed(() =>
	Math.min(currentPage.value * PAGE_SIZE, filteredRows.value.length),
)
const paginatedRows = computed<AnalyticsTableRow[]>(() =>
	filteredRows.value.slice((currentPage.value - 1) * PAGE_SIZE, currentPage.value * PAGE_SIZE),
)

watch(filteredRows, () => {
	currentPage.value = 1
})

watch(pageCount, (nextPageCount) => {
	if (currentPage.value > nextPageCount) {
		currentPage.value = nextPageCount
	}
})

function invalidateTableCaches() {
	tableCacheGeneration++
	invalidateSortedCaches()
}

function invalidateSortedCaches() {
	displayedRowsCache.value = null
}

function hasSortedRowsForMode(mode: TableMode): boolean {
	const cached = displayedRowsCache.value
	return (
		cached !== null &&
		cached.generation === tableCacheGeneration &&
		cached.mode === mode &&
		cached.sortColumn === sortColumn.value &&
		cached.sortDirection === sortDirection.value
	)
}

function setDisplayedRowsForMode(
	mode: TableMode,
	rows: AnalyticsTableRow[],
	generation = tableCacheGeneration,
) {
	displayedRowsCache.value = {
		generation,
		mode,
		sortColumn: sortColumn.value,
		sortDirection: sortDirection.value,
		rows,
	}

	if (mode === activeTableMode.value) {
		displayedSortedRowsGeneration = generation
		displayedTableMode.value = mode
		displayedSortColumn.value = sortColumn.value
		displayedSortDirection.value = sortDirection.value
		displayedSortedRows.value = rows
	}
}

function scheduleRowsForMode(mode: TableMode) {
	if (hasSortedRowsForMode(mode)) {
		if (mode === activeTableMode.value) {
			displayRowsForMode(mode)
		}
		return
	}

	const requestId = ++modeBuildRequestIds[mode]
	const generation = tableCacheGeneration

	void buildRowsForMode(mode, generation, requestId)
}

function displayRowsForMode(mode: TableMode) {
	const cached = displayedRowsCache.value
	if (!cached || cached.generation !== tableCacheGeneration || cached.mode !== mode) {
		return
	}

	displayedSortedRowsGeneration = cached.generation
	displayedTableMode.value = mode
	displayedSortColumn.value = cached.sortColumn
	displayedSortDirection.value = cached.sortDirection
	displayedSortedRows.value = cached.rows
}

async function buildRowsForMode(mode: TableMode, generation: number, requestId: number) {
	await waitForDeferredTableWork()

	if (isStaleBuild(mode, generation, requestId)) {
		return
	}

	const rows = sortTableRows(buildTableRows(mode))

	if (isStaleBuild(mode, generation, requestId)) {
		return
	}

	setDisplayedRowsForMode(mode, rows, generation)
}

function isStaleBuild(mode: TableMode, generation: number, requestId: number): boolean {
	return tableCacheGeneration !== generation || modeBuildRequestIds[mode] !== requestId
}

function waitForDeferredTableWork(): Promise<void> {
	if (!import.meta.client) {
		return Promise.resolve()
	}

	return new Promise((resolve) => {
		requestAnimationFrame(() => {
			requestAnimationFrame(() => resolve())
		})
	})
}

function scheduleInactiveModeWarmup() {
	if (!showBreakdownColumn.value) {
		return
	}
	if (analyticsPointCount.value > INACTIVE_MODE_WARMUP_POINT_LIMIT) {
		return
	}

	const inactiveMode: TableMode =
		activeTableMode.value === 'date_breakdown' ? 'breakdown_only' : 'date_breakdown'

	if (hasSortedRowsForMode(inactiveMode)) {
		return
	}

	if (!import.meta.client) {
		scheduleRowsForMode(inactiveMode)
		return
	}

	const windowWithIdleCallback = window as Window & {
		requestIdleCallback?: (callback: () => void, options?: { timeout?: number }) => number
	}

	if (windowWithIdleCallback.requestIdleCallback) {
		windowWithIdleCallback.requestIdleCallback(() => scheduleRowsForMode(inactiveMode), {
			timeout: 2000,
		})
	} else {
		window.setTimeout(() => scheduleRowsForMode(inactiveMode), 250)
	}
}

function sortTableRows(rows: AnalyticsTableRow[]): AnalyticsTableRow[] {
	const nextRows = [...rows]
	const activeSortColumn = sortColumn.value

	if (!activeSortColumn) {
		return nextRows
	}

	const directionFactor = sortDirection.value === 'asc' ? 1 : -1
	nextRows.sort(getRowComparator(activeSortColumn, directionFactor))

	return nextRows
}

function filterRowsBySearch(
	rows: AnalyticsTableRow[],
	searchableColumns: TableColumn<TableColumnKey>[],
): AnalyticsTableRow[] {
	const query = trimmedSearchQuery.value
	if (!query || searchableColumns.length === 0) {
		return rows
	}

	return rows.filter((row) =>
		searchableColumns.some((column) =>
			String(getSearchableCellValue(row, column.key)).toLowerCase().includes(query),
		),
	)
}

function getSearchableCellValue(row: AnalyticsTableRow, key: TableColumnKey): string {
	switch (key) {
		case 'date':
			return row.date
		case 'project':
			return row.project
		case 'breakdown':
			return row.breakdownDisplay
		default:
			return ''
	}
}

function resortDisplayedRowsForCurrentSort(): boolean {
	const mode = activeTableMode.value
	if (displayedTableMode.value !== mode || displayedSortedRowsGeneration !== tableCacheGeneration) {
		return false
	}

	setDisplayedRowsForMode(mode, sortTableRows(displayedSortedRows.value))
	return true
}

const revenueFormatter = computed(
	() =>
		new Intl.NumberFormat(undefined, {
			minimumFractionDigits: 2,
			maximumFractionDigits: 2,
		}),
)

function formatInteger(value: number): string {
	return formatNumber(Math.round(value))
}

function formatRevenue(value: number): string {
	const rounded = Math.round(value * 100) / 100
	return `$${revenueFormatter.value.format(rounded)}`
}

function formatCompactPlaytime(value: number): string {
	const totalSeconds = Math.max(0, Math.round(value))
	return `${(totalSeconds / SECONDS_PER_HOUR).toLocaleString(undefined, {
		minimumFractionDigits: 1,
		maximumFractionDigits: 1,
	})}h`
}

function formatFullPlaytime(value: number): string {
	const totalMinutes = Math.max(0, Math.round(value / SECONDS_PER_MINUTE))
	const days = Math.floor(totalMinutes / (24 * 60))
	const hours = Math.floor((totalMinutes % (24 * 60)) / 60)
	const minutes = totalMinutes % 60

	return [
		formatDurationTooltipPart(days, 'day'),
		formatDurationTooltipPart(hours, 'hour'),
		formatDurationTooltipPart(minutes, 'minute'),
	].join(', ')
}

function formatDurationTooltipPart(value: number, unit: string): string {
	return `${formatNumber(value)} ${unit}${value === 1 ? '' : 's'}`
}

function addMetricToRow(row: AnalyticsTableRow, point: Labrinth.Analytics.v3.ProjectAnalytics) {
	switch (point.metric_kind) {
		case 'views':
			row.views += point.views
			break
		case 'downloads':
			row.downloads += point.downloads
			break
		case 'playtime':
			row.playtime += point.seconds
			break
		case 'revenue': {
			const parsed = Number.parseFloat(point.revenue)
			row.revenue += Number.isFinite(parsed) ? parsed : 0
			break
		}
	}
}

function getStatForMetric(
	metricKind: Labrinth.Analytics.v3.ProjectAnalytics['metric_kind'],
): AnalyticsDashboardStat | null {
	switch (metricKind) {
		case 'views':
		case 'downloads':
		case 'revenue':
		case 'playtime':
			return metricKind
		default:
			return null
	}
}

function getMetricColumnForStat(stat: AnalyticsDashboardStat): TableColumn<TableColumnKey> | null {
	switch (stat) {
		case 'views':
			return {
				key: 'views',
				label: 'Views',
				enableSorting: true,
				defaultSortDirection: 'desc',
				align: 'right',
			}
		case 'downloads':
			return {
				key: 'downloads',
				label: 'Downloads',
				enableSorting: true,
				defaultSortDirection: 'desc',
				align: 'right',
			}
		case 'revenue':
			return {
				key: 'revenue',
				label: 'Revenue',
				enableSorting: true,
				defaultSortDirection: 'desc',
				align: 'right',
			}
		case 'playtime':
			return {
				key: 'playtime',
				label: 'Playtime',
				enableSorting: true,
				defaultSortDirection: 'desc',
				align: 'right',
			}
		default:
			return null
	}
}

function applyRouteOrDefaultSort(nextColumns = activeColumns.value) {
	const nextSortState = getRouteTableSortState(route.query, nextColumns)
	if (!isCurrentSortState(nextSortState)) {
		applyTableSortState(nextSortState)
	}

	syncTableSortRouteQuery()
}

function applyTableSortState(state: {
	sortColumn: TableColumnKey | undefined
	sortDirection: SortDirection
}) {
	sortColumn.value = state.sortColumn
	sortDirection.value = state.sortDirection
}

function getRouteTableSortState(
	query: LocationQuery,
	nextColumns = activeColumns.value,
): { sortColumn: TableColumnKey | undefined; sortDirection: SortDirection } {
	return getAvailableTableSortState(
		readAnalyticsTableSortState(query, getDefaultTableSortState(nextColumns)),
		nextColumns,
	)
}

function getAvailableTableSortState(
	state: { sortColumn: TableColumnKey | undefined; sortDirection: SortDirection },
	nextColumns = activeColumns.value,
): { sortColumn: TableColumnKey | undefined; sortDirection: SortDirection } {
	const availableColumns = new Set(nextColumns.map((column) => column.key))
	if (state.sortColumn && availableColumns.has(state.sortColumn)) {
		return state
	}

	return getDefaultTableSortState(nextColumns)
}

function getDefaultTableSortState(nextColumns = activeColumns.value): {
	sortColumn: TableColumnKey | undefined
	sortDirection: SortDirection
} {
	const nextSortColumn = getDefaultSortColumn(nextColumns)
	return {
		sortColumn: nextSortColumn,
		sortDirection: getDefaultSortDirection(nextSortColumn, nextColumns),
	}
}

function isCurrentSortState(state: {
	sortColumn: TableColumnKey | undefined
	sortDirection: SortDirection
}): boolean {
	return sortColumn.value === state.sortColumn && sortDirection.value === state.sortDirection
}

function syncTableSortRouteQuery() {
	if (import.meta.server) {
		return
	}

	const nextSortState = getAvailableTableSortState(
		{
			sortColumn: sortColumn.value,
			sortDirection: sortDirection.value,
		},
		activeColumns.value,
	)
	const nextRouteQuery = buildAnalyticsTableSortRouteQuery(
		route.query,
		nextSortState,
		getDefaultTableSortState(),
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

function applyRequestedSort(column: string, direction: SortDirection) {
	sortColumn.value = column as TableColumnKey
	sortDirection.value = direction
}

function switchPage(page: number) {
	currentPage.value = page
}

function selectSearchInputText(event: FocusEvent) {
	const target = event.target
	if (target instanceof HTMLInputElement) {
		target.select()
	}
}

function getDefaultSortColumn(
	nextColumns: TableColumn<TableColumnKey>[],
): TableColumnKey | undefined {
	const availableColumns = new Set(nextColumns.map((column) => column.key))
	if (availableColumns.has('date')) {
		return 'date'
	}

	if (showGraphDatasetSelection.value && availableColumns.has(activeStat.value)) {
		return activeStat.value
	}

	if (availableColumns.has('downloads')) {
		return 'downloads'
	}

	return nextColumns[0]?.key
}

function getDefaultSortDirection(
	column: TableColumnKey | undefined,
	nextColumns: TableColumn<TableColumnKey>[],
): SortDirection {
	return nextColumns.find((nextColumn) => nextColumn.key === column)?.defaultSortDirection ?? 'asc'
}

function getBreakdownValue(
	point: Labrinth.Analytics.v3.ProjectAnalytics,
	selectedBreakdown: AnalyticsBreakdownPreset,
): string {
	return getAnalyticsBreakdownValue(point, selectedBreakdown)
}

function getGraphDatasetId(breakdown: string, selectedBreakdown: AnalyticsBreakdownPreset): string {
	if (selectedBreakdown === 'none') {
		return ALL_PROJECTS_DATASET_ID
	}
	if (selectedBreakdown === 'project') {
		return breakdown
	}

	return `breakdown:${breakdown}`
}

function getDefaultSelectedGraphDatasetIds(ids: string[]): string[] {
	return ids.length > GRAPH_DATASET_SELECTION_LIMIT
		? ids.slice(0, GRAPH_DATASET_SELECTION_LIMIT)
		: ids
}

function getSelectableGraphDatasetIds(rows: AnalyticsTableRow[]): string[] {
	return Array.from(new Set(rows.map((row) => row.graphDatasetId)))
}

function getGroupByLabel(groupBy: string): string {
	switch (groupBy) {
		case '1h':
			return '1h'
		case '6h':
			return '6h'
		case 'day':
			return 'Day'
		case 'week':
			return 'Week'
		case 'month':
			return 'Month'
		case 'year':
			return 'Year'
		default:
			return 'Date'
	}
}

function getRowComparator(
	column: TableColumnKey,
	directionFactor: number,
): (left: AnalyticsTableRow, right: AnalyticsTableRow) => number {
	switch (column) {
		case 'date':
			return (left, right) => compareRows(left, right, left.dateMs - right.dateMs, directionFactor)
		case 'project':
			return (left, right) =>
				compareRows(left, right, sortCollator.compare(left.project, right.project), directionFactor)
		case 'breakdown':
			return (left, right) =>
				compareRows(
					left,
					right,
					sortCollator.compare(left.breakdownDisplay, right.breakdownDisplay),
					directionFactor,
				)
		case 'views':
			return (left, right) => compareRows(left, right, left.views - right.views, directionFactor)
		case 'downloads':
			return (left, right) =>
				compareRows(left, right, left.downloads - right.downloads, directionFactor)
		case 'revenue':
			return (left, right) =>
				compareRows(left, right, left.revenue - right.revenue, directionFactor)
		case 'playtime':
			return (left, right) =>
				compareRows(left, right, left.playtime - right.playtime, directionFactor)
		default:
			return () => 0
	}
}

function compareRows(
	left: AnalyticsTableRow,
	right: AnalyticsTableRow,
	primaryResult: number,
	directionFactor: number,
): number {
	if (primaryResult !== 0) {
		return primaryResult * directionFactor
	}

	const dateResult = left.dateMs - right.dateMs
	if (dateResult !== 0) {
		return dateResult * directionFactor
	}

	return sortCollator.compare(left.breakdown, right.breakdown) * directionFactor
}

function formatBreakdownDisplayValue(value: string): string {
	if (selectedBreakdown.value === 'project') {
		return projectNamesById.value.get(value) ?? value
	}
	return formatBreakdownLabel(value, selectedBreakdown.value, getVersionDisplayName)
}

function getProjectDisplayValue(
	breakdown: string,
	selectedBreakdown: AnalyticsBreakdownPreset,
): string {
	if (selectedBreakdown !== 'version_id') {
		return ''
	}

	return getVersionProjectName(breakdown) ?? ''
}

function getCsvCellValue(row: AnalyticsTableRow, key: TableColumnKey): string | number {
	switch (key) {
		case 'date':
			return row.date
		case 'project':
			return row.project
		case 'breakdown':
			return row.breakdownDisplay
		case 'views':
			return row.views
		case 'downloads':
			return row.downloads
		case 'revenue':
			return row.revenue
		case 'playtime':
			return row.playtime
		default:
			return ''
	}
}

function getCsvHeaderLabel(column: TableColumn<TableColumnKey>): string {
	if (column.key === 'playtime') {
		return 'Playtime (seconds)'
	}

	return column.label ?? column.key
}

function escapeCsvField(value: string | number): string {
	const stringValue = String(value)
	if (
		stringValue.includes(',') ||
		stringValue.includes('"') ||
		stringValue.includes('\n') ||
		stringValue.includes('\r')
	) {
		return `"${stringValue.replace(/"/g, '""')}"`
	}
	return stringValue
}

function getCsvRows(mode: TableMode): AnalyticsTableRow[] {
	const displayedCache = displayedRowsCache.value
	const visibleColumns = getCsvColumns(mode)
	if (
		displayedCache &&
		displayedCache.generation === tableCacheGeneration &&
		displayedCache.mode === mode &&
		displayedCache.sortColumn === sortColumn.value &&
		displayedCache.sortDirection === sortDirection.value
	) {
		return filterRowsBySearch(displayedCache.rows, visibleColumns)
	}

	return filterRowsBySearch(sortTableRows(buildTableRows(mode)), visibleColumns)
}

function getCsvColumns(mode: TableMode): TableColumn<TableColumnKey>[] {
	return buildColumns(mode === 'date_breakdown' || !showBreakdownColumn.value)
}

function formatCsvFilenameDate(date: Date): string {
	return date.toLocaleDateString(undefined, {
		year: 'numeric',
		month: 'short',
		day: 'numeric',
	})
}

function getCsvFilenameDateRange(): string {
	const timeRange = fetchRequest.value?.time_range
	if (!timeRange) {
		return 'Selected Range'
	}

	const start = new Date(timeRange.start)
	const end = new Date(timeRange.end)
	if (Number.isNaN(start.getTime()) || Number.isNaN(end.getTime())) {
		return 'Selected Range'
	}

	const startLabel = formatCsvFilenameDate(start)
	const endLabel = formatCsvFilenameDate(end)
	return startLabel === endLabel ? startLabel : `${startLabel} to ${endLabel}`
}

function sanitizeCsvFilename(value: string): string {
	return value
		.replace(/[<>:"/\\|?*]/g, '')
		.replace(/\s+/g, ' ')
		.trim()
}

function getCsvFilename(): string {
	return `${sanitizeCsvFilename(
		`Modrinth Analytics ${breakdownColumnLabel.value} Breakdown - ${getCsvFilenameDateRange()}`,
	)}.csv`
}

function downloadCsv(mode: TableMode) {
	if (!import.meta.client) {
		return
	}

	const csvRows = getCsvRows(mode)
	if (csvRows.length === 0) {
		return
	}

	const visibleColumns = getCsvColumns(mode)
	const header = visibleColumns.map((column) => escapeCsvField(getCsvHeaderLabel(column))).join(',')

	const rows = csvRows.map((row) =>
		visibleColumns.map((column) => escapeCsvField(getCsvCellValue(row, column.key))).join(','),
	)

	const csvContent = [header, ...rows].join('\n')
	const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
	const url = URL.createObjectURL(blob)

	const downloadLink = document.createElement('a')
	downloadLink.setAttribute('href', url)
	downloadLink.setAttribute('download', getCsvFilename())
	downloadLink.style.visibility = 'hidden'

	document.body.appendChild(downloadLink)
	downloadLink.click()
	document.body.removeChild(downloadLink)

	URL.revokeObjectURL(url)
}
</script>
