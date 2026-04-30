import type { Labrinth } from '@modrinth/api-client'
import { createContext, injectModrinthClient, type ProjectPageContext } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import type { ComputedRef, Ref } from 'vue'

import { fetchSegmentedWith } from '~/utils/fetch-helpers.ts'

import type { OrganizationContext } from '../organization-context'
import {
	type AnalyticsBreakdownPreset,
	type AnalyticsGroupByPreset,
	type AnalyticsLastTimeframeUnit,
	type AnalyticsSelectedFilters,
	type AnalyticsTimeframeMode,
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
	AnalyticsLastTimeframeUnit,
	AnalyticsQueryFilterCategory,
	AnalyticsSelectedFilters,
	AnalyticsTimeframeMode,
	AnalyticsTimeframePreset,
} from './query-builder-url'

export type AnalyticsDashboardStat = 'views' | 'downloads' | 'revenue' | 'playtime'

const MINECRAFT_JAVA_SERVER_PROJECT_TYPE = 'minecraft_java_server'
const ANALYTICS_START_TIMESTAMP = '2022-01-01T00:00:00.000Z'

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
	version_id: ['downloads', 'playtime'],
	loader: ['playtime'],
	game_version: ['playtime'],
}

export interface AnalyticsDashboardProject {
	id: string
	name: string
	downloads: number
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

export interface AnalyticsDashboardFilterOptions {
	countries: string[]
	gameVersions: string[]
	versionIds: string[]
}

export interface AnalyticsDashboardContextValue {
	projects: ComputedRef<AnalyticsDashboardProject[]>
	selectedProjectIds: Ref<string[]>
	selectedTimeframeMode: Ref<AnalyticsTimeframeMode>
	selectedTimeframe: Ref<AnalyticsTimeframePreset>
	selectedLastTimeframeAmount: Ref<number>
	selectedLastTimeframeUnit: Ref<AnalyticsLastTimeframeUnit>
	selectedCustomTimeframeStartDate: Ref<string>
	selectedCustomTimeframeEndDate: Ref<string>
	selectedGroupBy: Ref<AnalyticsGroupByPreset>
	selectedBreakdown: Ref<AnalyticsBreakdownPreset>
	selectedFilters: Ref<AnalyticsSelectedFilters>
	queryRefreshTimestamp: Ref<number>
	fetchRequest: Ref<Labrinth.Analytics.v3.FetchRequest | null>
	filterOptions: ComputedRef<AnalyticsDashboardFilterOptions>
	versionNumbersById: ComputedRef<Map<string, string>>
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
	refreshAnalyticsQuery: () => Promise<void>
	getVersionDisplayName: (versionId: string) => string
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
	filters: AnalyticsSelectedFilters,
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

			if (!doesAnalyticsPointMatchFilters(dataPoint, filters)) {
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

function sortStringValues(values: string[]): string[] {
	return [...values].sort((left, right) => left.localeCompare(right))
}

function getCountryFilterOptions(
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
): string[] {
	const countries = new Set<string>()

	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!('source_project' in dataPoint)) {
				continue
			}

			if (
				(dataPoint.metric_kind === 'views' || dataPoint.metric_kind === 'downloads') &&
				dataPoint.country
			) {
				const country = dataPoint.country.trim().toUpperCase()
				if (country.length > 0) {
					countries.add(country)
				}
			}
		}
	}

	return sortStringValues([...countries])
}

function addVersionIdsFromTimeSlices(
	versionIds: Set<string>,
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
) {
	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!('source_project' in dataPoint)) {
				continue
			}

			if (
				(dataPoint.metric_kind === 'downloads' || dataPoint.metric_kind === 'playtime') &&
				dataPoint.version_id
			) {
				const versionId = dataPoint.version_id.trim()
				if (versionId.length > 0) {
					versionIds.add(versionId)
				}
			}
		}
	}
}

function getVersionFilterOptions(timeSlices: Labrinth.Analytics.v3.TimeSlice[]): string[] {
	const versionIds = new Set<string>()
	addVersionIdsFromTimeSlices(versionIds, timeSlices)
	return sortStringValues([...versionIds])
}

function getGameVersionFilterOptions(timeSlices: Labrinth.Analytics.v3.TimeSlice[]): string[] {
	const gameVersions = new Set<string>()

	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!('source_project' in dataPoint)) {
				continue
			}

			if (dataPoint.metric_kind === 'playtime' && dataPoint.game_version) {
				const gameVersion = dataPoint.game_version.trim()
				if (gameVersion.length > 0) {
					gameVersions.add(gameVersion)
				}
			}
		}
	}

	return sortStringValues([...gameVersions])
}

export function doesAnalyticsPointMatchFilters(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
	filters: AnalyticsSelectedFilters,
): boolean {
	return (
		doesAnalyticsPointMatchFilter(dataPoint, filters.country, getCountryFilterValue) &&
		doesAnalyticsPointMatchFilter(dataPoint, filters.monetization, getMonetizationFilterValue) &&
		doesAnalyticsPointMatchFilter(
			dataPoint,
			filters.download_source,
			getDownloadSourceFilterValue,
		) &&
		doesAnalyticsPointMatchFilter(dataPoint, filters.version_id, getVersionFilterValue) &&
		doesAnalyticsPointMatchFilter(dataPoint, filters.game_version, getGameVersionFilterValue) &&
		doesAnalyticsPointMatchFilter(dataPoint, filters.loader_type, getLoaderFilterValue)
	)
}

function doesAnalyticsPointMatchFilter(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
	filterValues: string[],
	getPointValue: (dataPoint: Labrinth.Analytics.v3.ProjectAnalytics) => string | null,
): boolean {
	if (filterValues.length === 0) {
		return true
	}

	const pointValue = getPointValue(dataPoint)
	if (pointValue === null) {
		return false
	}

	const normalizedPointValue = pointValue.trim().toLowerCase()
	return filterValues.some((value) => value.trim().toLowerCase() === normalizedPointValue)
}

function getCountryFilterValue(dataPoint: Labrinth.Analytics.v3.ProjectAnalytics): string | null {
	if (dataPoint.metric_kind !== 'views' && dataPoint.metric_kind !== 'downloads') {
		return null
	}

	return dataPoint.country ?? null
}

function getMonetizationFilterValue(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
): string | null {
	if (dataPoint.metric_kind !== 'views' || typeof dataPoint.monetized !== 'boolean') {
		return null
	}

	return dataPoint.monetized ? 'monetized' : 'unmonetized'
}

function getDownloadSourceFilterValue(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
): string | null {
	if (dataPoint.metric_kind !== 'views' && dataPoint.metric_kind !== 'downloads') {
		return null
	}

	return dataPoint.domain ?? null
}

function getVersionFilterValue(dataPoint: Labrinth.Analytics.v3.ProjectAnalytics): string | null {
	if (dataPoint.metric_kind !== 'downloads' && dataPoint.metric_kind !== 'playtime') {
		return null
	}

	return dataPoint.version_id ?? null
}

function getGameVersionFilterValue(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
): string | null {
	if (dataPoint.metric_kind !== 'playtime') {
		return null
	}

	return dataPoint.game_version ?? null
}

function getLoaderFilterValue(dataPoint: Labrinth.Analytics.v3.ProjectAnalytics): string | null {
	if (dataPoint.metric_kind !== 'playtime') {
		return null
	}

	return dataPoint.loader ?? null
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
	const selectedTimeframeMode = ref<AnalyticsTimeframeMode>(initialQueryState.selectedTimeframeMode)
	const selectedTimeframe = ref<AnalyticsTimeframePreset>(initialQueryState.selectedTimeframe)
	const selectedLastTimeframeAmount = ref<number>(initialQueryState.selectedLastTimeframeAmount)
	const selectedLastTimeframeUnit = ref<AnalyticsLastTimeframeUnit>(
		initialQueryState.selectedLastTimeframeUnit,
	)
	const selectedCustomTimeframeStartDate = ref<string>(
		initialQueryState.selectedCustomTimeframeStartDate,
	)
	const selectedCustomTimeframeEndDate = ref<string>(
		initialQueryState.selectedCustomTimeframeEndDate,
	)
	const selectedGroupBy = ref<AnalyticsGroupByPreset>(initialQueryState.selectedGroupBy)
	const selectedBreakdown = ref<AnalyticsBreakdownPreset>(initialQueryState.selectedBreakdown)
	const selectedFilters = ref<AnalyticsSelectedFilters>(initialQueryState.selectedFilters)
	const queryRefreshTimestamp = ref(Date.now())
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
			return project && !isServerProject(project)
				? [{ id: project.id, name: project.title, downloads: project.downloads ?? 0 }]
				: []
		}

		if (hasOrganizationContext.value && options.organizationContext?.projects.value) {
			return options.organizationContext.projects.value
				.filter((project) => !isServerProject(project))
				.map((project) => ({
					id: project.id,
					name: project.name,
					downloads: project.downloads ?? 0,
				}))
		}

		return (userProjects.value ?? [])
			.filter((project) => !isServerProject(project))
			.map((project) => ({
				id: project.id,
				name: project.title,
				downloads: project.downloads ?? 0,
			}))
	})

	const availableProjectIds = computed(() => projects.value.map((project) => project.id))
	const sortedSelectedProjectIds = computed(() => sortStringValues(selectedProjectIds.value))

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
			if (selectedTimeframeMode.value !== nextQueryState.selectedTimeframeMode) {
				selectedTimeframeMode.value = nextQueryState.selectedTimeframeMode
			}
			if (selectedTimeframe.value !== nextQueryState.selectedTimeframe) {
				selectedTimeframe.value = nextQueryState.selectedTimeframe
			}
			if (selectedLastTimeframeAmount.value !== nextQueryState.selectedLastTimeframeAmount) {
				selectedLastTimeframeAmount.value = nextQueryState.selectedLastTimeframeAmount
			}
			if (selectedLastTimeframeUnit.value !== nextQueryState.selectedLastTimeframeUnit) {
				selectedLastTimeframeUnit.value = nextQueryState.selectedLastTimeframeUnit
			}
			if (
				selectedCustomTimeframeStartDate.value !== nextQueryState.selectedCustomTimeframeStartDate
			) {
				selectedCustomTimeframeStartDate.value = nextQueryState.selectedCustomTimeframeStartDate
			}
			if (selectedCustomTimeframeEndDate.value !== nextQueryState.selectedCustomTimeframeEndDate) {
				selectedCustomTimeframeEndDate.value = nextQueryState.selectedCustomTimeframeEndDate
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
			selectedTimeframeMode,
			selectedTimeframe,
			selectedLastTimeframeAmount,
			selectedLastTimeframeUnit,
			selectedCustomTimeframeStartDate,
			selectedCustomTimeframeEndDate,
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
					selectedTimeframeMode: selectedTimeframeMode.value,
					selectedTimeframe: selectedTimeframe.value,
					selectedLastTimeframeAmount: selectedLastTimeframeAmount.value,
					selectedLastTimeframeUnit: selectedLastTimeframeUnit.value,
					selectedCustomTimeframeStartDate: selectedCustomTimeframeStartDate.value,
					selectedCustomTimeframeEndDate: selectedCustomTimeframeEndDate.value,
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
		refetch: refetchCurrentTimeSlices,
	} = useQuery({
		queryKey: computed(() => ['analytics', 'dashboard', 'current', fetchRequest.value]),
		queryFn: () =>
			client.labrinth.analytics_v3.fetch(fetchRequest.value as Labrinth.Analytics.v3.FetchRequest),
		enabled: computed(() => fetchRequest.value !== null),
	})

	const countryFilterOptionsRequest = computed<Labrinth.Analytics.v3.FetchRequest | null>(() => {
		if (sortedSelectedProjectIds.value.length === 0) {
			return null
		}

		return {
			time_range: {
				start: ANALYTICS_START_TIMESTAMP,
				end: new Date(queryRefreshTimestamp.value).toISOString(),
				resolution: {
					slices: 1,
				},
			},
			project_ids: sortedSelectedProjectIds.value,
			return_metrics: {
				project_views: {
					bucket_by: ['country'],
				},
				project_downloads: {
					bucket_by: ['country'],
				},
			},
		}
	})

	const { data: countryFilterOptionsData } = useQuery({
		queryKey: computed(() => [
			'analytics',
			'dashboard',
			'filter-options',
			'countries',
			countryFilterOptionsRequest.value,
		]),
		queryFn: () =>
			client.labrinth.analytics_v3.fetch(
				countryFilterOptionsRequest.value as Labrinth.Analytics.v3.FetchRequest,
			),
		enabled: computed(() => countryFilterOptionsRequest.value !== null),
	})

	const versionFilterOptionsRequest = computed<Labrinth.Analytics.v3.FetchRequest | null>(() => {
		if (sortedSelectedProjectIds.value.length === 0) {
			return null
		}

		return {
			time_range: {
				start: ANALYTICS_START_TIMESTAMP,
				end: new Date(queryRefreshTimestamp.value).toISOString(),
				resolution: {
					slices: 1,
				},
			},
			project_ids: sortedSelectedProjectIds.value,
			return_metrics: {
				project_downloads: {
					bucket_by: ['version_id'],
				},
				project_playtime: {
					bucket_by: ['game_version', 'version_id'],
				},
			},
		}
	})

	const { data: versionFilterOptionsData } = useQuery({
		queryKey: computed(() => [
			'analytics',
			'dashboard',
			'filter-options',
			'versions',
			versionFilterOptionsRequest.value,
		]),
		queryFn: () =>
			client.labrinth.analytics_v3.fetch(
				versionFilterOptionsRequest.value as Labrinth.Analytics.v3.FetchRequest,
			),
		enabled: computed(() => versionFilterOptionsRequest.value !== null),
	})

	const filterOptions = computed<AnalyticsDashboardFilterOptions>(() =>
		({
			countries: getCountryFilterOptions(countryFilterOptionsData.value ?? []),
			gameVersions: getGameVersionFilterOptions(versionFilterOptionsData.value ?? []),
			versionIds: getVersionFilterOptions(versionFilterOptionsData.value ?? []),
		}),
	)

	const previousFetchRequest = computed(() => buildPreviousFetchRequest(fetchRequest.value))

	const {
		data: previousTimeSliceData,
		isPending: previousTimeSlicePending,
		isFetching: previousFetching,
		refetch: refetchPreviousTimeSlices,
	} = useQuery({
		queryKey: computed(() => ['analytics', 'dashboard', 'previous', previousFetchRequest.value]),
		queryFn: () =>
			client.labrinth.analytics_v3.fetch(
				previousFetchRequest.value as Labrinth.Analytics.v3.FetchRequest,
			),
		enabled: computed(() => previousFetchRequest.value !== null),
	})

	const timeSlices = ref<Labrinth.Analytics.v3.TimeSlice[]>([])
	const previousTimeSlices = ref<Labrinth.Analytics.v3.TimeSlice[]>([])

	watch(
		currentTimeSliceData,
		(nextTimeSlices) => {
			if (nextTimeSlices === undefined) {
				return
			}
			timeSlices.value = nextTimeSlices
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

	watch(fetchRequest, (nextFetchRequest) => {
		if (nextFetchRequest !== null) {
			return
		}
		timeSlices.value = []
		previousTimeSlices.value = []
	})

	const analyticsVersionIds = computed(() => {
		const versionIds = new Set<string>()
		for (const versionId of selectedFilters.value.version_id) {
			const normalizedVersionId = versionId.trim()
			if (normalizedVersionId.length > 0) {
				versionIds.add(normalizedVersionId)
			}
		}

		for (const versionId of filterOptions.value.versionIds) {
			versionIds.add(versionId)
		}

		addVersionIdsFromTimeSlices(versionIds, timeSlices.value)
		addVersionIdsFromTimeSlices(versionIds, previousTimeSlices.value)

		return sortStringValues([...versionIds])
	})

	const { data: versions } = useQuery({
		queryKey: computed(() => ['analytics', 'dashboard', 'versions', analyticsVersionIds.value]),
		queryFn: () =>
			fetchSegmentedWith(analyticsVersionIds.value, (ids) =>
				client.labrinth.versions_v3.getVersions(ids),
			),
		enabled: computed(() => analyticsVersionIds.value.length > 0),
		placeholderData: [],
	})

	const versionNumbersById = computed(
		() => new Map((versions.value ?? []).map((version) => [version.id, version.version_number])),
	)

	const selectedProjectIdSet = computed(() => new Set(selectedProjectIds.value))
	const availableProjectIdSet = computed(() => new Set(availableProjectIds.value))

	const currentTotals = computed<AnalyticsDashboardTotals>(() =>
		computeTotals(
			timeSlices.value,
			selectedProjectIdSet.value,
			availableProjectIdSet.value,
			selectedFilters.value,
		),
	)
	const previousTotals = computed<AnalyticsDashboardTotals>(() =>
		computeTotals(
			previousTimeSlices.value,
			selectedProjectIdSet.value,
			availableProjectIdSet.value,
			selectedFilters.value,
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

	async function refreshAnalyticsQuery() {
		if (fetchRequest.value === null) {
			return
		}

		const previousFetchRequestKey = JSON.stringify(fetchRequest.value)
		const now = Date.now()
		queryRefreshTimestamp.value =
			now > queryRefreshTimestamp.value ? now : queryRefreshTimestamp.value + 1
		await nextTick()

		if (
			fetchRequest.value === null ||
			JSON.stringify(fetchRequest.value) !== previousFetchRequestKey
		) {
			return
		}

		const refetches = [refetchCurrentTimeSlices()]
		if (previousFetchRequest.value !== null) {
			refetches.push(refetchPreviousTimeSlices())
		}

		await Promise.all(refetches)
	}

	function setFetchRequest(nextFetchRequest: Labrinth.Analytics.v3.FetchRequest) {
		fetchRequest.value = nextFetchRequest
	}

	function getVersionDisplayName(versionId: string): string {
		return versionNumbersById.value.get(versionId) ?? versionId
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
		selectedTimeframeMode,
		selectedTimeframe,
		selectedLastTimeframeAmount,
		selectedLastTimeframeUnit,
		selectedCustomTimeframeStartDate,
		selectedCustomTimeframeEndDate,
		selectedGroupBy,
		selectedBreakdown,
		selectedFilters,
		queryRefreshTimestamp,
		fetchRequest,
		filterOptions,
		versionNumbersById,
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
		refreshAnalyticsQuery,
		getVersionDisplayName,
		setFetchRequest,
		setActiveStat,
	}
}
