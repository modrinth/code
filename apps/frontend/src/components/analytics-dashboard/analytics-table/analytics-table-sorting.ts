import type { TableColumn } from '@modrinth/ui'

import type { AnalyticsDashboardStat } from '~/providers/analytics/analytics'

import { isAnalyticsTableBreakdownColumnKey } from './analytics-table-columns'
import type {
	AnalyticsTableColumnKey,
	AnalyticsTableRow,
	AnalyticsTableSortDirectionValue,
} from './analytics-table-types'

export function sortAnalyticsTableRows(
	rows: AnalyticsTableRow[],
	sortColumn: AnalyticsTableColumnKey | undefined,
	sortDirection: AnalyticsTableSortDirectionValue,
	sortCollator: Intl.Collator,
): AnalyticsTableRow[] {
	const nextRows = [...rows]

	if (!sortColumn) {
		return nextRows
	}

	const directionFactor = sortDirection === 'asc' ? 1 : -1
	nextRows.sort(getAnalyticsTableRowComparator(sortColumn, directionFactor, sortCollator))

	return nextRows
}

export function getAnalyticsTableDefaultSortColumn(
	nextColumns: TableColumn<AnalyticsTableColumnKey>[],
	showGraphDatasetSelection: boolean,
	activeStat: AnalyticsDashboardStat,
): AnalyticsTableColumnKey | undefined {
	const availableColumns = new Set(nextColumns.map((column) => column.key))
	if (availableColumns.has('date')) {
		return 'date'
	}

	if (showGraphDatasetSelection && availableColumns.has(activeStat)) {
		return activeStat
	}

	if (availableColumns.has('downloads')) {
		return 'downloads'
	}

	return nextColumns[0]?.key
}

export function getAnalyticsTableDefaultSortDirection(
	column: AnalyticsTableColumnKey | undefined,
	nextColumns: TableColumn<AnalyticsTableColumnKey>[],
): AnalyticsTableSortDirectionValue {
	return nextColumns.find((nextColumn) => nextColumn.key === column)?.defaultSortDirection ?? 'asc'
}

export function getAnalyticsTableMetricSortedGraphDatasetIds(
	rows: AnalyticsTableRow[],
	sortColumn: AnalyticsTableColumnKey | undefined,
	sortCollator: Intl.Collator,
): string[] {
	const metricColumn = getAnalyticsTableMetricSortColumn(sortColumn)
	if (!metricColumn) {
		return []
	}

	const totalsByGraphDatasetId = new Map<string, number>()
	const labelsByGraphDatasetId = new Map<string, string>()
	for (const row of rows) {
		totalsByGraphDatasetId.set(
			row.graphDatasetId,
			(totalsByGraphDatasetId.get(row.graphDatasetId) ?? 0) + row[metricColumn],
		)
		if (!labelsByGraphDatasetId.has(row.graphDatasetId)) {
			labelsByGraphDatasetId.set(row.graphDatasetId, row.breakdownDisplay)
		}
	}

	return Array.from(totalsByGraphDatasetId.keys()).sort((left, right) => {
		const totalDifference =
			(totalsByGraphDatasetId.get(right) ?? 0) - (totalsByGraphDatasetId.get(left) ?? 0)
		return (
			totalDifference ||
			sortCollator.compare(
				labelsByGraphDatasetId.get(left) ?? left,
				labelsByGraphDatasetId.get(right) ?? right,
			) ||
			left.localeCompare(right)
		)
	})
}

export function getAnalyticsTableMetricSortColumn(
	column: AnalyticsTableColumnKey | undefined,
): AnalyticsDashboardStat | null {
	switch (column) {
		case 'views':
		case 'downloads':
		case 'revenue':
		case 'playtime':
			return column
		default:
			return null
	}
}

function getAnalyticsTableRowComparator(
	column: AnalyticsTableColumnKey,
	directionFactor: number,
	sortCollator: Intl.Collator,
): (left: AnalyticsTableRow, right: AnalyticsTableRow) => number {
	switch (column) {
		case 'date':
			return (left, right) =>
				compareAnalyticsTableRows(
					left,
					right,
					left.dateMs - right.dateMs,
					directionFactor,
					sortCollator,
				)
		case 'project':
			return (left, right) =>
				compareAnalyticsTableRows(
					left,
					right,
					sortCollator.compare(left.project, right.project),
					directionFactor,
					sortCollator,
				)
		case 'dependent_on':
			return (left, right) =>
				compareAnalyticsTableRows(
					left,
					right,
					sortCollator.compare(left.dependent_on, right.dependent_on),
					directionFactor,
					sortCollator,
				)
		case 'breakdown':
			return (left, right) =>
				compareAnalyticsTableRows(
					left,
					right,
					sortCollator.compare(left.breakdownDisplay, right.breakdownDisplay),
					directionFactor,
					sortCollator,
				)
		case 'views':
			return (left, right) =>
				compareAnalyticsTableRows(
					left,
					right,
					left.views - right.views,
					directionFactor,
					sortCollator,
				)
		case 'downloads':
			return (left, right) =>
				compareAnalyticsTableRows(
					left,
					right,
					left.downloads - right.downloads,
					directionFactor,
					sortCollator,
				)
		case 'revenue':
			return (left, right) =>
				compareAnalyticsTableRows(
					left,
					right,
					left.revenue - right.revenue,
					directionFactor,
					sortCollator,
				)
		case 'playtime':
			return (left, right) =>
				compareAnalyticsTableRows(
					left,
					right,
					left.playtime - right.playtime,
					directionFactor,
					sortCollator,
				)
		default:
			if (isAnalyticsTableBreakdownColumnKey(column)) {
				return (left, right) =>
					compareAnalyticsTableRows(
						left,
						right,
						sortCollator.compare(String(left[column] ?? ''), String(right[column] ?? '')),
						directionFactor,
						sortCollator,
					)
			}

			return () => 0
	}
}

function compareAnalyticsTableRows(
	left: AnalyticsTableRow,
	right: AnalyticsTableRow,
	primaryResult: number,
	directionFactor: number,
	sortCollator: Intl.Collator,
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
