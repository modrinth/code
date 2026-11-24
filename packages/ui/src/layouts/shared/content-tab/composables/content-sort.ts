import { useSessionStorage } from '@vueuse/core'
import type { Ref } from 'vue'
import { computed, ref } from 'vue'

import type { ContentItem } from '../types'

export type ContentSortField = 'name' | 'author' | 'updated' | 'added'
export type ContentSortDirection = 'asc' | 'desc'

export interface ContentSortMode {
	field: ContentSortField
	direction: ContentSortDirection
}

export const contentSortFields: readonly ContentSortField[] = ['name', 'author', 'updated', 'added']

export const defaultContentSortMode: ContentSortMode = { field: 'name', direction: 'asc' }

export function isContentSortMode(value: unknown): value is ContentSortMode {
	if (!value || typeof value !== 'object') return false
	const { field, direction } = value as Partial<ContentSortMode>
	return (
		contentSortFields.includes(field as ContentSortField) &&
		(direction === 'asc' || direction === 'desc')
	)
}

export function getContentItemDisplayName(item: ContentItem) {
	return item.project?.title ?? item.embedded_metadata?.name ?? item.file_name
}

const sortKeys: Record<ContentSortField, (item: ContentItem) => string | undefined> = {
	name: (item) => getContentItemDisplayName(item).toLowerCase(),
	author: (item) => item.owner?.name.toLowerCase(),
	updated: (item) => item.version?.date_published,
	added: (item) => item.date_added,
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
	const key = sortKeys[mode.field]
	const sign = mode.direction === 'asc' ? 1 : -1
	return [...items].sort((a, b) => {
		const aKey = key(a)
		const bKey = key(b)
		if (aKey && bKey) return sign * aKey.localeCompare(bKey) || compareByName(a, b)
		if (aKey || bKey) return aKey ? -1 : 1
		return compareByName(a, b)
	})
}

export function getAvailableContentSortFields(items: ContentItem[]) {
	return contentSortFields.filter(
		(field) => field === 'name' || items.some((item) => !!sortKeys[field](item)),
	)
}

/**
 * Sorts content items by the selected mode, defaulting to name A-Z. When `persistKey` is provided
 * the mode is kept in session storage.
 */
export function useContentSort(items: Ref<ContentItem[]>, persistKey?: string) {
	const sortMode = persistKey
		? useSessionStorage<ContentSortMode>(`content-sort:${persistKey}`, defaultContentSortMode)
		: ref<ContentSortMode>(defaultContentSortMode)
	if (!isContentSortMode(sortMode.value)) sortMode.value = defaultContentSortMode

	const sortFields = computed(() => getAvailableContentSortFields(items.value))
	const sortedItems = computed(() => sortContentItems(items.value, sortMode.value))

	function resetSortMode() {
		sortMode.value = defaultContentSortMode
	}

	return { sortMode, sortedItems, sortFields, resetSortMode }
}
