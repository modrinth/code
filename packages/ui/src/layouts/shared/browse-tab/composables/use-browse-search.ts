import type { Labrinth } from '@modrinth/api-client'
import type { ComputedRef, Ref, ShallowRef } from 'vue'
import { computed, nextTick, ref, shallowRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { useDebugLogger } from '#ui/composables/debug-logger'
import type { FilterType, FilterValue, ProjectType, SortType } from '#ui/utils/search'
import { useSearch } from '#ui/utils/search'
import { useServerSearch } from '#ui/utils/server-search'

import type { BrowseSearchResponse } from '../types'

export interface UseBrowseSearchOptions {
	projectType: Ref<string>
	tags: Ref<{
		gameVersions: Labrinth.Tags.v2.GameVersion[]
		loaders: Labrinth.Tags.v2.Loader[]
		categories: Labrinth.Tags.v2.Category[]
	}>
	providedFilters?: ComputedRef<FilterValue[]>
	search: (params: string) => Promise<BrowseSearchResponse>
	persistentQueryParams: string[]
	getExtraQueryParams?: () => Record<string, string | undefined>
	maxResultsOptions?: ComputedRef<number[]>
	displayMode?: Ref<'list' | 'grid' | 'gallery'> | ComputedRef<'list' | 'grid' | 'gallery'>
}

export interface BrowseSearchState {
	query: Ref<string>

	filters: ComputedRef<FilterType[]>
	currentFilters: Ref<FilterValue[]>
	toggledGroups: Ref<string[]>
	overriddenProvidedFilterTypes: Ref<string[]>

	serverFilterTypes: ComputedRef<FilterType[]>
	serverCurrentFilters: Ref<FilterValue[]>
	serverToggledGroups: Ref<string[]>

	effectiveSortTypes: ComputedRef<readonly SortType[]>
	effectiveCurrentSortType: Ref<SortType>

	loading: Ref<boolean>
	projectHits: ShallowRef<BrowseSearchResponse['projectHits']>
	serverHits: ShallowRef<BrowseSearchResponse['serverHits']>
	totalHits: Ref<number>
	pageCount: ComputedRef<number>

	maxResults: Ref<number>
	currentPage: Ref<number>

	isServerType: ComputedRef<boolean>
	effectiveLayout: ComputedRef<'list' | 'grid'>
	deprioritizedTags: ComputedRef<string[]>
	excludeLoaders: ComputedRef<boolean>

	refreshSearch: () => Promise<void>
	setPage: (page: number) => Promise<void>
	clearSearch: () => void
	onFilterChange: () => void
}

const LOADER_FILTER_TYPES = [
	'mod_loader',
	'plugin_loader',
	'modpack_loader',
	'shader_loader',
	'plugin_platform',
] as const

export function useBrowseSearch(options: UseBrowseSearchOptions): BrowseSearchState {
	const debug = useDebugLogger('BrowseSearch')
	const route = useRoute()
	const router = useRouter()

	debug('init, projectType:', options.projectType.value)

	const projectTypes = computed(() => [options.projectType.value] as ProjectType[])
	const isServerType = computed(() => options.projectType.value === 'server')

	const {
		query,
		currentSortType,
		currentFilters,
		toggledGroups,
		maxResults,
		currentPage,
		overriddenProvidedFilterTypes,
		filters,
		sortTypes,
		requestParams,
		createPageParams,
	} = useSearch(projectTypes, options.tags, options.providedFilters ?? computed(() => []))

	const {
		serverCurrentSortType,
		serverCurrentFilters,
		serverToggledGroups,
		serverSortTypes,
		serverFilterTypes,
		serverRequestParams,
		createServerPageParams,
	} = useServerSearch({ tags: options.tags, query, maxResults, currentPage })

	const effectiveRequestParams = computed(() =>
		isServerType.value ? serverRequestParams.value : requestParams.value,
	)
	const effectiveSortTypes = computed(() =>
		isServerType.value ? (serverSortTypes as readonly SortType[]) : sortTypes,
	)
	const effectiveCurrentSortType = computed({
		get: () => (isServerType.value ? serverCurrentSortType.value : currentSortType.value),
		set: (v: SortType) => {
			if (isServerType.value) serverCurrentSortType.value = v
			else currentSortType.value = v
		},
	})

	const effectiveMaxResultsOptions = computed(
		() => options.maxResultsOptions?.value ?? [5, 10, 15, 20, 50, 100],
	)

	watch(effectiveMaxResultsOptions, (opts) => {
		if (!opts.includes(maxResults.value)) {
			maxResults.value = opts.reduce((prev, curr) =>
				Math.abs(curr - maxResults.value) <= Math.abs(prev - maxResults.value) ? curr : prev,
			)
		}
	})

	const effectiveDisplayMode = computed(() => options.displayMode?.value ?? 'list')
	const effectiveLayout = computed<'list' | 'grid'>(() =>
		effectiveDisplayMode.value === 'grid' || effectiveDisplayMode.value === 'gallery'
			? 'grid'
			: 'list',
	)

	const selectedFilterTags = computed(() =>
		currentFilters.value
			.filter(
				(f) =>
					f.type.startsWith('category_') ||
					LOADER_FILTER_TYPES.includes(f.type as (typeof LOADER_FILTER_TYPES)[number]),
			)
			.map((f) => f.option),
	)
	const excludeLoaders = computed(
		() =>
			currentFilters.value.some((f) =>
				LOADER_FILTER_TYPES.includes(f.type as (typeof LOADER_FILTER_TYPES)[number]),
			) || ['resourcepack', 'datapack'].includes(options.projectType.value),
	)
	const loadersNotForThisType = computed(
		() =>
			options.tags.value?.loaders
				?.filter((loader) => !loader.supported_project_types.includes(options.projectType.value))
				?.map((loader) => loader.name) ?? [],
	)
	const deprioritizedTags = computed(() => [
		...selectedFilterTags.value,
		...loadersNotForThisType.value,
	])

	const loading = ref(true)
	const projectHits = shallowRef<BrowseSearchResponse['projectHits']>([])
	const serverHits = shallowRef<BrowseSearchResponse['serverHits']>([])
	const totalHits = ref(0)

	const pageCount = computed(() => {
		if (totalHits.value === 0) return 1
		return Math.ceil(totalHits.value / maxResults.value)
	})

	let searchVersion = 0
	let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null

	watch(effectiveRequestParams, (newVal, oldVal) => {
		debug('effectiveRequestParams changed', {
			from: oldVal?.substring(0, 80),
			to: newVal?.substring(0, 80),
		})
		if (searchDebounceTimer) clearTimeout(searchDebounceTimer)
		searchDebounceTimer = setTimeout(() => {
			refreshSearch()
		}, 200)
	})

	async function refreshSearch() {
		const version = ++searchVersion
		debug('refreshSearch start', {
			version,
			projectType: options.projectType.value,
			params: effectiveRequestParams.value.substring(0, 100),
		})

		const currentHitsEmpty = isServerType.value
			? serverHits.value.length === 0
			: projectHits.value.length === 0
		if (currentHitsEmpty) {
			loading.value = true
		}

		try {
			const response = await options.search(effectiveRequestParams.value)

			if (version !== searchVersion) {
				debug('refreshSearch stale, discarding', { version, current: searchVersion })
				return
			}

			if (isServerType.value) {
				serverHits.value = response.serverHits
			} else {
				projectHits.value = response.projectHits
			}
			totalHits.value = response.total_hits
			debug('refreshSearch complete', {
				version,
				hits: response.total_hits,
				projectHits: response.projectHits.length,
				serverHits: response.serverHits.length,
			})

			updateUrlParams()
			loading.value = false
		} catch (err) {
			debug('refreshSearch error', err)
			console.error('Browse search error:', err)
			if (version === searchVersion) {
				loading.value = false
			}
		}
	}

	function updateUrlParams() {
		debug('updateUrlParams', { path: route.path })
		const persistentParams: Record<string, string | (string | null)[] | null | undefined> = {}

		for (const [key, value] of Object.entries(route.query)) {
			if (options.persistentQueryParams.includes(key)) {
				persistentParams[key] = value
			}
		}

		const extraParams = options.getExtraQueryParams?.() ?? {}
		for (const [key, value] of Object.entries(extraParams)) {
			if (value !== undefined) {
				persistentParams[key] = value
			}
		}

		const params = {
			...persistentParams,
			...(isServerType.value ? createServerPageParams() : createPageParams()),
		}

		router.replace({ path: route.path, query: params })
	}

	async function setPage(newPageNumber: number) {
		currentPage.value = newPageNumber
		await nextTick()
		window.scrollTo({ top: 0, behavior: 'smooth' })
	}

	function clearSearch() {
		query.value = ''
		currentPage.value = 1
	}

	function onFilterChange() {
		nextTick(() => window.scrollTo({ top: 0, behavior: 'smooth' }))
	}

	watch(
		() => options.projectType.value,
		(newType, oldType) => {
			debug('projectType changed', { from: oldType, to: newType })
			currentSortType.value = { display: 'Relevance', name: 'relevance' }
			query.value = ''
		},
	)

	return {
		query,
		filters,
		currentFilters,
		toggledGroups,
		overriddenProvidedFilterTypes,
		serverFilterTypes,
		serverCurrentFilters,
		serverToggledGroups,
		effectiveSortTypes,
		effectiveCurrentSortType,
		loading,
		projectHits,
		serverHits,
		totalHits,
		pageCount,
		maxResults,
		currentPage,
		isServerType,
		effectiveLayout,
		deprioritizedTags,
		excludeLoaders,
		refreshSearch,
		setPage,
		clearSearch,
		onFilterChange,
	}
}
