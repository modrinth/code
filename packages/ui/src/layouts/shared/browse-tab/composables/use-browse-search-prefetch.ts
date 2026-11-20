import type { Ref } from 'vue'
import { computed, ref } from 'vue'

import type {
	EnvironmentSearchOverride,
	FilterType,
	FilterValue,
	ProjectType,
} from '#ui/utils/search'
import { useSearch } from '#ui/utils/search'
import { SERVER_SORT_TYPES, useServerSearch } from '#ui/utils/server-search'

import type { BrowseTags } from '../types'

export interface BrowseSearchPrefetchOptions {
	projectTypes: Readonly<Ref<string[]>>
	providedFilters: (projectType: string) => FilterValue[]
	environmentOverride?: (projectType: string) => EnvironmentSearchOverride | undefined
	maxResultsOptions?: (projectType: string) => number[]
}

export interface BrowseSearchPrefetchState {
	tags: Readonly<Ref<BrowseTags>>
	maxResults: Readonly<Ref<number>>
	projectFilters: Readonly<Ref<FilterValue[]>>
	serverFilters: Readonly<Ref<FilterValue[]>>
	overriddenProvidedFilterTypes: Readonly<Ref<string[]>>
	withSavedAdvancedPrefs: (filters: FilterValue[], filterTypes: FilterType[]) => FilterValue[]
}

export function closestMaxResults(current: number, available: number[]) {
	if (available.includes(current)) return current
	return available.reduce((closest, candidate) =>
		Math.abs(candidate - current) <= Math.abs(closest - current) ? candidate : closest,
	)
}

export function useBrowseSearchPrefetch(
	options: BrowseSearchPrefetchOptions,
	state: BrowseSearchPrefetchState,
) {
	const tags = computed(() => state.tags.value)
	const probeType = ref<ProjectType>('mod')

	const projectProbe = useSearch(
		computed(() => [probeType.value]),
		tags,
		computed(() => options.providedFilters(probeType.value)),
		computed(() => options.environmentOverride?.(probeType.value)),
		{ readRouteQuery: false },
	)

	const serverQuery = ref('')
	const serverMaxResults = ref(20)
	const serverPage = ref(1)
	const serverProbe = useServerSearch({
		tags,
		query: serverQuery,
		maxResults: serverMaxResults,
		currentPage: serverPage,
		providedFilters: computed(() => options.providedFilters('server')),
	})

	function maxResultsFor(projectType: string) {
		const available = options.maxResultsOptions?.(projectType)
		return available ? closestMaxResults(state.maxResults.value, available) : state.maxResults.value
	}

	function serverRequestParams() {
		serverQuery.value = ''
		serverPage.value = 1
		serverMaxResults.value = maxResultsFor('server')
		serverProbe.serverCurrentSortType.value = SERVER_SORT_TYPES[0]
		serverProbe.serverCurrentFilters.value = [...state.serverFilters.value]
		return serverProbe.serverRequestParams.value
	}

	function projectRequestParams(projectType: string) {
		probeType.value = projectType as ProjectType
		const filterTypes = projectProbe.filters.value
		projectProbe.maxResults.value = maxResultsFor(projectType)
		projectProbe.overriddenProvidedFilterTypes.value = [
			...state.overriddenProvidedFilterTypes.value,
		]
		projectProbe.currentFilters.value = state.withSavedAdvancedPrefs(
			state.projectFilters.value.filter((filter) =>
				filterTypes.some((filterType) => filterType.id === filter.type),
			),
			filterTypes,
		)
		return projectProbe.requestParams.value
	}

	function initialRequestParams(projectType: string) {
		return projectType === 'server' ? serverRequestParams() : projectRequestParams(projectType)
	}

	return { initialRequestParams }
}
