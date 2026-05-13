<template>
	<div class="relative overflow-hidden rounded-2xl">
		<AnalyticsLoadingBar :loading="isDataLoading" />
		<Table
			v-model:sort-column="sortColumn"
			v-model:sort-direction="sortDirection"
			:columns="columns"
			:data="sortedRows"
			row-key="id"
			virtualized
			:virtual-row-height="56"
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
			<template #cell-breakdown="{ value }">
				<span
					class="text-primary"
					:class="{
						capitalize: selectedBreakdown === 'monetization',
					}"
					>{{ formatBreakdownDisplayValue(String(value)) }}</span
				>
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
					{{ emptyTableMessage }}
				</div>
			</template>
		</Table>
		<div v-if="isDataLoading" class="absolute inset-0 z-10 overflow-hidden rounded-xl">
			<div class="absolute inset-0 bg-surface-3 opacity-50" />
			<div class="absolute inset-0 backdrop-blur-[4px]" />
			<div class="absolute inset-0 flex h-full items-start justify-center pt-52">
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
import { ButtonStyled, Table, type TableColumn, Toggle, useFormatNumber } from '@modrinth/ui'

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
type TableColumnKey = 'date' | 'breakdown' | 'views' | 'downloads' | 'revenue' | 'playtime'

type AnalyticsTableRow = {
	id: string
	date: string
	dateMs: number
	breakdown: string
	views: number
	downloads: number
	revenue: number
	playtime: number
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
} = injectAnalyticsDashboardContext()
const formatNumber = useFormatNumber()
const isDataLoading = computed(() => isLoading.value)

const tableMode = ref<TableMode>('breakdown_only')
const sortColumn = ref<TableColumnKey | undefined>('date')
const sortDirection = ref<SortDirection>('asc')

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
const showDateToggle = computed(() => showBreakdownColumn.value)
const includeDateColumn = computed(
	() => tableMode.value === 'date_breakdown' || !showBreakdownColumn.value,
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

const tableRows = computed<AnalyticsTableRow[]>(() => {
	const nextFetchRequest = fetchRequest.value
	const nextTimeSlices = timeSlices.value
	const nextSelectedBreakdown = selectedBreakdown.value

	if (!nextFetchRequest || selectedProjectIdSet.value.size === 0) {
		return []
	}

	const sliceCount = getSliceCount(nextFetchRequest.time_range, nextTimeSlices.length)
	const includeDate = includeDateColumn.value

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
					breakdown,
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
				breakdown,
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
})

const columns = computed<TableColumn<TableColumnKey>[]>(() => {
	const nextColumns: TableColumn<TableColumnKey>[] = []

	const stats = getRelevantAnalyticsDashboardStats(selectedBreakdown.value, selectedFilters.value)

	if (includeDateColumn.value) {
		nextColumns.push({
			key: 'date',
			label: 'Date',
			enableSorting: true,
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

	for (const stat of stats) {
		const column = getMetricColumnForStat(stat)
		if (column) {
			nextColumns.push(column)
		}
	}

	return nextColumns
})

watch(
	columns,
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
	applyDefaultSort()
})

const sortedRows = computed<AnalyticsTableRow[]>(() => {
	const rows = [...tableRows.value]
	const activeSortColumn = sortColumn.value

	if (!activeSortColumn) {
		return rows
	}

	const directionFactor = sortDirection.value === 'asc' ? 1 : -1

	rows.sort((left, right) => {
		const primaryResult = getSortComparison(left, right, activeSortColumn)
		if (primaryResult !== 0) {
			return primaryResult * directionFactor
		}

		const dateResult = left.dateMs - right.dateMs
		if (dateResult !== 0) {
			return dateResult * directionFactor
		}

		return (
			left.breakdown.localeCompare(right.breakdown, undefined, { sensitivity: 'base' }) *
			directionFactor
		)
	})

	return rows
})

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

function applyDefaultSort(nextColumns = columns.value) {
	const nextSortColumn = getDefaultSortColumn(nextColumns)
	sortColumn.value = nextSortColumn
	sortDirection.value = getDefaultSortDirection(nextSortColumn)
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
	return column === 'date' || column === 'breakdown' ? 'asc' : 'desc'
}

function getBreakdownValue(
	point: Labrinth.Analytics.v3.ProjectAnalytics,
	selectedBreakdown: AnalyticsBreakdownPreset,
): string {
	return getAnalyticsBreakdownValue(point, selectedBreakdown)
}

function getSortComparison(
	left: AnalyticsTableRow,
	right: AnalyticsTableRow,
	column: TableColumnKey,
): number {
	switch (column) {
		case 'date':
			return left.dateMs - right.dateMs
		case 'breakdown':
			return formatBreakdownDisplayValue(left.breakdown).localeCompare(
				formatBreakdownDisplayValue(right.breakdown),
				undefined,
				{ sensitivity: 'base' },
			)
		case 'views':
			return left.views - right.views
		case 'downloads':
			return left.downloads - right.downloads
		case 'revenue':
			return left.revenue - right.revenue
		case 'playtime':
			return left.playtime - right.playtime
		default:
			return 0
	}
}

function formatBreakdownDisplayValue(value: string): string {
	if (selectedBreakdown.value === 'none') {
		return projectNamesById.value.get(value) ?? value
	}
	return formatBreakdownLabel(value, selectedBreakdown.value, getVersionDisplayName)
}

function getCsvCellValue(row: AnalyticsTableRow, key: TableColumnKey): string | number {
	switch (key) {
		case 'date':
			return row.date
		case 'breakdown':
			return formatBreakdownDisplayValue(row.breakdown)
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
		`analytics-${includeDateColumn.value ? 'date-breakdown' : 'breakdown-only'}.csv`,
	)
	downloadLink.style.visibility = 'hidden'

	document.body.appendChild(downloadLink)
	downloadLink.click()
	document.body.removeChild(downloadLink)

	URL.revokeObjectURL(url)
}
</script>
