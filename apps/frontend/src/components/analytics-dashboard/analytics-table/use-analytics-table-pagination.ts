import type { ComputedRef, Ref } from 'vue'
import { computed, ref, watch } from 'vue'

import type { AnalyticsTableRow } from './analytics-table-types'

type UseAnalyticsTablePaginationOptions = {
	filteredRows: ComputedRef<AnalyticsTableRow[]>
	pageSize: number
}

export function useAnalyticsTablePagination({
	filteredRows,
	pageSize,
}: UseAnalyticsTablePaginationOptions): {
	currentPage: Ref<number>
	pageCount: ComputedRef<number>
	visibleRowStart: ComputedRef<number>
	visibleRowEnd: ComputedRef<number>
	paginatedRows: ComputedRef<AnalyticsTableRow[]>
	switchPage: (page: number) => void
} {
	const currentPage = ref(1)
	const pageCount = computed(() => Math.max(Math.ceil(filteredRows.value.length / pageSize), 1))
	const visibleRowStart = computed(() =>
		filteredRows.value.length === 0 ? 0 : (currentPage.value - 1) * pageSize + 1,
	)
	const visibleRowEnd = computed(() =>
		Math.min(currentPage.value * pageSize, filteredRows.value.length),
	)
	const paginatedRows = computed<AnalyticsTableRow[]>(() =>
		filteredRows.value.slice((currentPage.value - 1) * pageSize, currentPage.value * pageSize),
	)

	watch(filteredRows, () => {
		currentPage.value = 1
	})

	watch(pageCount, (nextPageCount) => {
		if (currentPage.value > nextPageCount) {
			currentPage.value = nextPageCount
		}
	})

	function switchPage(page: number) {
		currentPage.value = page
	}

	return {
		currentPage,
		pageCount,
		visibleRowStart,
		visibleRowEnd,
		paginatedRows,
		switchPage,
	}
}
