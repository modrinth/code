import type { Ref } from 'vue'
import { computed, ref } from 'vue'

import type { FileItem } from '../types'

export function useFileSearch(items: Ref<FileItem[]>) {
	const searchQuery = ref('')

	const searchedItems = computed(() => {
		if (!searchQuery.value) return items.value

		const query = searchQuery.value.toLowerCase()
		return items.value.filter((item) => item.name.toLowerCase().includes(query))
	})

	return {
		searchQuery,
		searchedItems,
	}
}
