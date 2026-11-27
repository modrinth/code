import { useSessionStorage } from '@vueuse/core'
import type { Ref } from 'vue'
import { computed, ref, watch } from 'vue'

import { useDebugLogger } from '#ui/composables/debug-logger'
import { normalizeProjectType } from '#ui/utils/common-messages'

import type { ContentItem, ContentType } from '../types'
import { syncRefToRouteQuery, useContentSearch } from './content-search'
import {
	type ContentSortMode,
	defaultContentSortMode,
	getAvailableContentSortFields,
	isContentSortMode,
	sortContentItems,
} from './content-sort'
import {
	type ContentMetadataFilterValue,
	useContentMetadataFilters,
} from './use-content-metadata-filters'

export const contentTypeOrder: ContentType[] = [
	'mod',
	'plugin',
	'resourcepack',
	'datapack',
	'shader',
]

export function contentTypeOf(item: ContentItem) {
	return normalizeProjectType(item.project_type) as ContentType
}

export interface ContentViewStateOptions {
	items: Readonly<Ref<ContentItem[]>>
	types: Readonly<Ref<ContentType[]>>
	persistKey?: string
	searchQueryParam?: string
	showSharedContentFilter?: Readonly<Ref<boolean>>
	showEnabledFor: boolean
	showEnvironmentWarnings: Readonly<Ref<boolean>>
}

function persisted<T>(key: string | undefined, initial: T) {
	return key ? useSessionStorage<T>(key, initial) : ref(initial)
}

export function useContentViewState(options: ContentViewStateOptions) {
	const debug = useDebugLogger('ContentTypeDefault')
	const types = computed<ContentType[]>(() => {
		const provided = options.types.value
		const present = new Set(options.items.value.map(contentTypeOf))
		return [
			...provided,
			...contentTypeOrder.filter((type) => present.has(type) && !provided.includes(type)),
		]
	})

	const defaultType = types.value[0] ?? 'mod'
	const activeType = ref<ContentType>(defaultType)
	syncRefToRouteQuery(activeType, 'type', defaultType, (candidate): candidate is ContentType =>
		types.value.includes(candidate as ContentType),
	)
	watch(types, (next) => {
		if (next.length > 0 && !next.includes(activeType.value)) activeType.value = next[0]
	})

	const typeItems = computed(() =>
		options.items.value.filter((item) => contentTypeOf(item) === activeType.value),
	)

	let autoSelectedType: ContentType | null = activeType.value === defaultType ? defaultType : null
	debug('setup', {
		defaultType,
		activeType: activeType.value,
		types: types.value,
		autoSelectedType,
		itemCount: options.items.value.length,
	})
	watch(activeType, (next, previous) => {
		debug('activeType changed', { previous, next, autoSelectedType, stack: new Error().stack })
		if (next !== autoSelectedType) autoSelectedType = null
	})
	watch(
		options.items,
		(items) => {
			debug('items changed', {
				autoSelectedType,
				activeType: activeType.value,
				typeItemCount: typeItems.value.length,
				itemTypes: items.map((item) => `${item.project_type}->${contentTypeOf(item)}`),
				types: types.value,
			})
			if (autoSelectedType === null || typeItems.value.length > 0) return
			const populated = types.value.find((type) =>
				items.some((item) => contentTypeOf(item) === type),
			)
			if (!populated) return
			autoSelectedType = populated
			activeType.value = populated
		},
		{ immediate: true },
	)

	const sortModes = persisted<Partial<Record<ContentType, ContentSortMode>>>(
		options.persistKey ? `content-sort-by-type:${options.persistKey}` : undefined,
		{},
	)
	const sortMode = computed<ContentSortMode>({
		get: () => {
			const mode = sortModes.value[activeType.value]
			return isContentSortMode(mode) ? mode : defaultContentSortMode
		},
		set: (mode) => {
			sortModes.value = { ...sortModes.value, [activeType.value]: mode }
		},
	})

	const filtersByType = persisted<Partial<Record<ContentType, ContentMetadataFilterValue>>>(
		options.persistKey ? `content-filters-by-type:${options.persistKey}` : undefined,
		{},
	)
	const filters = computed<ContentMetadataFilterValue>({
		get: () => filtersByType.value[activeType.value] ?? {},
		set: (value) => {
			filtersByType.value = { ...filtersByType.value, [activeType.value]: value }
		},
	})

	const sortFields = computed(() => getAvailableContentSortFields(typeItems.value))
	const sortedItems = computed(() => sortContentItems(typeItems.value, sortMode.value))

	const { searchQuery, search } = useContentSearch(
		sortedItems,
		['project.title', 'owner.name', 'file_name'],
		{ queryParam: options.searchQueryParam },
	)

	const { metadataFilterCategories, applyMetadataFilters } = useContentMetadataFilters(
		typeItems,
		filters,
		{
			showSharedContent: options.showSharedContentFilter,
			showEnabledFor: options.showEnabledFor,
			showEnvironmentWarnings: options.showEnvironmentWarnings.value,
		},
	)

	const retainedIds = ref(new Set<string>())

	function retain(id: string) {
		retainedIds.value.add(id)
	}

	function clearRetained() {
		retainedIds.value.clear()
	}

	watch([searchQuery, filters, activeType], clearRetained)

	const visibleItems = computed(() => {
		const searched = search(sortedItems.value)
		const matching = applyMetadataFilters(searched)
		if (retainedIds.value.size === 0) return matching
		const matchingIds = new Set(matching.map((item) => item.id))
		return searched.filter((item) => matchingIds.has(item.id) || retainedIds.value.has(item.id))
	})

	function reveal(item: ContentItem) {
		searchQuery.value = ''
		filters.value = {}
		activeType.value = contentTypeOf(item)
	}

	return {
		types,
		activeType,
		typeItems,
		sortMode,
		sortFields,
		searchQuery,
		filters,
		metadataFilterCategories,
		visibleItems,
		retain,
		clearRetained,
		reveal,
	}
}
