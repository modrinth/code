import type { AnalyticsBreakdownPreset } from '~/providers/analytics/analytics'
import type {
	AnalyticsTableSortColumn,
	AnalyticsTableSortDirection,
} from '~/providers/analytics/query-builder-url'

export type AnalyticsTableMode = 'date_breakdown' | 'breakdown_only'
export type AnalyticsTableBreakdownPreset = Exclude<AnalyticsBreakdownPreset, 'none'>
export type AnalyticsTableBreakdownColumnKey = `breakdown_${AnalyticsTableBreakdownPreset}`
export type AnalyticsTableBreakdownDisplayValues = Partial<
	Record<AnalyticsTableBreakdownPreset, string>
>
export type AnalyticsTableColumnKey = AnalyticsTableSortColumn
export type AnalyticsTableSortState = {
	sortColumn: AnalyticsTableColumnKey | undefined
	sortDirection: AnalyticsTableSortDirection
}
export type AnalyticsTableSortDirectionValue = AnalyticsTableSortDirection

export type AnalyticsTableRow = {
	[key: string]: string | number | AnalyticsTableBreakdownDisplayValues
	id: string
	date: string
	dateMs: number
	project: string
	breakdown: string
	breakdownValues: AnalyticsTableBreakdownDisplayValues
	breakdownDisplays: AnalyticsTableBreakdownDisplayValues
	graphDatasetId: string
	breakdownDisplay: string
	views: number
	downloads: number
	revenue: number
	playtime: number
}

export type AnalyticsTableDisplayedRowsCache = {
	generation: number
	mode: AnalyticsTableMode
	sortColumn: AnalyticsTableColumnKey | undefined
	sortDirection: AnalyticsTableSortDirectionValue
	rows: AnalyticsTableRow[]
}
