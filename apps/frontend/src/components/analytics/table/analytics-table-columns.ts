import type { TableColumn } from '@modrinth/ui'

import type {
	AnalyticsBreakdownPreset,
	AnalyticsDashboardStat,
	AnalyticsSelectedFilters,
} from '~/providers/analytics/analytics'

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
	showProjectVersionProjectColumn: boolean
	getRelevantAnalyticsDashboardStats: (
		breakdowns: readonly AnalyticsBreakdownPreset[],
		filters?: AnalyticsSelectedFilters,
	) => readonly AnalyticsDashboardStat[]
}

export function getAnalyticsTableBreakdownColumnLabel(breakdown: AnalyticsBreakdownPreset): string {
	switch (breakdown) {
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
}

export function buildAnalyticsTableColumns({
	includeDate,
	selectedBreakdowns,
	selectedFilters,
	showBreakdownColumn,
	showProjectVersionProjectColumn,
	getRelevantAnalyticsDashboardStats,
}: BuildAnalyticsTableColumnsOptions): TableColumn<AnalyticsTableColumnKey>[] {
	const nextColumns: TableColumn<AnalyticsTableColumnKey>[] = []
	const stats = getRelevantAnalyticsDashboardStats(selectedBreakdowns, selectedFilters)

	if (includeDate) {
		nextColumns.push({
			key: 'date',
			label: 'Date',
			enableSorting: true,
			defaultSortDirection: 'desc',
			width: stats.length > 2 ? '20%' : '',
		})
	}

	if (showBreakdownColumn) {
		for (const breakdown of selectedBreakdowns) {
			nextColumns.push({
				key: getAnalyticsTableBreakdownColumnKey(breakdown),
				label: getAnalyticsTableBreakdownColumnLabel(breakdown),
				enableSorting: true,
			})
		}
	}

	if (showProjectVersionProjectColumn) {
		nextColumns.push({
			key: 'project',
			label: 'Project',
			enableSorting: true,
		})
	}

	for (const stat of stats) {
		const column = getAnalyticsTableMetricColumn(stat)
		if (column) {
			nextColumns.push(column)
		}
	}

	return nextColumns
}

export function getAnalyticsTableMetricColumn(
	stat: AnalyticsDashboardStat,
): TableColumn<AnalyticsTableColumnKey> | null {
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
