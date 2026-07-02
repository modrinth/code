import type { TableColumn } from '@modrinth/ui'

import type {
	AnalyticsBreakdownPreset,
	AnalyticsDashboardStat,
	AnalyticsSelectedFilters,
} from '~/providers/analytics/analytics'

import {
	analyticsGroupByMessages,
	formatAnalyticsBreakdownLabel,
	formatAnalyticsStatLabel,
	type FormatMessage,
} from '../analytics-messages'
import type {
	AnalyticsTableBreakdownColumnKey,
	AnalyticsTableBreakdownPreset,
	AnalyticsTableColumnKey,
} from './analytics-table-types'

type BuildAnalyticsTableColumnsOptions = {
	includeDate: boolean
	selectedBreakdowns: readonly AnalyticsTableBreakdownPreset[]
	selectedFilters: AnalyticsSelectedFilters
	showBreakdownColumn: boolean
	formatMessage: FormatMessage
	getRelevantAnalyticsDashboardStats: (
		breakdowns: readonly AnalyticsBreakdownPreset[],
		filters?: AnalyticsSelectedFilters,
	) => readonly AnalyticsDashboardStat[]
}

export function getAnalyticsTableBreakdownColumnLabel(
	breakdown: AnalyticsBreakdownPreset,
	formatMessage: FormatMessage,
): string {
	return formatAnalyticsBreakdownLabel(breakdown, formatMessage)
}

export function buildAnalyticsTableColumns({
	includeDate,
	selectedBreakdowns,
	selectedFilters,
	showBreakdownColumn,
	formatMessage,
	getRelevantAnalyticsDashboardStats,
}: BuildAnalyticsTableColumnsOptions): TableColumn<AnalyticsTableColumnKey>[] {
	const nextColumns: TableColumn<AnalyticsTableColumnKey>[] = []
	const stats = getRelevantAnalyticsDashboardStats(selectedBreakdowns, selectedFilters)

	if (includeDate) {
		nextColumns.push({
			key: 'date',
			label: formatMessage(analyticsGroupByMessages.date),
			enableSorting: true,
			defaultSortDirection: 'desc',
			width: stats.length > 2 ? '20%' : '',
		})
	}

	if (showBreakdownColumn) {
		for (const breakdown of selectedBreakdowns) {
			nextColumns.push({
				key: getAnalyticsTableBreakdownColumnKey(breakdown),
				label: getAnalyticsTableBreakdownColumnLabel(breakdown, formatMessage),
				enableSorting: true,
				width: breakdown === 'project' && selectedBreakdowns.length === 1 ? '45%' : undefined,
			})
		}
	}

	for (const stat of stats) {
		const column = getAnalyticsTableMetricColumn(stat, formatMessage)
		if (column) {
			nextColumns.push(column)
		}
	}

	return nextColumns
}

export function getAnalyticsTableMetricColumn(
	stat: AnalyticsDashboardStat,
	formatMessage: FormatMessage,
): TableColumn<AnalyticsTableColumnKey> | null {
	switch (stat) {
		case 'views':
			return {
				key: 'views',
				label: formatAnalyticsStatLabel('views', formatMessage),
				enableSorting: true,
				defaultSortDirection: 'desc',
				align: 'right',
				width: '20%',
			}
		case 'downloads':
			return {
				key: 'downloads',
				label: formatAnalyticsStatLabel('downloads', formatMessage),
				enableSorting: true,
				defaultSortDirection: 'desc',
				align: 'right',
				width: '20%',
			}
		case 'revenue':
			return {
				key: 'revenue',
				label: formatAnalyticsStatLabel('revenue', formatMessage),
				enableSorting: true,
				defaultSortDirection: 'desc',
				align: 'right',
				width: '20%',
			}
		case 'playtime':
			return {
				key: 'playtime',
				label: formatAnalyticsStatLabel('playtime', formatMessage),
				enableSorting: true,
				defaultSortDirection: 'desc',
				align: 'right',
				width: '20%',
			}
		default:
			return null
	}
}

export function getAnalyticsTableBreakdownColumnKey(
	breakdown: AnalyticsTableBreakdownPreset,
): AnalyticsTableBreakdownColumnKey {
	return `breakdown_${breakdown}`
}

export function isAnalyticsTableBreakdownColumnKey(
	key: AnalyticsTableColumnKey,
): key is AnalyticsTableBreakdownColumnKey {
	return key.startsWith('breakdown_')
}
