import type { Labrinth } from '@modrinth/api-client'
import type { TableColumn } from '@modrinth/ui'

import { analyticsTableMessages, type FormatMessage } from '../analytics-messages'
import { isAnalyticsTableBreakdownColumnKey } from './analytics-table-columns'
import type { AnalyticsTableColumnKey, AnalyticsTableRow } from './analytics-table-types'

export function buildAnalyticsTableCsvContent(
	rows: AnalyticsTableRow[],
	visibleColumns: TableColumn<AnalyticsTableColumnKey>[],
	formatMessage: FormatMessage,
): string {
	const header = visibleColumns
		.map((column) =>
			escapeAnalyticsTableCsvField(getAnalyticsTableCsvHeaderLabel(column, formatMessage)),
		)
		.join(',')

	const csvRows = rows.map((row) =>
		visibleColumns
			.map((column) => escapeAnalyticsTableCsvField(getAnalyticsTableCsvCellValue(row, column.key)))
			.join(','),
	)

	return [header, ...csvRows].join('\n')
}

export function downloadAnalyticsTableCsv(filename: string, csvContent: string) {
	if (!import.meta.client) {
		return
	}

	const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
	const url = URL.createObjectURL(blob)

	const downloadLink = document.createElement('a')
	downloadLink.setAttribute('href', url)
	downloadLink.setAttribute('download', filename)
	downloadLink.style.visibility = 'hidden'

	document.body.appendChild(downloadLink)
	downloadLink.click()
	document.body.removeChild(downloadLink)

	URL.revokeObjectURL(url)
}

export function getAnalyticsTableCsvFilename(
	breakdownColumnLabel: string,
	fetchRequest: Labrinth.Analytics.v3.FetchRequest | null,
	formatMessage: FormatMessage,
): string {
	return `${sanitizeAnalyticsTableCsvFilename(
		formatMessage(analyticsTableMessages.csvFilename, {
			breakdown: breakdownColumnLabel,
			dateRange: getAnalyticsTableCsvFilenameDateRange(fetchRequest, formatMessage),
		}),
	)}.csv`
}

function getAnalyticsTableCsvCellValue(
	row: AnalyticsTableRow,
	key: AnalyticsTableColumnKey,
): string | number {
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
			return isAnalyticsTableBreakdownColumnKey(key) ? String(row[key] ?? '') : ''
	}
}

function getAnalyticsTableCsvHeaderLabel(
	column: TableColumn<AnalyticsTableColumnKey>,
	formatMessage: FormatMessage,
): string {
	if (column.key === 'playtime') {
		return formatMessage(analyticsTableMessages.playtimeSecondsHeader)
	}

	return column.label ?? column.key
}

function escapeAnalyticsTableCsvField(value: string | number): string {
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

function formatAnalyticsTableCsvFilenameDate(date: Date): string {
	return date.toLocaleDateString(undefined, {
		year: 'numeric',
		month: 'short',
		day: 'numeric',
	})
}

function getAnalyticsTableCsvFilenameDateRange(
	fetchRequest: Labrinth.Analytics.v3.FetchRequest | null,
	formatMessage: FormatMessage,
): string {
	const timeRange = fetchRequest?.time_range
	if (!timeRange) {
		return formatMessage(analyticsTableMessages.csvSelectedRange)
	}

	const start = new Date(timeRange.start)
	const end = new Date(timeRange.end)
	if (Number.isNaN(start.getTime()) || Number.isNaN(end.getTime())) {
		return formatMessage(analyticsTableMessages.csvSelectedRange)
	}

	const startLabel = formatAnalyticsTableCsvFilenameDate(start)
	const endLabel = formatAnalyticsTableCsvFilenameDate(end)
	return startLabel === endLabel
		? startLabel
		: formatMessage(analyticsTableMessages.csvDateRange, {
				start: startLabel,
				end: endLabel,
			})
}

function sanitizeAnalyticsTableCsvFilename(value: string): string {
	return value
		.replace(/[<>:"/\\|?*]/g, '')
		.replace(/\s+/g, ' ')
		.trim()
}
