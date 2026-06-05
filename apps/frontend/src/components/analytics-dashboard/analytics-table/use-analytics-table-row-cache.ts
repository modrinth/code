import type { ComputedRef, Ref, ShallowRef } from 'vue'
import { ref, shallowRef } from 'vue'

import type {
	AnalyticsTableColumnKey,
	AnalyticsTableDisplayedRowsCache,
	AnalyticsTableMode,
	AnalyticsTableRow,
	AnalyticsTableSortDirectionValue,
} from './analytics-table-types'

type UseAnalyticsTableRowCacheOptions = {
	activeTableMode: ComputedRef<AnalyticsTableMode>
	showBreakdownColumn: ComputedRef<boolean>
	analyticsPointCount: ComputedRef<number>
	sortColumn: Ref<AnalyticsTableColumnKey | undefined>
	sortDirection: Ref<AnalyticsTableSortDirectionValue>
	buildRows: (mode: AnalyticsTableMode) => AnalyticsTableRow[]
	sortRows: (rows: AnalyticsTableRow[]) => AnalyticsTableRow[]
	inactiveModeWarmupPointLimit: number
}

export function useAnalyticsTableRowCache({
	activeTableMode,
	showBreakdownColumn,
	analyticsPointCount,
	sortColumn,
	sortDirection,
	buildRows,
	sortRows,
	inactiveModeWarmupPointLimit,
}: UseAnalyticsTableRowCacheOptions): {
	displayedTableMode: Ref<AnalyticsTableMode>
	displayedSortColumn: Ref<AnalyticsTableColumnKey | undefined>
	displayedSortDirection: Ref<AnalyticsTableSortDirectionValue>
	displayedSortedRows: ShallowRef<AnalyticsTableRow[]>
	invalidateTableCaches: () => void
	invalidateSortedCaches: () => void
	scheduleRowsForMode: (mode: AnalyticsTableMode) => void
	scheduleInactiveModeWarmup: () => void
	resortDisplayedRowsForCurrentSort: () => boolean
	getSortedRowsForMode: (mode: AnalyticsTableMode) => AnalyticsTableRow[]
} {
	const modeBuildRequestIds: Record<AnalyticsTableMode, number> = {
		date_breakdown: 0,
		breakdown_only: 0,
	}
	let tableCacheGeneration = 0
	let displayedSortedRowsGeneration = 0
	const displayedTableMode = ref<AnalyticsTableMode>('breakdown_only')
	const displayedSortColumn = ref<AnalyticsTableColumnKey | undefined>(sortColumn.value)
	const displayedSortDirection = ref<AnalyticsTableSortDirectionValue>(sortDirection.value)
	const displayedSortedRows = shallowRef<AnalyticsTableRow[]>([])
	const displayedRowsCache = shallowRef<AnalyticsTableDisplayedRowsCache | null>(null)

	function invalidateTableCaches() {
		tableCacheGeneration++
		invalidateSortedCaches()
	}

	function invalidateSortedCaches() {
		displayedRowsCache.value = null
	}

	function hasSortedRowsForMode(mode: AnalyticsTableMode): boolean {
		const cached = displayedRowsCache.value
		return (
			cached !== null &&
			cached.generation === tableCacheGeneration &&
			cached.mode === mode &&
			cached.sortColumn === sortColumn.value &&
			cached.sortDirection === sortDirection.value
		)
	}

	function setDisplayedRowsForMode(
		mode: AnalyticsTableMode,
		rows: AnalyticsTableRow[],
		generation = tableCacheGeneration,
	) {
		displayedRowsCache.value = {
			generation,
			mode,
			sortColumn: sortColumn.value,
			sortDirection: sortDirection.value,
			rows,
		}

		if (mode === activeTableMode.value) {
			displayedSortedRowsGeneration = generation
			displayedTableMode.value = mode
			displayedSortColumn.value = sortColumn.value
			displayedSortDirection.value = sortDirection.value
			displayedSortedRows.value = rows
		}
	}

	function scheduleRowsForMode(mode: AnalyticsTableMode) {
		if (hasSortedRowsForMode(mode)) {
			if (mode === activeTableMode.value) {
				displayRowsForMode(mode)
			}
			return
		}

		const requestId = ++modeBuildRequestIds[mode]
		const generation = tableCacheGeneration

		void buildRowsForMode(mode, generation, requestId)
	}

	function displayRowsForMode(mode: AnalyticsTableMode) {
		const cached = displayedRowsCache.value
		if (!cached || cached.generation !== tableCacheGeneration || cached.mode !== mode) {
			return
		}

		displayedSortedRowsGeneration = cached.generation
		displayedTableMode.value = mode
		displayedSortColumn.value = cached.sortColumn
		displayedSortDirection.value = cached.sortDirection
		displayedSortedRows.value = cached.rows
	}

	async function buildRowsForMode(mode: AnalyticsTableMode, generation: number, requestId: number) {
		await waitForDeferredTableWork()

		if (isStaleBuild(mode, generation, requestId)) {
			return
		}

		const rows = sortRows(buildRows(mode))

		if (isStaleBuild(mode, generation, requestId)) {
			return
		}

		setDisplayedRowsForMode(mode, rows, generation)
	}

	function isStaleBuild(mode: AnalyticsTableMode, generation: number, requestId: number): boolean {
		return tableCacheGeneration !== generation || modeBuildRequestIds[mode] !== requestId
	}

	function waitForDeferredTableWork(): Promise<void> {
		if (!import.meta.client) {
			return Promise.resolve()
		}

		return new Promise((resolve) => {
			requestAnimationFrame(() => {
				requestAnimationFrame(() => resolve())
			})
		})
	}

	function scheduleInactiveModeWarmup() {
		if (!showBreakdownColumn.value) {
			return
		}
		if (analyticsPointCount.value > inactiveModeWarmupPointLimit) {
			return
		}

		const inactiveMode: AnalyticsTableMode =
			activeTableMode.value === 'date_breakdown' ? 'breakdown_only' : 'date_breakdown'

		if (hasSortedRowsForMode(inactiveMode)) {
			return
		}

		if (!import.meta.client) {
			scheduleRowsForMode(inactiveMode)
			return
		}

		const windowWithIdleCallback = window as Window & {
			requestIdleCallback?: (callback: () => void, options?: { timeout?: number }) => number
		}

		if (windowWithIdleCallback.requestIdleCallback) {
			windowWithIdleCallback.requestIdleCallback(() => scheduleRowsForMode(inactiveMode), {
				timeout: 2000,
			})
		} else {
			window.setTimeout(() => scheduleRowsForMode(inactiveMode), 250)
		}
	}

	function resortDisplayedRowsForCurrentSort(): boolean {
		const mode = activeTableMode.value
		if (
			displayedTableMode.value !== mode ||
			displayedSortedRowsGeneration !== tableCacheGeneration
		) {
			return false
		}

		setDisplayedRowsForMode(mode, sortRows(displayedSortedRows.value))
		return true
	}

	function getSortedRowsForMode(mode: AnalyticsTableMode): AnalyticsTableRow[] {
		const cached = displayedRowsCache.value
		if (
			cached &&
			cached.generation === tableCacheGeneration &&
			cached.mode === mode &&
			cached.sortColumn === sortColumn.value &&
			cached.sortDirection === sortDirection.value
		) {
			return cached.rows
		}

		return sortRows(buildRows(mode))
	}

	return {
		displayedTableMode,
		displayedSortColumn,
		displayedSortDirection,
		displayedSortedRows,
		invalidateTableCaches,
		invalidateSortedCaches,
		scheduleRowsForMode,
		scheduleInactiveModeWarmup,
		resortDisplayedRowsForCurrentSort,
		getSortedRowsForMode,
	}
}
