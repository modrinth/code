<template>
	<div class="relative overflow-hidden rounded-2xl">
		<AnalyticsLoadingBar :loading="isDataLoading" />
		<Table
			:sort-column="displayedSortColumn"
			:sort-direction="displayedSortDirection"
			:columns="columns"
			:data="paginatedRows"
			row-key="id"
			virtualized
			:virtual-row-height="56"
			@sort="applyRequestedSort"
		>
			<template #header>
				<div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
					<div class="text-xl font-semibold text-contrast">Breakdown</div>

					<div class="flex flex-wrap items-center gap-3">
						<div v-if="showDateToggle" class="inline-flex items-center gap-2">
							<label for="date-toggle" class="cursor-pointer text-sm text-secondary">Date</label>
							<Toggle id="date-toggle" v-model="includeDate" small />
						</div>

						<div v-if="showDateToggle" class="mx-1 h-6 w-px bg-surface-5"></div>

						<ButtonStyled>
							<button
								class="!shadow-none"
								:disabled="isDataLoading || sortedRows.length === 0"
								@click="downloadCsv"
							>
								<DownloadIcon />
								Export CSV
							</button>
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
						capitalize: selectedBreakdown === 'monetization',
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
				<span>{{ formatInteger(row.playtime) }}</span>
			</template>
			<template #empty-state>
				<div class="flex h-64 items-center justify-center text-secondary">
					{{ !isDataLoading ? emptyTableMessage : '' }}
				</div>
			</template>
		</Table>
		<div
			v-if="sortedRows.length > PAGE_SIZE"
			class="mt-3 flex flex-wrap items-center justify-between gap-3 px-1 text-sm text-secondary"
		>
			<span>
				Showing {{ visibleRowStart }} to {{ visibleRowEnd }} of {{ sortedRows.length }}
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
import { DownloadIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	Pagination,
	Table,
	type TableColumn,
	Toggle,
	useFormatNumber,
} from '@modrinth/ui'

import {
	type AnalyticsBreakdownPreset,
	type AnalyticsDashboardStat,
	doesAnalyticsPointMatchFilters,
	doesProjectStatusMatchFilters,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

import AnalyticsLoadingBar from '../AnalyticsLoadingBar.vue'
import { ALL_BREAKDOWN_VALUE, getAnalyticsBreakdownValue } from '../breakdown'
import {
	formatBreakdownLabel,
	formatBucketEndLabel,
	getSliceBucketRange,
	getSliceCount,
	isTimeRelevantForGroupBy,
	isYearRelevantForTimeRange,
} from '../graph/utils'

type TableMode = 'date_breakdown' | 'breakdown_only'
type SortDirection = 'asc' | 'desc'
type TableColumnKey =
	| 'date'
	| 'project'
	| 'breakdown'
	| 'views'
	| 'downloads'
	| 'revenue'
	| 'playtime'

type AnalyticsTableRow = {
	id: string
	date: string
	dateMs: number
	project: string
	breakdown: string
	breakdownDisplay: string
	views: number
	downloads: number
	revenue: number
	playtime: number
}

type SortedRowsCache = {
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
	getRelevantAnalyticsDashboardStats,
	isLoading,
	getVersionDisplayName,
	getVersionProjectName,
} = injectAnalyticsDashboardContext()
const formatNumber = useFormatNumber()
const isDataLoading = computed(() => isLoading.value)

const tableMode = ref<TableMode>('breakdown_only')
const sortColumn = ref<TableColumnKey | undefined>('date')
const sortDirection = ref<SortDirection>('asc')
const displayedSortColumn = ref<TableColumnKey | undefined>('date')
const displayedSortDirection = ref<SortDirection>('asc')
const PAGE_SIZE = 500
const currentPage = ref(1)
const sortCollator = new Intl.Collator(undefined, { sensitivity: 'base' })
const tableRowsByMode = shallowRef<Record<TableMode, AnalyticsTableRow[] | null>>({
	date_breakdown: null,
	breakdown_only: null,
})
const sortedRowsByMode = shallowRef<Record<TableMode, SortedRowsCache | null>>({
	date_breakdown: null,
	breakdown_only: null,
})
const modeBuildRequestIds: Record<TableMode, number> = {
	date_breakdown: 0,
	breakdown_only: 0,
}
let tableCacheGeneration = 0
const displayedTableMode = ref<TableMode>('breakdown_only')
const displayedSortedRows = shallowRef<AnalyticsTableRow[]>([])

const includeDate = computed<boolean>({
	get: () => tableMode.value === 'date_breakdown',
	set: (value) => {
		tableMode.value = value ? 'date_breakdown' : 'breakdown_only'
	},
})

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
const isSingleProjectView = computed(() => selectedProjectIdSet.value.size === 1)
const showBreakdownColumn = computed(
	() => selectedBreakdown.value !== 'none' || !isSingleProjectView.value,
)
const showProjectVersionProjectColumn = computed(
	() => selectedBreakdown.value === 'version_id' && selectedProjectIdSet.value.size > 1,
)
const showDateToggle = computed(() => showBreakdownColumn.value)
const includeDateColumn = computed(
	() => tableMode.value === 'date_breakdown' || !showBreakdownColumn.value,
)
const activeTableMode = computed<TableMode>(() =>
	tableMode.value === 'date_breakdown' || !showBreakdownColumn.value
		? 'date_breakdown'
		: 'breakdown_only',
)
const displayedIncludeDateColumn = computed(
	() => displayedTableMode.value === 'date_breakdown' || !showBreakdownColumn.value,
)
const projectNamesById = computed(
	() => new Map(projects.value.map((project) => [project.id, project.name])),
)
const hasAvailableProjects = computed(() => projects.value.length > 0)
const emptyTableMessage = computed(() => {
	if (hasProjectContext.value) {
		return 'No data available for analytics'
	}

	return hasAvailableProjects.value ? 'No data available' : 'No projects available for analytics'
})

const breakdownColumnLabel = computed(() => {
	switch (selectedBreakdown.value) {
		case 'none':
			return 'Project'
		case 'country':
			return 'Country'
		case 'monetization':
			return 'Monetization'
		case 'download_source':
			return 'Download source'
		case 'download_reason':
			return 'Download type'
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

	if (!nextFetchRequest || selectedProjectIdSet.value.size === 0) {
		return []
	}

	const sliceCount = getSliceCount(nextFetchRequest.time_range, nextTimeSlices.length)
	const includeDate = mode === 'date_breakdown' || !showBreakdownColumn.value

	const breakdownValues = new Set<string>()
	if (nextSelectedBreakdown === 'none') {
		for (const projectId of selectedProjectIdSet.value) {
			breakdownValues.add(projectId)
		}
	} else {
		for (const slice of nextTimeSlices) {
			for (const point of slice) {
				if (!('source_project' in point)) continue
				if (!selectedProjectIdSet.value.has(point.source_project)) continue
				if (!doesAnalyticsPointMatchFilters(point, selectedFilters.value)) continue

				const pointStat = getStatForMetric(point.metric_kind)
				if (!pointStat || !relevantStats.value.has(pointStat)) continue

				const breakdownValue = getBreakdownValue(point, nextSelectedBreakdown)
				if (breakdownValue === ALL_BREAKDOWN_VALUE) continue
				breakdownValues.add(breakdownValue)
			}
		}
	}

	if (breakdownValues.size === 0) {
		return []
	}

	const breakdownDisplayValues = new Map<string, string>()
	const projectDisplayValues = new Map<string, string>()

	for (const breakdown of breakdownValues) {
		breakdownDisplayValues.set(breakdown, formatBreakdownDisplayValue(breakdown))
		projectDisplayValues.set(breakdown, getProjectDisplayValue(breakdown, nextSelectedBreakdown))
	}

	const nextRows = new Map<string, AnalyticsTableRow>()

	if (includeDate) {
		for (let sliceIndex = 0; sliceIndex < sliceCount; sliceIndex++) {
			const bucketRange = getSliceBucketRange(nextFetchRequest.time_range, sliceCount, sliceIndex)
			const dateMs = bucketRange.end.getTime()
			const dateLabel = formatBucketEndLabel(
				bucketRange.end,
				showTimeInBucketLabel.value,
				showYearInBucketLabel.value,
			)

			for (const breakdown of breakdownValues) {
				const rowId = `${dateMs}::${breakdown}`
				nextRows.set(rowId, {
					id: rowId,
					date: dateLabel,
					dateMs,
					project: projectDisplayValues.get(breakdown) ?? '',
					breakdown,
					breakdownDisplay: breakdownDisplayValues.get(breakdown) ?? breakdown,
					views: 0,
					downloads: 0,
					revenue: 0,
					playtime: 0,
				})
			}
		}
	} else {
		for (const breakdown of breakdownValues) {
			nextRows.set(breakdown, {
				id: breakdown,
				date: '',
				dateMs: 0,
				project: projectDisplayValues.get(breakdown) ?? '',
				breakdown,
				breakdownDisplay: breakdownDisplayValues.get(breakdown) ?? breakdown,
				views: 0,
				downloads: 0,
				revenue: 0,
				playtime: 0,
			})
		}
	}

	nextTimeSlices.forEach((slice, sliceIndex) => {
		const bucketRange = getSliceBucketRange(nextFetchRequest.time_range, sliceCount, sliceIndex)
		const dateMs = bucketRange.end.getTime()

		for (const point of slice) {
			if (!('source_project' in point)) {
				continue
			}

			if (!selectedProjectIdSet.value.has(point.source_project)) {
				continue
			}

			if (!doesAnalyticsPointMatchFilters(point, selectedFilters.value)) {
				continue
			}

			const pointStat = getStatForMetric(point.metric_kind)
			if (!pointStat || !relevantStats.value.has(pointStat)) {
				continue
			}

			const breakdown =
				nextSelectedBreakdown === 'none'
					? point.source_project
					: getBreakdownValue(point, nextSelectedBreakdown)
			if (nextSelectedBreakdown !== 'none' && breakdown === ALL_BREAKDOWN_VALUE) {
				continue
			}

			const rowId = includeDate ? `${dateMs}::${breakdown}` : breakdown
			const row = nextRows.get(rowId)
			if (row) {
				addMetricToRow(row, point)
			}
		}
	})

	return Array.from(nextRows.values())
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
			width: stats.length > 2 ? '20%' : '',
		})
	}

	if (showBreakdownColumn.value) {
		if (showProjectVersionProjectColumn.value) {
			nextColumns.push({
				key: 'project',
				label: 'Project',
				enableSorting: true,
			})
		}

		nextColumns.push({
			key: 'breakdown',
			label: breakdownColumnLabel.value,
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
		const availableColumns = new Set(nextColumns.map((column) => column.key))
		if (sortColumn.value && availableColumns.has(sortColumn.value)) {
			return
		}

		applyDefaultSort(nextColumns)
	},
	{ immediate: true },
)

watch(includeDateColumn, () => {
	applyDefaultSort(activeColumns.value)
})

const sortedRows = computed<AnalyticsTableRow[]>(() => {
	return displayedSortedRows.value
})

watch(
	[
		fetchRequest,
		timeSlices,
		selectedProjectIds,
		selectedGroupBy,
		selectedBreakdown,
		selectedFilters,
		projects,
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

watch([sortColumn, sortDirection], () => {
	invalidateSortedCaches()
	scheduleRowsForMode(activeTableMode.value)
	scheduleInactiveModeWarmup()
})

const pageCount = computed(() => Math.max(Math.ceil(sortedRows.value.length / PAGE_SIZE), 1))
const visibleRowStart = computed(() =>
	sortedRows.value.length === 0 ? 0 : (currentPage.value - 1) * PAGE_SIZE + 1,
)
const visibleRowEnd = computed(() =>
	Math.min(currentPage.value * PAGE_SIZE, sortedRows.value.length),
)
const paginatedRows = computed<AnalyticsTableRow[]>(() =>
	sortedRows.value.slice((currentPage.value - 1) * PAGE_SIZE, currentPage.value * PAGE_SIZE),
)

watch(sortedRows, () => {
	currentPage.value = 1
})

watch(pageCount, (nextPageCount) => {
	if (currentPage.value > nextPageCount) {
		currentPage.value = nextPageCount
	}
})

function invalidateTableCaches() {
	tableCacheGeneration++
	tableRowsByMode.value = {
		date_breakdown: null,
		breakdown_only: null,
	}
	invalidateSortedCaches()
}

function invalidateSortedCaches() {
	sortedRowsByMode.value = {
		date_breakdown: null,
		breakdown_only: null,
	}
}

function hasSortedRowsForMode(mode: TableMode): boolean {
	const cached = sortedRowsByMode.value[mode]
	return (
		cached !== null &&
		cached.sortColumn === sortColumn.value &&
		cached.sortDirection === sortDirection.value
	)
}

function setCachedTableRows(mode: TableMode, rows: AnalyticsTableRow[]) {
	tableRowsByMode.value = {
		...tableRowsByMode.value,
		[mode]: rows,
	}
}

function setCachedSortedRows(mode: TableMode, rows: AnalyticsTableRow[]) {
	sortedRowsByMode.value = {
		...sortedRowsByMode.value,
		[mode]: {
			sortColumn: sortColumn.value,
			sortDirection: sortDirection.value,
			rows,
		},
	}

	if (mode === activeTableMode.value) {
		displayRowsForMode(mode)
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
	const cached = sortedRowsByMode.value[mode]
	if (!cached) {
		return
	}

	displayedTableMode.value = mode
	displayedSortColumn.value = cached.sortColumn
	displayedSortDirection.value = cached.sortDirection
	displayedSortedRows.value = cached.rows
}

async function buildRowsForMode(
	mode: TableMode,
	generation: number,
	requestId: number,
) {
	await waitForDeferredTableWork()

	if (isStaleBuild(mode, generation, requestId)) {
		return
	}

	const cachedRows = tableRowsByMode.value[mode]
	const rows = cachedRows ?? buildTableRows(mode)

	if (isStaleBuild(mode, generation, requestId)) {
		return
	}

	if (!cachedRows) {
		setCachedTableRows(mode, rows)
	}

	setCachedSortedRows(mode, sortTableRows(rows))
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
				align: 'right',
			}
		case 'downloads':
			return {
				key: 'downloads',
				label: 'Downloads',
				enableSorting: true,
				align: 'right',
			}
		case 'revenue':
			return {
				key: 'revenue',
				label: 'Revenue',
				enableSorting: true,
				align: 'right',
			}
		case 'playtime':
			return {
				key: 'playtime',
				label: 'Playtime',
				enableSorting: true,
				align: 'right',
			}
		default:
			return null
	}
}

function applyDefaultSort(nextColumns = activeColumns.value) {
	const nextSortColumn = getDefaultSortColumn(nextColumns)
	sortColumn.value = nextSortColumn
	sortDirection.value = getDefaultSortDirection(nextSortColumn)
}

function applyRequestedSort(column: string, direction: SortDirection) {
	sortColumn.value = column as TableColumnKey
	sortDirection.value = direction
}

function switchPage(page: number) {
	currentPage.value = page
}

function getDefaultSortColumn(
	nextColumns: TableColumn<TableColumnKey>[],
): TableColumnKey | undefined {
	const availableColumns = new Set(nextColumns.map((column) => column.key))
	if (availableColumns.has('date')) {
		return 'date'
	}

	if (availableColumns.has('downloads')) {
		return 'downloads'
	}

	return nextColumns[0]?.key
}

function getDefaultSortDirection(column: TableColumnKey | undefined): SortDirection {
	return column === 'date' || column === 'project' || column === 'breakdown' ? 'asc' : 'desc'
}

function getBreakdownValue(
	point: Labrinth.Analytics.v3.ProjectAnalytics,
	selectedBreakdown: AnalyticsBreakdownPreset,
): string {
	return getAnalyticsBreakdownValue(point, selectedBreakdown)
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
	if (selectedBreakdown.value === 'none') {
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

function downloadCsv() {
	if (!import.meta.client || sortedRows.value.length === 0) {
		return
	}

	const visibleColumns = columns.value
	const header = visibleColumns
		.map((column) => escapeCsvField(column.label ?? column.key))
		.join(',')

	const rows = sortedRows.value.map((row) =>
		visibleColumns.map((column) => escapeCsvField(getCsvCellValue(row, column.key))).join(','),
	)

	const csvContent = [header, ...rows].join('\n')
	const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
	const url = URL.createObjectURL(blob)

	const downloadLink = document.createElement('a')
	downloadLink.setAttribute('href', url)
	downloadLink.setAttribute(
		'download',
		`analytics-${displayedIncludeDateColumn.value ? 'date-breakdown' : 'breakdown-only'}.csv`,
	)
	downloadLink.style.visibility = 'hidden'

	document.body.appendChild(downloadLink)
	downloadLink.click()
	document.body.removeChild(downloadLink)

	URL.revokeObjectURL(url)
}
</script>
