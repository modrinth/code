import type { Ref } from 'vue'
import { computed, ref } from 'vue'

import type { FileItem } from '../types'

export function useFileSelection(items: Ref<FileItem[]>) {
	const selectedItems = ref<Set<string>>(new Set())

	function toggleItemSelection(path: string) {
		const newSet = new Set(selectedItems.value)
		if (newSet.has(path)) {
			newSet.delete(path)
		} else {
			newSet.add(path)
		}
		selectedItems.value = newSet
	}

	function selectAll() {
		selectedItems.value = new Set(items.value.map((i) => i.path))
	}

	function deselectAll() {
		selectedItems.value = new Set()
	}

	function toggleSelectAll() {
		if (allSelected.value) {
			deselectAll()
		} else {
			selectAll()
		}
	}

	const allSelected = computed(
		() => items.value.length > 0 && selectedItems.value.size === items.value.length,
	)

	const someSelected = computed(
		() => selectedItems.value.size > 0 && selectedItems.value.size < items.value.length,
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
