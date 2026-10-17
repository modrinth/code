import type { ComputedRef, Ref } from 'vue'
import { computed, ref } from 'vue'

import type { FileItem } from '../types'

export interface FileListingSelection {
	allSelected: ComputedRef<boolean>
	someSelected: ComputedRef<boolean>
	toggleSelectAll: () => void
}

/**
 * Selection of file entries, keyed by path. Entries can come from any directory (e.g. the sidebar
 * tree, or several tabs' listings), so the items themselves are kept rather than looked up in a
 * listing. `forListing` gives a listing its select-all state over the entries it shows.
 */
export function useFileSelection() {
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

	function deselectAll() {
		selectedItems.value = new Map()
	}

	function forListing(items: Ref<FileItem[]>): FileListingSelection {
		const allSelected = computed(
			() =>
				items.value.length > 0 && items.value.every((item) => selectedItems.value.has(item.path)),
		)
		const someSelected = computed(
			() => !allSelected.value && items.value.some((item) => selectedItems.value.has(item.path)),
		)

		function toggleSelectAll() {
			const next = new Map(selectedItems.value)
			if (allSelected.value) {
				for (const item of items.value) next.delete(item.path)
			} else {
				for (const item of items.value) next.set(item.path, item)
			}
			selectedItems.value = next
		}

		return { allSelected, someSelected, toggleSelectAll }
	}

	return {
		selectedItems,
		toggleItemSelection,
		deselectAll,
		forListing,
	}
}
