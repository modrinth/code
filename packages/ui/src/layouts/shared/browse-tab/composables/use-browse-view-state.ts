import { keepPreviousData, useQuery, useQueryClient } from '@tanstack/vue-query'
import { useDebounceFn } from '@vueuse/core'
import type { ComputedRef, Ref, WritableComputedRef } from 'vue'
import { computed, nextTick, shallowRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import type {
	EnvironmentSearchOverride,
	FilterType,
	FilterValue,
	ProjectType,
	SortType,
} from '#ui/utils/search'
import { LOADER_FILTER_TYPES, useSearch } from '#ui/utils/search'
import { useServerSearch } from '#ui/utils/server-search'

import type {
	BrowseDisplayMode,
	BrowseHit,
	BrowseSearchQueryOptions,
	BrowseSearchResults,
	BrowseTags,
} from '../types'
import { useAdvancedPrefsSync } from './use-advanced-prefs-sync'
import {
	type BrowseSearchPrefetchOptions,
	closestMaxResults,
	useBrowseSearchPrefetch,
} from './use-browse-search-prefetch'

const DEFAULT_MAX_RESULTS_OPTIONS = [5, 10, 15, 20, 50, 100]
const SEARCH_DEBOUNCE_MS = 200

export interface BrowseViewStateOptions {
	projectType: Readonly<Ref<string>>
	tags: Readonly<Ref<BrowseTags>>
	searchQuery: (params: string) => BrowseSearchQueryOptions
	persistentQueryParams: string[]
	extraQueryParams?: () => Record<string, string | undefined>
	providedFilters?: Readonly<Ref<FilterValue[]>>
	environmentOverride?: Readonly<Ref<EnvironmentSearchOverride | undefined>>
	active?: Readonly<Ref<boolean>>
	maxResultsOptions?: Readonly<Ref<number[]>>
	displayMode?: Readonly<Ref<BrowseDisplayMode>>
	initialServerFilters?: FilterValue[]
	prefetch?: BrowseSearchPrefetchOptions
}

export interface BrowseViewState {
	query: Ref<string>
	isServerType: ComputedRef<boolean>
	sortTypes: ComputedRef<readonly SortType[]>
	sort: WritableComputedRef<SortType>
	filterTypes: ComputedRef<FilterType[]>
	filters: WritableComputedRef<FilterValue[]>
	toggledGroups: WritableComputedRef<string[]>
	overriddenProvidedFilterTypes: Ref<string[]>
	maxResults: Ref<number>
	maxResultsOptions: ComputedRef<number[]>
	page: Ref<number>
	pageCount: ComputedRef<number>
	layout: ComputedRef<'list' | 'grid'>
	deprioritizedTags: ComputedRef<string[]>
	excludeLoaders: ComputedRef<boolean>
	hits: ComputedRef<BrowseHit[]>
	totalHits: ComputedRef<number>
	loading: ComputedRef<boolean>
	refreshing: ComputedRef<boolean>
	linkOverridesAdvancedPrefs: Ref<boolean>
	applySavedAdvancedPrefs: () => void
	setPage: (page: number) => Promise<void>
	clearSearch: () => void
	onFilterChange: () => void
	ready: () => Promise<void>
}

type TypedSearchResults = BrowseSearchResults & { projectType: string }

function isLoaderFilter(type: string) {
	return LOADER_FILTER_TYPES.includes(type as (typeof LOADER_FILTER_TYPES)[number])
}

export function useBrowseViewState(options: BrowseViewStateOptions): BrowseViewState {
	const route = useRoute()
	const router = useRouter()

	const active = computed(() => options.active?.value ?? true)
	const projectType = computed(() => options.projectType.value)
	const isServerType = computed(() => projectType.value === 'server')
	const providedFilters = computed(() => options.providedFilters?.value ?? [])

	const projectSearch = useSearch(
		computed(() => [projectType.value] as ProjectType[]),
		computed(() => options.tags.value),
		providedFilters,
		computed(() => options.environmentOverride?.value),
	)
	const { query, maxResults, currentPage: page, overriddenProvidedFilterTypes } = projectSearch

	const serverSearch = useServerSearch({
		tags: computed(() => options.tags.value),
		query,
		maxResults,
		currentPage: page,
		providedFilters,
	})

	for (const filter of options.initialServerFilters ?? []) {
		const exists = serverSearch.serverCurrentFilters.value.some(
			(current) => current.type === filter.type && current.option === filter.option,
		)
		if (!exists) serverSearch.serverCurrentFilters.value.push(filter)
	}

	const sortTypes = computed<readonly SortType[]>(() =>
		isServerType.value
			? (serverSearch.serverSortTypes as readonly SortType[])
			: projectSearch.sortTypes,
	)
	const sort = computed<SortType>({
		get: () =>
			isServerType.value
				? serverSearch.serverCurrentSortType.value
				: projectSearch.currentSortType.value,
		set: (value) => {
			if (isServerType.value) serverSearch.serverCurrentSortType.value = value
			else projectSearch.currentSortType.value = value
		},
	})
	const filterTypes = computed(() =>
		isServerType.value ? serverSearch.serverFilterTypes.value : projectSearch.filters.value,
	)
	const filters = computed<FilterValue[]>({
		get: () =>
			isServerType.value
				? serverSearch.serverCurrentFilters.value
				: projectSearch.currentFilters.value,
		set: (value) => {
			if (isServerType.value) serverSearch.serverCurrentFilters.value = value
			else projectSearch.currentFilters.value = value
		},
	})
	const toggledGroups = computed<string[]>({
		get: () =>
			isServerType.value
				? serverSearch.serverToggledGroups.value
				: projectSearch.toggledGroups.value,
		set: (value) => {
			if (isServerType.value) serverSearch.serverToggledGroups.value = value
			else projectSearch.toggledGroups.value = value
		},
	})
	const requestParams = computed(() =>
		isServerType.value ? serverSearch.serverRequestParams.value : projectSearch.requestParams.value,
	)

	const maxResultsOptions = computed(
		() => options.maxResultsOptions?.value ?? DEFAULT_MAX_RESULTS_OPTIONS,
	)
	function clampMaxResults() {
		maxResults.value = closestMaxResults(maxResults.value, maxResultsOptions.value)
	}
	watch(maxResultsOptions, clampMaxResults, { immediate: true })

	const layout = computed<'list' | 'grid'>(() => {
		const mode = options.displayMode?.value ?? 'list'
		return mode === 'grid' || mode === 'gallery' ? 'grid' : 'list'
	})

	const excludeLoaders = computed(
		() =>
			projectSearch.currentFilters.value.some((filter) => isLoaderFilter(filter.type)) ||
			['resourcepack', 'datapack'].includes(projectType.value),
	)
	const deprioritizedTags = computed(() => [
		...projectSearch.currentFilters.value
			.filter((filter) => filter.type.startsWith('category_') || isLoaderFilter(filter.type))
			.map((filter) => filter.option),
		...(options.tags.value?.loaders ?? [])
			.filter((loader) => !loader.supported_project_types.includes(projectType.value))
			.map((loader) => loader.name),
	])

	const request = computed(() => ({ projectType: projectType.value, params: requestParams.value }))
	const debouncedRequest = shallowRef(request.value)
	function commitRequest() {
		debouncedRequest.value = request.value
	}
	const commitRequestDebounced = useDebounceFn(commitRequest, SEARCH_DEBOUNCE_MS)
	watch(request, () => void commitRequestDebounced())

	function resultsQueryOptions(type: string, params: string) {
		const search = options.searchQuery(params)
		return {
			queryKey: [...search.queryKey, type],
			queryFn: async (): Promise<TypedSearchResults> => ({
				...(await search.queryFn()),
				projectType: type,
			}),
			staleTime: search.staleTime,
			networkMode: search.networkMode,
		}
	}

	const resultsQuery = useQuery(
		computed(() => ({
			...resultsQueryOptions(debouncedRequest.value.projectType, debouncedRequest.value.params),
			enabled: active.value,
			placeholderData: keepPreviousData,
		})),
	)

	const currentResults = computed(() => {
		const data = resultsQuery.data.value
		return data?.projectType === projectType.value ? data : undefined
	})
	const hits = computed(() => currentResults.value?.hits ?? [])
	const totalHits = computed(() => currentResults.value?.total_hits ?? 0)
	const pageCount = computed(() =>
		totalHits.value === 0 ? 1 : Math.ceil(totalHits.value / maxResults.value),
	)
	const refreshing = computed(
		() => request.value !== debouncedRequest.value || resultsQuery.isFetching.value,
	)
	const loading = computed(
		() => !currentResults.value && (refreshing.value || resultsQuery.isPending.value),
	)

	const {
		linkOverridesAdvancedPrefs,
		applySavedAdvancedPrefs,
		withSavedPrefs,
		init: initAdvancedPrefs,
	} = useAdvancedPrefsSync({
		filters,
		filterTypes,
		hasLinkQuery: () =>
			Object.keys(route.query).some((key) => !options.persistentQueryParams.includes(key)),
	})

	watch(
		[query, maxResults, projectType, sort, filters, overriddenProvidedFilterTypes, providedFilters],
		() => {
			page.value = 1
		},
		{ deep: true },
	)

	watch(
		projectType,
		() => {
			sort.value =
				sortTypes.value.find((sortType) => sortType.name === 'relevance') ?? sortTypes.value[0]
			query.value = ''
			page.value = 1
			clampMaxResults()
			initAdvancedPrefs()
			commitRequest()
		},
		{ flush: 'sync' },
	)

	if (options.prefetch) {
		const prefetch = options.prefetch
		const queryClient = useQueryClient()
		const { initialRequestParams } = useBrowseSearchPrefetch(prefetch, {
			tags: options.tags,
			maxResults,
			projectFilters: projectSearch.currentFilters,
			serverFilters: serverSearch.serverCurrentFilters,
			overriddenProvidedFilterTypes,
			withSavedAdvancedPrefs: withSavedPrefs,
		})

		let prefetchedRequests = ''
		watch(
			[currentResults, prefetch.projectTypes],
			([results, types]) => {
				if (typeof window === 'undefined' || !results || !active.value) return
				const requests = types
					.filter((type) => type !== projectType.value)
					.map((type) => ({ type, params: initialRequestParams(type) }))
				const signature = JSON.stringify(requests)
				if (signature === prefetchedRequests) return
				prefetchedRequests = signature
				for (const { type, params } of requests) {
					void queryClient.prefetchQuery(resultsQueryOptions(type, params))
				}
			},
			{ immediate: true },
		)
	}

	function syncUrl() {
		const params: Record<string, string | (string | null)[] | null | undefined> = {}
		for (const [key, value] of Object.entries(route.query)) {
			if (options.persistentQueryParams.includes(key)) params[key] = value
		}
		Object.assign(params, options.extraQueryParams?.() ?? {})
		Object.assign(
			params,
			isServerType.value ? serverSearch.createServerPageParams() : projectSearch.createPageParams(),
		)
		const target = { path: route.path, query: params }
		if (router.resolve(target).fullPath === route.fullPath) return
		void router.replace(target)
	}

	watch(
		[currentResults, resultsQuery.isPlaceholderData],
		([results, placeholder]) => {
			if (results && !placeholder && active.value) syncUrl()
		},
		{ immediate: true },
	)

	async function setPage(next: number) {
		page.value = next
		await nextTick()
		window.scrollTo({ top: 0, behavior: 'smooth' })
	}

	function clearSearch() {
		query.value = ''
		page.value = 1
	}

	function onFilterChange() {
		void nextTick(() => window.scrollTo({ top: 0, behavior: 'smooth' }))
	}

	async function ready() {
		try {
			await resultsQuery.suspense()
		} catch (error) {
			console.error('Browse search error:', error)
		}
	}

	return {
		query,
		isServerType,
		sortTypes,
		sort,
		filterTypes,
		filters,
		toggledGroups,
		overriddenProvidedFilterTypes,
		maxResults,
		maxResultsOptions,
		page,
		pageCount,
		layout,
		deprioritizedTags,
		excludeLoaders,
		hits,
		totalHits,
		loading,
		refreshing,
		linkOverridesAdvancedPrefs,
		applySavedAdvancedPrefs,
		setPage,
		clearSearch,
		onFilterChange,
		ready,
	}
}
