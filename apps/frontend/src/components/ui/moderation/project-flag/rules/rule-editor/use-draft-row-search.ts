import Fuse from 'fuse.js'
import { computed, type Ref, ref, shallowRef, watch } from 'vue'

export function useDraftRowSearch<T extends object>(entries: Ref<T[]>, keys: (keyof T & string)[]) {
	const search = ref<string>()
	const showSearch = computed(() => entries.value.length > 6)
	const focusedEntry = shallowRef<T>()
	const searchIndex = computed(
		() => new Fuse(entries.value, { keys, threshold: 0.3, ignoreLocation: true }),
	)
	const visibleRows = computed(() => {
		const rows = entries.value.map((entry, index) => ({ entry, index }))
		const query = search.value?.trim()
		if (!showSearch.value || !query) return rows

		const matches = new Set(searchIndex.value.search(query).map(({ refIndex }) => refIndex))
		return rows.filter(({ entry, index }) => matches.has(index) || entry === focusedEntry.value)
	})

	watch(showSearch, (visible) => {
		if (!visible) search.value = ''
	})

	function onRowFocusOut(event: FocusEvent) {
		const row = event.currentTarget as HTMLElement
		if (!row.contains(event.relatedTarget as Node | null)) focusedEntry.value = undefined
	}

	return { search, showSearch, visibleRows, focusedEntry, onRowFocusOut }
}
