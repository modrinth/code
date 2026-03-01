import type { Ref } from 'vue'
import { computed, ref, watch } from 'vue'

import type { ContentItem } from '../../components/instances/types'

export function useContentSelection(
	items: Ref<ContentItem[]>,
	getItemId: (item: ContentItem) => string,
) {
	const selectedIds = ref<string[]>([])

	const selectedItems = computed(() =>
		items.value.filter((item) => selectedIds.value.includes(getItemId(item))),
	)

	watch(items, (newItems) => {
		if (selectedIds.value.length === 0) return
		const validIds = new Set(newItems.map(getItemId))
		const pruned = selectedIds.value.filter((id) => validIds.has(id))
		if (pruned.length !== selectedIds.value.length) {
			selectedIds.value = pruned
		}
	})

	function clearSelection() {
		selectedIds.value = []
	}

	function removeFromSelection(id: string) {
		selectedIds.value = selectedIds.value.filter((i) => i !== id)
	}

	return { selectedIds, selectedItems, clearSelection, removeFromSelection }
}
