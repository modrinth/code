import type { TableColumn } from '@modrinth/ui'
import type { LocationQuery } from 'vue-router'

import {
	buildAnalyticsTableSortRouteQuery,
	readAnalyticsTableSortState,
} from '~/components/analytics-dashboard/analytics-route-query'
import type { AnalyticsDashboardStat } from '~/providers/analytics/analytics'

import { isAnalyticsTableBreakdownColumnKey } from './analytics-table-columns'
import {
	getAnalyticsTableDefaultSortColumn,
	getAnalyticsTableDefaultSortDirection,
} from './analytics-table-sorting'
import type {
	AnalyticsTableColumnKey,
	AnalyticsTableSortDirectionValue,
	AnalyticsTableSortState,
} from './analytics-table-types'

type GetDefaultAnalyticsTableSortStateOptions = {
	columns: TableColumn<AnalyticsTableColumnKey>[]
	showGraphDatasetSelection: boolean
	activeStat: AnalyticsDashboardStat
}

export function getRouteAnalyticsTableSortState(
	query: LocationQuery,
	columns: TableColumn<AnalyticsTableColumnKey>[],
	defaultSortOptions: GetDefaultAnalyticsTableSortStateOptions,
): AnalyticsTableSortState {
	return getAvailableAnalyticsTableSortState(
		readAnalyticsTableSortState(query, getDefaultAnalyticsTableSortState(defaultSortOptions)),
		columns,
		defaultSortOptions,
	)
}

export function getAvailableAnalyticsTableSortState(
	state: AnalyticsTableSortState,
	columns: TableColumn<AnalyticsTableColumnKey>[],
	defaultSortOptions: GetDefaultAnalyticsTableSortStateOptions,
): AnalyticsTableSortState {
	const availableColumns = new Set(columns.map((column) => column.key))
	if (state.sortColumn && availableColumns.has(state.sortColumn)) {
		return state
	}
	if (state.sortColumn === 'breakdown') {
		const firstBreakdownColumn = columns.find((column) =>
			isAnalyticsTableBreakdownColumnKey(column.key),
		)
		if (firstBreakdownColumn) {
			return {
				sortColumn: firstBreakdownColumn.key,
				sortDirection: state.sortDirection,
			}
		}
	}

	return getDefaultAnalyticsTableSortState(defaultSortOptions)
}

export function getDefaultAnalyticsTableSortState({
	columns,
	showGraphDatasetSelection,
	activeStat,
}: GetDefaultAnalyticsTableSortStateOptions): AnalyticsTableSortState {
	const nextSortColumn = getAnalyticsTableDefaultSortColumn(
		columns,
		showGraphDatasetSelection,
		activeStat,
	)
	return {
		sortColumn: nextSortColumn,
		sortDirection: getAnalyticsTableDefaultSortDirection(nextSortColumn, columns),
	}
}

export function areAnalyticsTableSortStatesEqual(
	left: AnalyticsTableSortState,
	right: AnalyticsTableSortState,
): boolean {
	return left.sortColumn === right.sortColumn && left.sortDirection === right.sortDirection
}

export function buildSyncedAnalyticsTableSortRouteQuery(
	query: LocationQuery,
	sortState: AnalyticsTableSortState,
	columns: TableColumn<AnalyticsTableColumnKey>[],
	defaultSortOptions: GetDefaultAnalyticsTableSortStateOptions,
) {
	const nextSortState = getAvailableAnalyticsTableSortState(sortState, columns, defaultSortOptions)

	return buildAnalyticsTableSortRouteQuery(
		query,
		nextSortState,
		getDefaultAnalyticsTableSortState(defaultSortOptions),
	)
}

export function toAnalyticsTableSortState(
	sortColumn: AnalyticsTableColumnKey | undefined,
	sortDirection: AnalyticsTableSortDirectionValue,
): AnalyticsTableSortState {
	return {
		sortColumn,
		sortDirection,
	}
}
