import type { Labrinth } from '@modrinth/api-client'
import type { Component, ComputedRef, MaybeRef, Ref, ShallowRef } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import { createContext } from '#ui/providers/create-context'
import type { FilterType, FilterValue, SortType } from '#ui/utils/search'

import type {
	BrowseInstallContext,
	BrowseSearchResponse,
	CardAction,
	ServerModpackContent,
} from '../types'

export interface BrowseManagerContext {
	tags: Ref<{
		gameVersions: Labrinth.Tags.v2.GameVersion[]
		loaders: Labrinth.Tags.v2.Loader[]
		categories: Labrinth.Tags.v2.Category[]
	}>
	projectType: Ref<string>

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

	getProjectLink: (result: Labrinth.Search.v2.ResultSearchProject) => string | RouteLocationRaw
	getServerProjectLink: (
		result: Labrinth.Search.v3.ResultSearchProject,
	) => string | RouteLocationRaw

	selectableProjectTypes: ComputedRef<{ label: string; href: string; shown?: boolean }[]>
	showProjectTypeTabs: ComputedRef<boolean>

	variant: 'app' | 'web'

	getCardActions?: (
		result: Labrinth.Search.v2.ResultSearchProject | Labrinth.Search.v3.ResultSearchProject,
		projectType: string,
	) => CardAction[]

	installContext?: ComputedRef<BrowseInstallContext | null>
	providedFilters?: ComputedRef<FilterValue[]>
	hideInstalled?: Ref<boolean>
	showHideInstalled?: ComputedRef<boolean>
	hideInstalledLabel?: ComputedRef<string>
	hideSelected?: Ref<boolean>
	showHideSelected?: ComputedRef<boolean>
	hideSelectedLabel?: ComputedRef<string>
	onInstalled?: (projectId: string) => void

	displayMode?: Ref<'list' | 'grid' | 'gallery'> | ComputedRef<'list' | 'grid' | 'gallery'>
	cycleDisplayMode?: () => void
	maxResultsOptions?: ComputedRef<number[]>

	serverPings?: Ref<Record<string, number | undefined>>
	getServerModpackContent?: (
		result: Labrinth.Search.v3.ResultSearchProject,
	) => ServerModpackContent | undefined

	onProjectHover?: (result: Labrinth.Search.v2.ResultSearchProject) => void
	onServerProjectHover?: (result: Labrinth.Search.v3.ResultSearchProject) => void
	onProjectHoverEnd?: () => void
	onContextMenu?: (
		event: MouseEvent,
		result: Labrinth.Search.v2.ResultSearchProject | Labrinth.Search.v3.ResultSearchProject,
	) => void
	offline?: Ref<boolean>

	filtersMenuOpen?: Ref<boolean>

	lockedFilterMessages?: MaybeRef<{
		gameVersion?: string
		modLoader?: string
		environment?: string
		syncButton?: string
		providedBy?: string
		gameVersionShaderMessage?: string
	}>

	loadingComponent?: Component
}

export const [injectBrowseManager, provideBrowseManager] = createContext<BrowseManagerContext>(
	'BrowsePageLayout',
	'browseManagerContext',
)
