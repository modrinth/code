import { computed, type Ref, ref, watch } from 'vue'

import type { SortDirection } from '#ui/components/base/Table.vue'

import type { InvitedPlayerMethod, InvitedPlayerRow } from '../types'

type MethodFilter = InvitedPlayerMethod | 'all'

export function useInvitedPlayersTable(
	rows: Ref<InvitedPlayerRow[]>,
	formatRelativeTime: (date: Date) => string,
	labels: Ref<{ methods: Record<InvitedPlayerMethod, string>; never: string; pending: string }>,
) {
	const search = ref('')
	const methodFilter = ref<MethodFilter>('all')
	const sortColumn = ref<string | undefined>('joined')
	const sortDirection = ref<SortDirection>('desc')
	const methodFilterOptions = computed<Array<{ id: InvitedPlayerMethod; label: string }>>(() => [
		{ id: 'direct', label: labels.value.methods.direct },
		{ id: 'link', label: labels.value.methods.link },
	])
	const hasMultipleMethods = computed(() => new Set(rows.value.map((row) => row.method)).size > 1)
	const filteredRows = computed(() => {
		const query = search.value.trim().toLowerCase()
		return rows.value.filter((row) => {
			if (methodFilter.value !== 'all' && row.method !== methodFilter.value) return false
			if (!query) return true
			return [
				row.username,
				row.lastPlayedAt ? formatRelativeTime(row.lastPlayedAt) : labels.value.never,
				row.pending ? labels.value.pending : row.joinedAt ? formatRelativeTime(row.joinedAt) : '',
				labels.value.methods[row.method],
			].some((value) => value.toLowerCase().includes(query))
		})
	})
	const sortedRows = computed(() => [...filteredRows.value].sort(compareRows))

	function compareRows(a: InvitedPlayerRow, b: InvitedPlayerRow) {
		let compared: number
		if (sortColumn.value === 'username') compared = a.username.localeCompare(b.username)
		else if (sortColumn.value === 'lastPlayed')
			compared =
				(a.lastPlayedAt?.getTime() ?? Number.NEGATIVE_INFINITY) -
				(b.lastPlayedAt?.getTime() ?? Number.NEGATIVE_INFINITY)
		else if (sortColumn.value === 'method')
			compared = labels.value.methods[a.method].localeCompare(labels.value.methods[b.method])
		else
			compared =
				(a.pending
					? Number.MAX_SAFE_INTEGER
					: (a.joinedAt?.getTime() ?? Number.NEGATIVE_INFINITY)) -
					(b.pending
						? Number.MAX_SAFE_INTEGER
						: (b.joinedAt?.getTime() ?? Number.NEGATIVE_INFINITY)) ||
				a.username.localeCompare(b.username)
		return sortDirection.value === 'asc' ? compared : -compared
	}
	function toggleMethodFilter(filter: InvitedPlayerMethod) {
		methodFilter.value = methodFilter.value === filter ? 'all' : filter
	}

	watch(hasMultipleMethods, (multiple) => {
		if (!multiple) methodFilter.value = 'all'
	})

	return {
		search,
		methodFilter,
		sortColumn,
		sortDirection,
		methodFilterOptions,
		hasMultipleMethods,
		sortedRows,
		toggleMethodFilter,
	}
}
