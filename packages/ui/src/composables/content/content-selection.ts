import type { Ref } from 'vue'
import { computed, ref } from 'vue'
import type { ContentItem } from '../../components/instances/types'

export function useContentSelection(
	items: Ref<ContentItem[]>,
	getItemId: (item: ContentItem) => string,
) {
	const selectedIds = ref<string[]>([])

	const selectedItems = computed(() =>
		items.value.filter((item) =>
			selectedIds.value.includes(getItemId(item)),
		),
	)

	function clearSelection() {
		selectedIds.value = []
	}

	function removeFromSelection(id: string) {
		selectedIds.value = selectedIds.value.filter((i) => i !== id)
	}

	return { selectedIds, selectedItems, clearSelection, removeFromSelection }
}
