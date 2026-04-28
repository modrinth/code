import type { Labrinth } from '@modrinth/api-client'
import { createContext, injectModrinthClient, type ProjectPageContext } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import type { ComputedRef, Ref } from 'vue'

import type { OrganizationContext } from '../organization-context'
import {
	type AnalyticsBreakdownPreset,
	type AnalyticsGroupByPreset,
	type AnalyticsSelectedFilters,
	type AnalyticsTimeframePreset,
	areSelectedFiltersEqual,
	areStringArraysEqual,
	buildAnalyticsQueryBuilderRouteQuery,
	hasAnalyticsQueryBuilderRouteChange,
	readAnalyticsQueryBuilderState,
} from './query-builder-url'

export type {
	AnalyticsBreakdownPreset,
	AnalyticsGroupByPreset,
	AnalyticsQueryFilterCategory,
	AnalyticsSelectedFilters,
	AnalyticsTimeframePreset,
} from './query-builder-url'

export type AnalyticsDashboardStat = 'views' | 'downloads' | 'revenue' | 'playtime'

const MINECRAFT_JAVA_SERVER_PROJECT_TYPE = 'minecraft_java_server'

type ProjectTypeMetadata = {
	project_type?: string | null
	project_types?: readonly string[] | null
}

const ANALYTICS_DASHBOARD_STAT_ORDER: AnalyticsDashboardStat[] = [
	'views',
	'downloads',
	'revenue',
	'playtime',
]

const ANALYTICS_RELEVANT_STATS_BY_BREAKDOWN: Record<
	AnalyticsBreakdownPreset,
	readonly AnalyticsDashboardStat[]
> = {
	none: ANALYTICS_DASHBOARD_STAT_ORDER,
	country: ['views', 'downloads'],
	monetization: ['views'],
	download_source: ['downloads'],
	download_type: ['downloads', 'playtime'],
	loader: ['playtime'],
	game_version: ['playtime'],
}

export interface AnalyticsDashboardProject {
	id: string
	name: string
}

export interface AnalyticsDashboardTotals {
	views: number
	downloads: number
	revenue: number
	playtime: number
}

export interface AnalyticsDashboardPercentChanges {
	views: number
	downloads: number
	revenue: number
	playtime: number
}

export interface AnalyticsDashboardContextValue {
	projects: ComputedRef<AnalyticsDashboardProject[]>
	selectedProjectIds: Ref<string[]>
	selectedTimeframe: Ref<AnalyticsTimeframePreset>
	selectedGroupBy: Ref<AnalyticsGroupByPreset>
	selectedBreakdown: Ref<AnalyticsBreakdownPreset>
	selectedFilters: Ref<AnalyticsSelectedFilters>
	fetchRequest: Ref<Labrinth.Analytics.v3.FetchRequest | null>
	timeSlices: Ref<Labrinth.Analytics.v3.TimeSlice[]>
	previousTimeSlices: Ref<Labrinth.Analytics.v3.TimeSlice[]>
	isLoading: ComputedRef<boolean>
	isRefetching: ComputedRef<boolean>
	activeStat: Ref<AnalyticsDashboardStat>
	currentTotals: ComputedRef<AnalyticsDashboardTotals>
	previousTotals: ComputedRef<AnalyticsDashboardTotals>
	percentChanges: ComputedRef<AnalyticsDashboardPercentChanges>
	getRelevantAnalyticsDashboardStats: (
		breakdown: AnalyticsBreakdownPreset,
	) => readonly AnalyticsDashboardStat[]
	isAnalyticsDashboardStatRelevant: (
		stat: AnalyticsDashboardStat,
		breakdown: AnalyticsBreakdownPreset,
	) => boolean
	setFetchRequest: (fetchRequest: Labrinth.Analytics.v3.FetchRequest) => void
	setActiveStat: (stat: AnalyticsDashboardStat) => void
}

export type CreateAnalyticsDashboardContextOptions = {
	auth: Ref<{ user?: { id?: string } | null }>
	projectPageContext?: ProjectPageContext | null
	organizationContext?: OrganizationContext | null
}

export const [injectAnalyticsDashboardContext, provideAnalyticsDashboardContext] =
	createContext<AnalyticsDashboardContextValue>('AnalyticsDashboard')

function buildPreviousFetchRequest(
	fetchRequest: Labrinth.Analytics.v3.FetchRequest | null,
): Labrinth.Analytics.v3.FetchRequest | null {
	if (!fetchRequest) {
		return null
	}

	const startTimestamp = new Date(fetchRequest.time_range.start).getTime()
	const endTimestamp = new Date(fetchRequest.time_range.end).getTime()
	const duration = endTimestamp - startTimestamp

	if (!Number.isFinite(duration) || duration <= 0) {
		return null
	}

	const previousEnd = new Date(startTimestamp)
	const previousStart = new Date(startTimestamp - duration)

	return {
		time_range: {
			start: previousStart.toISOString(),
			end: previousEnd.toISOString(),
			resolution: fetchRequest.time_range.resolution,
		},
		project_ids: fetchRequest.project_ids,
		return_metrics: fetchRequest.return_metrics,
	}
}

function getPercentChange(currentValue: number, previousValue: number): number {
	if (previousValue === 0) {
		if (currentValue === 0) {
			return 0
		}
		return 100
	}

	return ((currentValue - previousValue) / previousValue) * 100
}

function computeTotals(
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
	selectedProjectIds: Set<string>,
	availableProjectIds: Set<string>,
): AnalyticsDashboardTotals {
	const totals: AnalyticsDashboardTotals = {
		views: 0,
		downloads: 0,
		revenue: 0,
		playtime: 0,
	}

	if (availableProjectIds.size === 0) {
		return totals
	}

	const effectiveProjectIds = selectedProjectIds.size > 0 ? selectedProjectIds : availableProjectIds

	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!('source_project' in dataPoint)) {
				continue
			}

			if (!effectiveProjectIds.has(dataPoint.source_project)) {
				continue
			}

			switch (dataPoint.metric_kind) {
				case 'views':
					totals.views += dataPoint.views
					break
				case 'downloads':
					totals.downloads += dataPoint.downloads
					break
				case 'playtime':
					totals.playtime += dataPoint.seconds
					break
				case 'revenue': {
					const value = Number.parseFloat(dataPoint.revenue)
					totals.revenue += Number.isFinite(value) ? value : 0
					break
				}
			}
		}
	}

	return totals
}

function isServerProject(project: ProjectTypeMetadata): boolean {
	if (project.project_type === MINECRAFT_JAVA_SERVER_PROJECT_TYPE) {
		return true
	}

	return project.project_types?.includes(MINECRAFT_JAVA_SERVER_PROJECT_TYPE) ?? false
}

export function createAnalyticsDashboardContext(
	options: CreateAnalyticsDashboardContextOptions,
): AnalyticsDashboardContextValue {
	const client = injectModrinthClient()
	const route = useRoute()
	const router = useRouter()
	const initialQueryState = readAnalyticsQueryBuilderState(route.query, [])

	const activeStat = ref<AnalyticsDashboardStat>('views')
	const selectedProjectIds = ref<string[]>(initialQueryState.selectedProjectIds)
	const selectedTimeframe = ref<AnalyticsTimeframePreset>(initialQueryState.selectedTimeframe)
	const selectedGroupBy = ref<AnalyticsGroupByPreset>(initialQueryState.selectedGroupBy)
	const selectedBreakdown = ref<AnalyticsBreakdownPreset>(initialQueryState.selectedBreakdown)
	const selectedFilters = ref<AnalyticsSelectedFilters>(initialQueryState.selectedFilters)
	const fetchRequest = ref<Labrinth.Analytics.v3.FetchRequest | null>(null)

	const hasProjectContext = computed(() => Boolean(options.projectPageContext))
	const hasOrganizationContext = computed(
		() => !hasProjectContext.value && Boolean(options.organizationContext),
	)

	const { data: userProjects } = useQuery({
		queryKey: computed(() => ['analytics', 'dashboard', options.auth.value?.user?.id, 'projects']),
		queryFn: () => client.labrinth.users_v2.getProjects(options.auth.value.user?.id ?? ''),
		enabled: computed(
			() =>
				Boolean(options.auth.value.user?.id) &&
				!hasProjectContext.value &&
				!hasOrganizationContext.value,
		),
		placeholderData: [],
	})

	const projects = computed<AnalyticsDashboardProject[]>(() => {
		if (hasProjectContext.value && options.projectPageContext) {
			const project = options.projectPageContext.projectV2.value
			return project && !isServerProject(project) ? [{ id: project.id, name: project.title }] : []
		}

		if (hasOrganizationContext.value && options.organizationContext?.projects.value) {
			return options.organizationContext.projects.value
				.filter((project) => !isServerProject(project))
				.map((project) => ({
					id: project.id,
					name: project.name,
				}))
		}

		return (userProjects.value ?? [])
			.filter((project) => !isServerProject(project))
			.map((project) => ({
				id: project.id,
				name: project.title,
			}))
	})

	const availableProjectIds = computed(() => projects.value.map((project) => project.id))

	function getRelevantAnalyticsDashboardStats(
		breakdown: AnalyticsBreakdownPreset,
	): readonly AnalyticsDashboardStat[] {
		return ANALYTICS_RELEVANT_STATS_BY_BREAKDOWN[breakdown] ?? ANALYTICS_DASHBOARD_STAT_ORDER
	}

	function isAnalyticsDashboardStatRelevant(
		stat: AnalyticsDashboardStat,
		breakdown: AnalyticsBreakdownPreset,
	): boolean {
		return getRelevantAnalyticsDashboardStats(breakdown).includes(stat)
	}

	watch(
		[selectedBreakdown, activeStat],
		([nextBreakdown, nextActiveStat]) => {
			if (isAnalyticsDashboardStatRelevant(nextActiveStat, nextBreakdown)) {
				return
			}

			const fallbackStat = getRelevantAnalyticsDashboardStats(nextBreakdown)[0]
			if (fallbackStat && fallbackStat !== nextActiveStat) {
				activeStat.value = fallbackStat
			}
		},
		{ immediate: true },
	)

	watch(
		projects,
		(nextProjects) => {
			if (nextProjects.length === 0) {
				return
			}

			const availableProjectIds = new Set(nextProjects.map((project) => project.id))
			const retainedSelection = selectedProjectIds.value.filter((id) => availableProjectIds.has(id))

			selectedProjectIds.value =
				retainedSelection.length > 0 ? retainedSelection : nextProjects.map((project) => project.id)
		},
		{ immediate: true },
	)

	watch(
		() => route.query,
		(nextQuery) => {
			const nextQueryState = readAnalyticsQueryBuilderState(nextQuery, availableProjectIds.value)

			if (!areStringArraysEqual(selectedProjectIds.value, nextQueryState.selectedProjectIds)) {
				selectedProjectIds.value = nextQueryState.selectedProjectIds
			}
			if (selectedTimeframe.value !== nextQueryState.selectedTimeframe) {
				selectedTimeframe.value = nextQueryState.selectedTimeframe
			}
			if (selectedGroupBy.value !== nextQueryState.selectedGroupBy) {
				selectedGroupBy.value = nextQueryState.selectedGroupBy
			}
			if (selectedBreakdown.value !== nextQueryState.selectedBreakdown) {
				selectedBreakdown.value = nextQueryState.selectedBreakdown
			}
			if (!areSelectedFiltersEqual(selectedFilters.value, nextQueryState.selectedFilters)) {
				selectedFilters.value = nextQueryState.selectedFilters
			}
		},
	)

	watch(
		[
			selectedProjectIds,
			selectedTimeframe,
			selectedGroupBy,
			selectedBreakdown,
			selectedFilters,
			availableProjectIds,
		],
		() => {
			if (import.meta.server) {
				return
			}

			const nextRouteQuery = buildAnalyticsQueryBuilderRouteQuery(
				route.query,
				{
					selectedProjectIds: selectedProjectIds.value,
					selectedTimeframe: selectedTimeframe.value,
					selectedGroupBy: selectedGroupBy.value,
					selectedBreakdown: selectedBreakdown.value,
					selectedFilters: selectedFilters.value,
				},
				availableProjectIds.value,
			)

			const hasAnalyticsQueryChange = hasAnalyticsQueryBuilderRouteChange(
				route.query,
				nextRouteQuery,
			)

			if (!hasAnalyticsQueryChange) return

			router.replace({
				path: route.path,
				query: nextRouteQuery,
			})
		},
		{ deep: true, immediate: true },
	)

	const {
		data: currentTimeSliceData,
		isPending: currentTimeSlicePending,
		isFetching: currentFetching,
	} = useQuery({
		queryKey: computed(() => ['analytics', 'dashboard', 'current', fetchRequest.value]),
		queryFn: () =>
			client.labrinth.analytics_v3.fetch(fetchRequest.value as Labrinth.Analytics.v3.FetchRequest),
		enabled: computed(() => fetchRequest.value !== null),
		placeholderData: [],
	})

	const previousFetchRequest = computed(() => buildPreviousFetchRequest(fetchRequest.value))

	const {
		data: previousTimeSliceData,
		isPending: previousTimeSlicePending,
		isFetching: previousFetching,
	} = useQuery({
		queryKey: computed(() => ['analytics', 'dashboard', 'previous', previousFetchRequest.value]),
		queryFn: () =>
			client.labrinth.analytics_v3.fetch(
				previousFetchRequest.value as Labrinth.Analytics.v3.FetchRequest,
			),
		enabled: computed(() => previousFetchRequest.value !== null),
		placeholderData: [],
	})

	const timeSlices = ref<Labrinth.Analytics.v3.TimeSlice[]>([])
	const previousTimeSlices = ref<Labrinth.Analytics.v3.TimeSlice[]>([])

	watch(
		currentTimeSliceData,
		(nextTimeSlices) => {
			timeSlices.value = nextTimeSlices ?? []
		},
		{ immediate: true },
	)

	watch(
		previousTimeSliceData,
		(nextTimeSlices) => {
			previousTimeSlices.value = nextTimeSlices ?? []
		},
		{ immediate: true },
	)

	const selectedProjectIdSet = computed(() => new Set(selectedProjectIds.value))
	const availableProjectIdSet = computed(() => new Set(availableProjectIds.value))

	const currentTotals = computed<AnalyticsDashboardTotals>(() =>
		computeTotals(timeSlices.value, selectedProjectIdSet.value, availableProjectIdSet.value),
	)
	const previousTotals = computed<AnalyticsDashboardTotals>(() =>
		computeTotals(
			previousTimeSlices.value,
			selectedProjectIdSet.value,
			availableProjectIdSet.value,
		),
	)

	const percentChanges = computed<AnalyticsDashboardPercentChanges>(() => ({
		views: getPercentChange(currentTotals.value.views, previousTotals.value.views),
		downloads: getPercentChange(currentTotals.value.downloads, previousTotals.value.downloads),
		revenue: getPercentChange(currentTotals.value.revenue, previousTotals.value.revenue),
		playtime: getPercentChange(currentTotals.value.playtime, previousTotals.value.playtime),
	}))

	const isLoading = computed(() => currentTimeSlicePending.value || previousTimeSlicePending.value)
	const isRefetching = computed(() => currentFetching.value || previousFetching.value)

	function setFetchRequest(nextFetchRequest: Labrinth.Analytics.v3.FetchRequest) {
		fetchRequest.value = nextFetchRequest
	}

	function setActiveStat(nextStat: AnalyticsDashboardStat) {
		if (!isAnalyticsDashboardStatRelevant(nextStat, selectedBreakdown.value)) {
			return
		}

		activeStat.value = nextStat
	}

	return {
		projects,
		selectedProjectIds,
		selectedTimeframe,
		selectedGroupBy,
		selectedBreakdown,
		selectedFilters,
		fetchRequest,
		timeSlices,
		previousTimeSlices,
		isLoading,
		isRefetching,
		activeStat,
		currentTotals,
		previousTotals,
		percentChanges,
		getRelevantAnalyticsDashboardStats,
		isAnalyticsDashboardStatRelevant,
		setFetchRequest,
		setActiveStat,
	}
}
