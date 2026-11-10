import { useSessionStorage } from '@vueuse/core'
import type { Ref } from 'vue'
import { computed, ref } from 'vue'

import type { ContentItem } from '../types'

export type ContentSortMode =
	| 'alphabetical-asc'
	| 'alphabetical-desc'
	| 'date-added-newest'
	| 'date-added-oldest'

export const contentSortModes: readonly ContentSortMode[] = [
	'alphabetical-asc',
	'alphabetical-desc',
	'date-added-newest',
	'date-added-oldest',
]

const defaultSortMode: ContentSortMode = 'alphabetical-asc'

export function getContentItemDisplayName(item: ContentItem) {
	return item.project?.title ?? item.embedded_metadata?.name ?? item.file_name
}

function compareByName(a: ContentItem, b: ContentItem) {
	return (
		getContentItemDisplayName(a)
			.toLowerCase()
			.localeCompare(getContentItemDisplayName(b).toLowerCase()) ||
		a.file_name.localeCompare(b.file_name)
	)
}

export function sortContentItems(items: ContentItem[], mode: ContentSortMode) {
	const sorted = [...items]
	switch (mode) {
		case 'alphabetical-desc':
			return sorted.sort((a, b) => compareByName(b, a))
		case 'date-added-newest':
			return sorted.sort(
				(a, b) => (b.date_added ?? '').localeCompare(a.date_added ?? '') || compareByName(a, b),
			)
		case 'date-added-oldest':
			return sorted.sort(
				(a, b) => (a.date_added ?? '').localeCompare(b.date_added ?? '') || compareByName(a, b),
			)
		default:
			return sorted.sort(compareByName)
	}
}

/**
 * Sorts content items by the selected mode, defaulting to A-Z. When `persistKey` is provided the
 * mode is kept in session storage.
 */
export function useContentSort(items: Ref<ContentItem[]>, persistKey?: string) {
	const sortMode = persistKey
		? useSessionStorage<ContentSortMode>(`content-sort:${persistKey}`, defaultSortMode)
		: ref<ContentSortMode>(defaultSortMode)
	if (!contentSortModes.includes(sortMode.value)) sortMode.value = defaultSortMode

	const hasDateAdded = computed(() => items.value.some((item) => !!item.date_added))
	const sortedItems = computed(() => sortContentItems(items.value, sortMode.value))

	function resetSortMode() {
		sortMode.value = defaultSortMode
	}

	return { sortMode, sortedItems, hasDateAdded, resetSortMode }
}
