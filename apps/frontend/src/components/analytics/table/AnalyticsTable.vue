<template>
	<section class="flex flex-col rounded-2xl border border-solid border-surface-5 bg-surface-3">
		<div class="flex flex-col gap-3 p-4 xl:flex-row xl:items-center xl:justify-between">
			<div class="text-xl font-semibold text-contrast">Breakdown</div>

			<div class="flex flex-wrap items-center gap-2">
				<button
					type="button"
					class="inline-flex items-center rounded-xl border border-solid px-3 py-1.5 text-sm font-semibold transition-colors"
					:class="
						tableMode === 'date_breakdown'
							? 'border-brand bg-highlight-green text-brand'
							: 'border-surface-5 bg-surface-3 text-primary hover:bg-surface-4'
					"
					@click="tableMode = 'date_breakdown'"
				>
					Date + Breakdown
				</button>
				<button
					v-if="showBreakdownOnlyMode"
					type="button"
					class="inline-flex items-center rounded-xl border border-solid px-3 py-1.5 text-sm font-semibold transition-colors"
					:class="
						tableMode === 'breakdown_only'
							? 'border-brand bg-highlight-green text-brand'
							: 'border-surface-5 bg-surface-3 text-primary hover:bg-surface-4'
					"
					@click="tableMode = 'breakdown_only'"
				>
					Breakdown only
				</button>
				<button
					type="button"
					class="inline-flex items-center gap-2 rounded-xl border border-solid px-3 py-1.5 text-sm font-semibold transition-colors"
					:class="
						sortedRows.length > 0
							? 'border-surface-5 bg-surface-3 text-primary hover:bg-surface-4'
							: 'cursor-not-allowed border-surface-5 bg-surface-2 text-secondary'
					"
					:disabled="sortedRows.length === 0"
					@click="downloadCsv"
				>
					<DownloadIcon class="size-4" />
					Export CSV
				</button>
			</div>
		</div>

		<Table
			v-model:sort-column="sortColumn"
			v-model:sort-direction="sortDirection"
			:columns="columns"
			:data="sortedRows"
			row-key="id"
		>
			<template #cell-date="{ value }">
				<span class="text-primary">{{ value }}</span>
			</template>
			<template #cell-breakdown="{ value }">
				<span
					class="text-primary"
					:class="{
						capitalize: selectedBreakdown === 'monetization',
					}"
					>{{ value }}</span
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
		</Table>
	</section>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon } from '@modrinth/assets'
import { Table, type TableColumn, useFormatNumber } from '@modrinth/ui'

import type { AnalyticsBreakdownPreset } from '~/providers/analytics/analytics'
import { injectAnalyticsDashboardContext } from '~/providers/analytics/analytics'

import { getAnalyticsBreakdownValue } from '../breakdown'
import {
	formatBucketEndLabel,
	getSliceBucketRange,
	getSliceCount,
	isTimeRelevantForGroupBy,
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

const { selectedProjectIds, selectedGroupBy, selectedBreakdown, fetchRequest, timeSlices } =
	injectAnalyticsDashboardContext()
const formatNumber = useFormatNumber()

const tableMode = ref<TableMode>('date_breakdown')
const sortColumn = ref<TableColumnKey | undefined>('date')
const sortDirection = ref<SortDirection>('asc')

const showBreakdownOnlyMode = computed(() => selectedProjectIds.value.length > 1)

watch(
	showBreakdownOnlyMode,
	(nextValue) => {
		if (!nextValue && tableMode.value === 'breakdown_only') {
			tableMode.value = 'date_breakdown'
		}
	},
	{ immediate: true },
)

watch(tableMode, (nextMode) => {
	if (nextMode === 'breakdown_only' && sortColumn.value === 'date') {
		sortColumn.value = 'breakdown'
		sortDirection.value = 'asc'
	}
})

const selectedProjectIdSet = computed(() => new Set(selectedProjectIds.value))

const showTimeInBucketLabel = computed(() => isTimeRelevantForGroupBy(selectedGroupBy.value))

const tableRows = computed<AnalyticsTableRow[]>(() => {
	const nextFetchRequest = fetchRequest.value
	const nextTimeSlices = timeSlices.value
	const nextSelectedBreakdown = selectedBreakdown.value

	if (!nextFetchRequest || nextTimeSlices.length === 0 || selectedProjectIdSet.value.size === 0) {
		return []
	}

	const nextRows = new Map<string, AnalyticsTableRow>()
	const sliceCount = getSliceCount(nextFetchRequest.time_range, nextTimeSlices.length)

	nextTimeSlices.forEach((slice, sliceIndex) => {
		const bucketRange = getSliceBucketRange(nextFetchRequest.time_range, sliceCount, sliceIndex)
		const dateMs = bucketRange.end.getTime()
		const dateLabel = formatBucketEndLabel(bucketRange.end, showTimeInBucketLabel.value)

		for (const point of slice) {
			if (!('source_project' in point)) {
				continue
			}

			if (!selectedProjectIdSet.value.has(point.source_project)) {
				continue
			}

			const breakdown = getBreakdownValue(point, nextSelectedBreakdown)
			const rowId = tableMode.value === 'date_breakdown' ? `${dateMs}::${breakdown}` : breakdown

			let row = nextRows.get(rowId)

			if (!row) {
				row = {
					id: rowId,
					date: tableMode.value === 'date_breakdown' ? dateLabel : '',
					dateMs: tableMode.value === 'date_breakdown' ? dateMs : 0,
					breakdown,
					views: 0,
					downloads: 0,
					revenue: 0,
					playtime: 0,
				}
				nextRows.set(rowId, row)
			}

			addMetricToRow(row, point)
		}
	})

	return Array.from(nextRows.values())
})

const columns = computed<TableColumn<TableColumnKey>[]>(() => {
	const nextColumns: TableColumn<TableColumnKey>[] = []

	if (tableMode.value === 'date_breakdown') {
		nextColumns.push({
			key: 'date',
			label: 'Date',
			enableSorting: true,
			width: '20%',
		})
	}

	nextColumns.push(
		{
			key: 'breakdown',
			label: 'Breakdown',
			enableSorting: true,
		},
		{
			key: 'views',
			label: 'Views',
			enableSorting: true,
			align: 'right',
		},
		{
			key: 'downloads',
			label: 'Downloads',
			enableSorting: true,
			align: 'right',
		},
		{
			key: 'revenue',
			label: 'Revenue',
			enableSorting: true,
			align: 'right',
		},
		{
			key: 'playtime',
			label: 'Playtime',
			enableSorting: true,
			align: 'right',
		},
	)

	return nextColumns
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
			return left.breakdown.localeCompare(right.breakdown, undefined, { sensitivity: 'base' })
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

function getCsvCellValue(row: AnalyticsTableRow, key: TableColumnKey): string | number {
	switch (key) {
		case 'date':
			return row.date
		case 'breakdown':
			return row.breakdown
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
		`analytics-${tableMode.value === 'breakdown_only' ? 'breakdown-only' : 'date-breakdown'}.csv`,
	)
	downloadLink.style.visibility = 'hidden'

	document.body.appendChild(downloadLink)
	downloadLink.click()
	document.body.removeChild(downloadLink)

	URL.revokeObjectURL(url)
}
</script>
