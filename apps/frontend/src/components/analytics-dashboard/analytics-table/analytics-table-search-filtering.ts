import type { TableColumn } from '@modrinth/ui'

import { isAnalyticsTableBreakdownColumnKey } from './analytics-table-columns'
import type { AnalyticsTableColumnKey, AnalyticsTableRow } from './analytics-table-types'

const SEARCHABLE_COLUMN_KEYS = new Set<AnalyticsTableColumnKey>([
	'date',
	'project',
	'dependent_on',
])

export function getAnalyticsTableSearchableColumns(
	columns: TableColumn<AnalyticsTableColumnKey>[],
): TableColumn<AnalyticsTableColumnKey>[] {
	return columns.filter(
		(column) =>
			SEARCHABLE_COLUMN_KEYS.has(column.key) || isAnalyticsTableBreakdownColumnKey(column.key),
	)
}

export function filterAnalyticsTableRowsBySearch(
	rows: AnalyticsTableRow[],
	searchableColumns: TableColumn<AnalyticsTableColumnKey>[],
	query: string,
): AnalyticsTableRow[] {
	if (!query || searchableColumns.length === 0) {
		return rows
	}

	return rows.filter((row) =>
		searchableColumns.some((column) =>
			String(getAnalyticsTableSearchableCellValue(row, column.key)).toLowerCase().includes(query),
		),
	)
}

function getAnalyticsTableSearchableCellValue(
	row: AnalyticsTableRow,
	key: AnalyticsTableColumnKey,
): string {
	switch (key) {
		case 'date':
			return row.date
		case 'project':
			return row.project
		case 'dependent_on':
			return row.dependent_on
		case 'breakdown':
			return row.breakdownDisplay
		default:
			return isAnalyticsTableBreakdownColumnKey(key) ? String(row[key] ?? '') : ''
	}
}
