import type { ComputedRef, Ref, WritableComputedRef } from 'vue'
import { computed, watch } from 'vue'

import { areStringArraysEqual } from '~/components/analytics-dashboard/analytics-route-query'
import type {
	AnalyticsDashboardStat,
	AnalyticsSelectedBreakdowns,
} from '~/providers/analytics/analytics'

import { isUnknownAnalyticsBreakdownValue } from '../breakdown'
import { getAnalyticsTableMetricSortedGraphDatasetIds } from './analytics-table-sorting'
import type { AnalyticsTableColumnKey, AnalyticsTableRow } from './analytics-table-types'

type UseAnalyticsTableGraphSelectionOptions = {
	sortedRows: ComputedRef<AnalyticsTableRow[]>
	filteredRows: ComputedRef<AnalyticsTableRow[]>
	sortColumn: Ref<AnalyticsTableColumnKey | undefined>
	showGraphDatasetSelection: ComputedRef<boolean>
	selectedGraphDatasetIds: Ref<string[]>
	hasExplicitGraphDatasetSelection: Ref<boolean>
	isGraphDatasetSelectionActive: Ref<boolean>
	defaultGraphDatasetIds: Ref<string[]>
	topGraphDatasetIds: Ref<string[]>
	queryResetToken: Ref<number>
	currentSelectedBreakdowns: Ref<AnalyticsSelectedBreakdowns>
	currentSelectedProjectIds: Ref<string[]>
	activeStat: Ref<AnalyticsDashboardStat>
	sortCollator: Intl.Collator
	hasTableSortQuery: () => boolean
	applyActiveStatSort: () => void
	graphDatasetSelectionLimit: number
}

export function useAnalyticsTableGraphSelection({
	sortedRows,
	filteredRows,
	sortColumn,
	showGraphDatasetSelection,
	selectedGraphDatasetIds,
	hasExplicitGraphDatasetSelection,
	isGraphDatasetSelectionActive,
	defaultGraphDatasetIds,
	topGraphDatasetIds,
	queryResetToken,
	currentSelectedBreakdowns,
	currentSelectedProjectIds,
	activeStat,
	sortCollator,
	hasTableSortQuery,
	applyActiveStatSort,
	graphDatasetSelectionLimit,
}: UseAnalyticsTableGraphSelectionOptions): {
	filteredSelectableGraphDatasetIds: ComputedRef<string[]>
	tableSelectedGraphDatasetIds: WritableComputedRef<unknown[]>
} {
	const selectableGraphDatasetIds = computed(() =>
		getAnalyticsTableSelectableGraphDatasetIds(sortedRows.value),
	)
	const filteredSelectableGraphDatasetIds = computed(() =>
		getAnalyticsTableSelectableGraphDatasetIds(filteredRows.value),
	)
	const unknownGraphDatasetIds = computed(() =>
		getAnalyticsTableUnknownGraphDatasetIds(sortedRows.value),
	)
	const sortedMetricGraphDatasetIds = computed(() =>
		getAnalyticsTableMetricSortedGraphDatasetIds(sortedRows.value, sortColumn.value, sortCollator),
	)
	const defaultSelectedGraphDatasetIds = computed(() => {
		const sortedMetricIds = sortedMetricGraphDatasetIds.value
		const defaultIds =
			sortedMetricIds.length > 0 ? sortedMetricIds : selectableGraphDatasetIds.value
		return defaultIds
			.filter((id) => !unknownGraphDatasetIds.value.has(id))
			.slice(0, graphDatasetSelectionLimit)
	})
	const tableSelectedGraphDatasetIds = computed<unknown[]>({
		get: () => selectedGraphDatasetIds.value,
		set: (ids) => {
			const nextGraphDatasetIds = ids.filter((id): id is string => typeof id === 'string')
			if (showGraphDatasetSelection.value && isDefaultGraphDatasetSelection(nextGraphDatasetIds)) {
				setSelectedGraphDatasetIds(defaultSelectedGraphDatasetIds.value, false)
				return
			}

			selectedGraphDatasetIds.value = nextGraphDatasetIds
			hasExplicitGraphDatasetSelection.value = showGraphDatasetSelection.value
		},
	})

	function setSelectedGraphDatasetIds(ids: string[], explicit: boolean) {
		selectedGraphDatasetIds.value = ids
		hasExplicitGraphDatasetSelection.value = explicit
	}

	function resetGraphDatasetSelection() {
		setSelectedGraphDatasetIds([], false)
	}

	function isDefaultGraphDatasetSelection(ids: string[]) {
		const defaultIds = defaultSelectedGraphDatasetIds.value
		if (defaultIds.length === 0 || ids.length !== defaultIds.length) {
			return false
		}

		const selectedIdSet = new Set(ids)
		return defaultIds.every((id) => selectedIdSet.has(id))
	}

	watch(
		[showGraphDatasetSelection, queryResetToken],
		([nextShowSelection]) => {
			isGraphDatasetSelectionActive.value = nextShowSelection
		},
		{ immediate: true },
	)

	watch(activeStat, () => {
		if (!showGraphDatasetSelection.value) {
			return
		}
		if (hasTableSortQuery()) {
			return
		}

		applyActiveStatSort()
	})

	watch(
		currentSelectedBreakdowns,
		(nextBreakdowns, previousBreakdowns) => {
			if (areStringArraysEqual([...nextBreakdowns], [...(previousBreakdowns ?? [])])) {
				return
			}

			resetGraphDatasetSelection()
		},
		{ deep: true },
	)

	watch(
		currentSelectedProjectIds,
		(nextProjectIds, previousProjectIds) => {
			if (areStringArraysEqual(nextProjectIds, previousProjectIds ?? [])) {
				return
			}

			resetGraphDatasetSelection()
		},
		{ deep: true },
	)

	watch(
		[defaultSelectedGraphDatasetIds, sortedMetricGraphDatasetIds, showGraphDatasetSelection],
		([nextDefaultGraphDatasetIds, nextTopGraphDatasetIds, nextShowGraphDatasetSelection]) => {
			defaultGraphDatasetIds.value = nextShowGraphDatasetSelection
				? [...nextDefaultGraphDatasetIds]
				: []
			topGraphDatasetIds.value = nextShowGraphDatasetSelection
				? nextTopGraphDatasetIds.filter((id) => !unknownGraphDatasetIds.value.has(id))
				: []
		},
		{ immediate: true },
	)

	watch(
		[
			defaultSelectedGraphDatasetIds,
			showGraphDatasetSelection,
			hasExplicitGraphDatasetSelection,
			queryResetToken,
		],
		([nextDefaultGraphDatasetIds, nextShowGraphDatasetSelection, nextHasExplicitSelection]) => {
			if (!nextShowGraphDatasetSelection) {
				return
			}

			if (nextHasExplicitSelection) {
				if (isDefaultGraphDatasetSelection(selectedGraphDatasetIds.value)) {
					setSelectedGraphDatasetIds(nextDefaultGraphDatasetIds, false)
				}
				return
			}

			if (!areStringArraysEqual(selectedGraphDatasetIds.value, nextDefaultGraphDatasetIds)) {
				setSelectedGraphDatasetIds(nextDefaultGraphDatasetIds, false)
			}
		},
		{ immediate: true },
	)

	function getAnalyticsTableSelectableGraphDatasetIds(rows: AnalyticsTableRow[]): string[] {
		return Array.from(new Set(rows.map((row) => row.graphDatasetId)))
	}

	function getAnalyticsTableUnknownGraphDatasetIds(rows: AnalyticsTableRow[]): Set<string> {
		return new Set(
			rows
				.filter((row) =>
					Object.values(row.breakdownValues).some((value) =>
						isUnknownAnalyticsBreakdownValue(value),
					),
				)
				.map((row) => row.graphDatasetId),
		)
	}

	return {
		filteredSelectableGraphDatasetIds,
		tableSelectedGraphDatasetIds,
	}
}
