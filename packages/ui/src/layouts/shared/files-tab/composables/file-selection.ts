import type { Ref } from 'vue'
import { computed, ref } from 'vue'

import type { FileItem } from '../types'

/**
 * Selection of file entries, keyed by path. Entries can come from any directory (e.g. the sidebar
 * tree), so the items themselves are kept rather than looked up in the current listing.
 * `allSelected` / `someSelected` / `toggleSelectAll` refer to the given listing `items`.
 */
export function useFileSelection(items: Ref<FileItem[]>) {
	const selectedItems = ref(new Map<string, FileItem>())

	function toggleItemSelection(item: FileItem) {
		const next = new Map(selectedItems.value)
		if (next.has(item.path)) {
			next.delete(item.path)
		} else {
			next.set(item.path, item)
		}
		selectedItems.value = next
	}

	function selectAll() {
		const next = new Map(selectedItems.value)
		for (const item of items.value) next.set(item.path, item)
		selectedItems.value = next
	}

	function deselectAll() {
		selectedItems.value = new Map()
	}

	function deselectListed() {
		const next = new Map(selectedItems.value)
		for (const item of items.value) next.delete(item.path)
		selectedItems.value = next
	}

	function toggleSelectAll() {
		if (allSelected.value) {
			deselectListed()
		} else {
			selectAll()
		}
	}

	const allSelected = computed(
		() =>
			items.value.length > 0 && items.value.every((item) => selectedItems.value.has(item.path)),
	)

	const someSelected = computed(
		() => !allSelected.value && items.value.some((item) => selectedItems.value.has(item.path)),
	)

	return {
		selectedItems,
		toggleItemSelection,
		selectAll,
		deselectAll,
		toggleSelectAll,
		allSelected,
		someSelected,
	}
}
