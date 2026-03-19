import type { Ref } from 'vue'
import { computed, ref } from 'vue'

import type { FileItem, FileSortField, FileViewFilter } from '../types'

export function useFileSorting(items: Ref<FileItem[]>) {
	const sortField = ref<FileSortField>('name')
	const sortDesc = ref(false)
	const viewFilter = ref<FileViewFilter>('all')

	function handleSort(field: FileSortField) {
		if (sortField.value === field) {
			sortDesc.value = !sortDesc.value
		} else {
			sortField.value = field
			sortDesc.value = false
		}
	}

	function resetSort() {
		sortField.value = 'name'
		sortDesc.value = false
		viewFilter.value = 'all'
	}

	const sortedItems = computed(() => {
		let result = [...items.value]

		switch (viewFilter.value) {
			case 'filesOnly':
				result = result.filter((item) => item.type !== 'directory')
				break
			case 'foldersOnly':
				result = result.filter((item) => item.type === 'directory')
				break
		}

		function compareItems(a: FileItem, b: FileItem) {
			if (viewFilter.value === 'all') {
				if (a.type === 'directory' && b.type !== 'directory') return -1
				if (a.type !== 'directory' && b.type === 'directory') return 1
			}

			switch (sortField.value) {
				case 'modified':
					return sortDesc.value ? a.modified - b.modified : b.modified - a.modified
				case 'created':
					return sortDesc.value ? a.created - b.created : b.created - a.created
				case 'size': {
					const aValue =
						a.type === 'directory'
							? a.count !== undefined
								? a.count
								: 0
							: a.size !== undefined
								? a.size
								: 0
					const bValue =
						b.type === 'directory'
							? b.count !== undefined
								? b.count
								: 0
							: b.size !== undefined
								? b.size
								: 0
					return sortDesc.value ? aValue - bValue : bValue - aValue
				}
				default:
					return sortDesc.value ? b.name.localeCompare(a.name) : a.name.localeCompare(b.name)
			}
		}

		result.sort(compareItems)
		return result
	})

	return {
		sortField,
		sortDesc,
		viewFilter,
		sortedItems,
		handleSort,
		resetSort,
	}
}
