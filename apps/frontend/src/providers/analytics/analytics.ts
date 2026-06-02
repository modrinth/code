import type { Labrinth } from '@modrinth/api-client'
import {
	createContext,
	injectModrinthClient,
	injectNotificationManager,
	type ProjectPageContext,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import type { ComputedRef, Ref } from 'vue'

import {
	areSelectedFiltersEqual,
	areStringArraysEqual,
	buildDefaultAnalyticsGraphState,
	buildDefaultAnalyticsQueryBuilderState,
	getAnalyticsBreakdownPresetsForProjectSelection,
	getDefaultAnalyticsBreakdownPresets,
	getDefaultAnalyticsGraphProjectEventsVisibility,
	hasAnalyticsBreakdownQuery,
	hasAnalyticsGraphProjectEventsVisibilityQuery,
	hasAnalyticsProjectSelectionQuery,
	isAnalyticsGraphStateDefault,
	isAnalyticsQueryBuilderStateDefault,
	readAnalyticsGraphState,
	readAnalyticsQueryBuilderState,
} from '~/components/analytics-dashboard/analytics-route-query'
import {
	getEnabledAnalyticsStatsForState,
	PROJECT_STATUS_FILTER_VALUES,
	type ProjectStatusFilterValue,
	sanitizeAnalyticsSelectedFilters,
} from '~/components/analytics-dashboard/query-builder/query-filter'
import { useAnalyticsRouteSync } from '~/components/analytics-dashboard/use-analytics-route-sync'

import type { OrganizationContext } from '../organization-context'
import {
	addVersionIdsFromTimeSlices,
	addVersionProjectNamesFromTimeSlices,
	ANALYTICS_START_TIME,
	areAnalyticsFetchRequestsEqual,
	buildAnalyticsCurrentTimeSlicesQueryKey,
	buildAnalyticsFacetsRequest,
	buildComparisonFetchRequest,
	buildDailyAnalyticsFetchRequest,
	cloneAnalyticsFetchRequest,
	computeTotals,
	fetchAnalyticsData,
	fetchAnalyticsTimeSlices,
	getAnalyticsProjectEventsInTimeRange,
	getAnalyticsTimeframeDurationMs,
	getCountryDownloadsByCodeFromTimeSlices,
	getGameVersionDownloadsByVersionFromTimeSlices,
	getPercentChange,
	getProjectDownloadsByIdFromTimeSlices,
	getProjectVersionDownloadsByIdFromTimeSlices,
	isAnalyticsFetchRequestReady,
	isRevenueHourlyGroupBy,
	REVENUE_MIN_TIMEFRAME_MS,
	splitAnalyticsTimeSlices,
} from './analytics-data-utils'
import {
	cloneAnalyticsFilterOptions,
	cloneAnalyticsSelectedFilters,
	fetchAnalyticsVersionMetadataByIds,
	getAnalyticsFacetsFilterOptionSummary,
	getAnalyticsVersionIdsFromProjects,
	getProjectVersionFilterOptionSummary,
	sanitizeAnalyticsSelectedFiltersForAvailableOptions,
	sortStringValues,
} from './analytics-filter-utils'
import {
	getProjectIdsMatchingStatusFilter,
	getProjectOrganizationId,
	getSingleQueryValue,
	getUniqueAnalyticsDashboardProjects,
	isAnalyticsEligibleProject,
	toAnalyticsDashboardProject,
	UNKNOWN_ORGANIZATION_NAME,
} from './analytics-project-utils'
import type {
	AnalyticsBreakdownPreset,
	AnalyticsDashboardFilterOptions,
	AnalyticsDashboardPercentChanges,
	AnalyticsDashboardProject,
	AnalyticsDashboardProjectGroup,
	AnalyticsDashboardStat,
	AnalyticsDashboardTotals,
	AnalyticsGraphViewMode,
	AnalyticsGroupByPreset,
	AnalyticsLastTimeframeUnit,
	AnalyticsProjectVersionSource,
	AnalyticsSelectedBreakdowns,
	AnalyticsSelectedFilters,
	AnalyticsTimeframeMode,
	AnalyticsTimeframePreset,
	AnalyticsVersionMetadata,
} from './analytics-types'

export { ANALYTICS_START_DATE_INPUT_VALUE } from './analytics-data-utils'
export {
	doesAnalyticsPointMatchFilters,
	doesAnalyticsPointMatchNormalizedFilters,
	normalizeAnalyticsSelectedFilters,
} from './analytics-filter-utils'
export {
	doesProjectStatusMatchFilters,
	getProjectIdsMatchingStatusFilter,
} from './analytics-project-utils'
export type {
	AnalyticsBreakdownPreset,
	AnalyticsDashboardFilterOptions,
	AnalyticsDashboardPercentChanges,
	AnalyticsDashboardProject,
	AnalyticsDashboardProjectGroup,
	AnalyticsDashboardStat,
	AnalyticsDashboardTotals,
	AnalyticsGraphState,
	AnalyticsGraphViewMode,
	AnalyticsGroupByPreset,
	AnalyticsLastTimeframeUnit,
	AnalyticsQueryFilterCategory,
	AnalyticsSelectedBreakdowns,
	AnalyticsSelectedFilters,
	AnalyticsTableSortColumn,
	AnalyticsTableSortDirection,
	AnalyticsTimeframeMode,
	AnalyticsTimeframePreset,
	NormalizedAnalyticsSelectedFilters,
} from './analytics-types'

const REVENUE_GROUP_BY_FALLBACK: AnalyticsGroupByPreset = 'day'
const ANALYTICS_TIME_SLICES_GC_TIME_MS = 30 * 1000
const ANALYTICS_PREFETCH_GC_TIME_MS = 15 * 1000
const ANALYTICS_FILTER_OPTIONS_GC_TIME_MS = 60 * 1000
const ANALYTICS_MOBILE_LAYOUT_QUERY = '(pointer: coarse), (max-width: 800px)'
const ANALYTICS_ALL_TIME_START_OFFSET_MONTHS = 2

function subtractAnalyticsCalendarMonths(date: Date, months: number): Date {
	const nextDate = new Date(date)
	const day = nextDate.getDate()
	nextDate.setDate(1)
	nextDate.setMonth(nextDate.getMonth() - months)
	const daysInMonth = new Date(nextDate.getFullYear(), nextDate.getMonth() + 1, 0).getDate()
	nextDate.setDate(Math.min(day, daysInMonth))
	return nextDate
}

function getAnalyticsFetchErrorMessage(error: unknown): string {
	if (error && typeof error === 'object') {
		const dataDescription = (error as { data?: { description?: unknown } }).data?.description
		if (typeof dataDescription === 'string' && dataDescription.length > 0) {
			return dataDescription
		}

		const message = (error as { message?: unknown }).message
		if (typeof message === 'string' && message.length > 0) {
			return message
		}
	}

	if (typeof error === 'string' && error.length > 0) {
		return error
	}

	return 'Please try refreshing the page or changing your query.'
}

export interface AnalyticsDashboardContextValue {
	hasProjectContext: ComputedRef<boolean>
	projectGroups: ComputedRef<AnalyticsDashboardProjectGroup[]>
	projects: ComputedRef<AnalyticsDashboardProject[]>
	selectedProjectIds: Ref<string[]>
	selectedTimeframeMode: Ref<AnalyticsTimeframeMode>
	selectedTimeframe: Ref<AnalyticsTimeframePreset>
	selectedLastTimeframeAmount: Ref<number>
	selectedLastTimeframeUnit: Ref<AnalyticsLastTimeframeUnit>
	selectedCustomTimeframeStartDate: Ref<string>
	selectedCustomTimeframeEndDate: Ref<string>
	selectedGroupBy: Ref<AnalyticsGroupByPreset>
	analyticsAllTimeStartDate: ComputedRef<Date>
	selectedBreakdowns: Ref<AnalyticsSelectedBreakdowns>
	selectedFilters: Ref<AnalyticsSelectedFilters>
	queryRefreshTimestamp: Ref<number>
	queryResetToken: Ref<number>
	isAnalyticsQueryBuilderDefault: ComputedRef<boolean>
	fetchRequest: Ref<Labrinth.Analytics.v3.FetchRequest | null>
	displayedSelectedProjectIds: Ref<string[]>
	displayedSelectedGroupBy: Ref<AnalyticsGroupByPreset>
	displayedSelectedBreakdowns: Ref<AnalyticsSelectedBreakdowns>
	displayedSelectedFilters: Ref<AnalyticsSelectedFilters>
	displayedFetchRequest: Ref<Labrinth.Analytics.v3.FetchRequest | null>
	displayedFilterOptions: Ref<AnalyticsDashboardFilterOptions>
	filterOptions: ComputedRef<AnalyticsDashboardFilterOptions>
	isAnalyticsFilterOptionsLoading: ComputedRef<boolean>
	versionNumbersById: ComputedRef<Map<string, string>>
	versionPublishedDatesById: ComputedRef<Map<string, string>>
	versionProjectNamesById: ComputedRef<Map<string, string>>
	versionProjectIconUrlsById: ComputedRef<Map<string, string>>
	projectStatusById: ComputedRef<Map<string, ProjectStatusFilterValue>>
	availableProjectStatuses: ComputedRef<ProjectStatusFilterValue[]>
	projectDownloadsById: ComputedRef<Map<string, number>>
	projectVersionDownloadsById: ComputedRef<Map<string, number>>
	gameVersionDownloadsByVersion: ComputedRef<Map<string, number>>
	countryDownloadsByCode: ComputedRef<Map<string, number>>
	timeSlices: Ref<Labrinth.Analytics.v3.TimeSlice[]>
	displayedTimeSlices: Ref<Labrinth.Analytics.v3.TimeSlice[]>
	previousTimeSlices: Ref<Labrinth.Analytics.v3.TimeSlice[]>
	displayedPreviousTimeSlices: Ref<Labrinth.Analytics.v3.TimeSlice[]>
	projectEvents: Ref<Labrinth.Analytics.v3.ProjectAnalyticsEvent[]>
	displayedProjectEvents: Ref<Labrinth.Analytics.v3.ProjectAnalyticsEvent[]>
	hasCompletedAnalyticsLoading: Ref<boolean>
	isLoading: ComputedRef<boolean>
	isRefetching: ComputedRef<boolean>
	activeStat: Ref<AnalyticsDashboardStat>
	activeGraphViewMode: Ref<AnalyticsGraphViewMode>
	isRatioMode: Ref<boolean>
	showChartEvents: Ref<boolean>
	showProjectEvents: Ref<boolean>
	showPreviousPeriod: Ref<boolean>
	isMobileLayout: Ref<boolean>
	hiddenGraphDatasetIds: Ref<string[]>
	hasExplicitGraphDatasetSelection: Ref<boolean>
	isGraphDatasetSelectionActive: Ref<boolean>
	selectedGraphDatasetIds: Ref<string[]>
	defaultGraphDatasetIds: Ref<string[]>
	topGraphDatasetIds: Ref<string[]>
	currentTotals: ComputedRef<AnalyticsDashboardTotals>
	previousTotals: ComputedRef<AnalyticsDashboardTotals>
	percentChanges: ComputedRef<AnalyticsDashboardPercentChanges>
	hasPreviousPeriodComparison: ComputedRef<boolean>
	getRelevantAnalyticsDashboardStats: (
		breakdowns: readonly AnalyticsBreakdownPreset[],
		filters?: AnalyticsSelectedFilters,
	) => readonly AnalyticsDashboardStat[]
	isAnalyticsDashboardStatRelevant: (
		stat: AnalyticsDashboardStat,
		breakdowns: readonly AnalyticsBreakdownPreset[],
		filters?: AnalyticsSelectedFilters,
	) => boolean
	refreshAnalyticsQuery: () => Promise<void>
	resetAnalyticsQueryBuilder: () => void
	getVersionDisplayName: (versionId: string) => string
	getVersionPublishedDate: (versionId: string) => string | undefined
	getVersionProjectName: (versionId: string) => string | undefined
	getVersionProjectIconUrl: (versionId: string) => string | undefined
	setFetchRequest: (fetchRequest: Labrinth.Analytics.v3.FetchRequest) => void
	setActiveStat: (stat: AnalyticsDashboardStat) => void
}

export type CreateAnalyticsDashboardContextOptions = {
	auth: Ref<{ user?: { id?: string; username?: string; role?: string } | null }>
	projectPageContext?: ProjectPageContext | null
	organizationContext?: OrganizationContext | null
}

export const [injectAnalyticsDashboardContext, provideAnalyticsDashboardContext] =
	createContext<AnalyticsDashboardContextValue>('AnalyticsDashboard')

export function createAnalyticsDashboardContext(
	options: CreateAnalyticsDashboardContextOptions,
): AnalyticsDashboardContextValue {
	const client = injectModrinthClient()
	const { addNotification } = injectNotificationManager()
	const queryClient = useQueryClient()
	const route = useRoute()
	const initialQueryState = readAnalyticsQueryBuilderState(route.query, [])
	const initialGraphState = readAnalyticsGraphState(
		route.query,
		initialQueryState.selectedProjectIds,
	)

	const activeStat = ref<AnalyticsDashboardStat>(initialGraphState.activeStat)
	const activeGraphViewMode = ref<AnalyticsGraphViewMode>(initialGraphState.activeGraphViewMode)
	const isRatioMode = ref(initialGraphState.isRatioMode)
	const showChartEvents = ref(initialGraphState.showChartEvents)
	const showProjectEvents = ref(initialGraphState.showProjectEvents)
	const showPreviousPeriod = ref(initialGraphState.showPreviousPeriod)
	const isMobileLayout = ref(false)
	const hiddenGraphDatasetIds = ref<string[]>(initialGraphState.hiddenGraphDatasetIds)
	const hasExplicitGraphDatasetSelection = ref(initialGraphState.selectedGraphDatasetIds !== null)
	const isGraphDatasetSelectionActive = ref(false)
	const selectedGraphDatasetIds = ref<string[]>(initialGraphState.selectedGraphDatasetIds ?? [])
	const defaultGraphDatasetIds = ref<string[]>([])
	const topGraphDatasetIds = ref<string[]>([])
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
	const selectedBreakdowns = ref<AnalyticsSelectedBreakdowns>(initialQueryState.selectedBreakdowns)
	const selectedFilters = ref<AnalyticsSelectedFilters>(initialQueryState.selectedFilters)
	const queryRefreshTimestamp = ref(Date.now())
	const queryResetToken = ref(0)
	const fetchRequest = ref<Labrinth.Analytics.v3.FetchRequest | null>(null)
	const hasInitializedAnalyticsFetchRequest = ref(false)
	const hasCompletedAnalyticsLoading = ref(false)
	let revenueHourlyGroupByBeforeOverride: AnalyticsGroupByPreset | null = null
	let mobileLayoutMedia: MediaQueryList | null = null

	function syncMobileLayoutState() {
		isMobileLayout.value = mobileLayoutMedia?.matches ?? false
	}

	function setupMobileLayoutMedia() {
		if (typeof window === 'undefined') {
			return
		}

		mobileLayoutMedia = window.matchMedia(ANALYTICS_MOBILE_LAYOUT_QUERY)
		syncMobileLayoutState()
		mobileLayoutMedia.addEventListener('change', syncMobileLayoutState)
	}

	function teardownMobileLayoutMedia() {
		mobileLayoutMedia?.removeEventListener('change', syncMobileLayoutState)
		mobileLayoutMedia = null
	}

	onMounted(() => {
		setupMobileLayoutMedia()
	})

	onBeforeUnmount(() => {
		teardownMobileLayoutMedia()
	})

	const hasProjectContext = computed(() => Boolean(options.projectPageContext))
	const hasOrganizationContext = computed(
		() => !hasProjectContext.value && Boolean(options.organizationContext),
	)
	const isDashboardAnalyticsRoute = computed(
		() => route.path.replace(/\/$/, '') === '/dashboard/analytics',
	)
	const requestedDashboardUserId = computed(() => {
		if (!isDashboardAnalyticsRoute.value || options.auth.value.user?.role !== 'admin') {
			return undefined
		}

		return getSingleQueryValue(route.query.user)
	})
	const effectiveUserId = computed(
		() => requestedDashboardUserId.value ?? options.auth.value.user?.id,
	)
	const isUsingDashboardUserOverride = computed(() => requestedDashboardUserId.value !== undefined)
	const analyticsQueryUserId = computed(() => effectiveUserId.value ?? options.auth.value.user?.id)
	const shouldFetchEffectiveUser = computed(
		() =>
			Boolean(effectiveUserId.value) &&
			!hasProjectContext.value &&
			!hasOrganizationContext.value &&
			(effectiveUserId.value !== options.auth.value.user?.id || !options.auth.value.user?.username),
	)
	const shouldFetchDashboardAllProjects = computed(
		() =>
			Boolean(effectiveUserId.value) &&
			isDashboardAnalyticsRoute.value &&
			!hasProjectContext.value &&
			!hasOrganizationContext.value,
	)

	const { data: effectiveUser } = useQuery({
		queryKey: computed(() => ['analytics', 'dashboard', effectiveUserId.value, 'user']),
		queryFn: async () => {
			try {
				return await client.labrinth.users_v2.get(effectiveUserId.value ?? '')
			} catch (error) {
				if (isUsingDashboardUserOverride.value) {
					return null
				}

				throw error
			}
		},
		enabled: computed(() => shouldFetchEffectiveUser.value && hasCompletedAnalyticsLoading.value),
		placeholderData: null,
		refetchOnWindowFocus: false,
	})
	const effectiveUsername = computed(() => {
		if (effectiveUserId.value === options.auth.value.user?.id) {
			return (
				options.auth.value.user?.username ??
				effectiveUser.value?.username ??
				effectiveUserId.value ??
				'User'
			)
		}

		return effectiveUser.value?.username ?? effectiveUserId.value ?? 'User'
	})

	const { data: dashboardAllProjects, isFetched: hasFetchedDashboardAllProjects } = useQuery({
		queryKey: computed(() => ['analytics', 'dashboard', effectiveUserId.value, 'all-projects']),
		queryFn: async (): Promise<Labrinth.Users.v3.AllProjectsResponse> => {
			try {
				const user = effectiveUserId.value
				if (!user) {
					return {
						projects: [],
						organizations: {},
					}
				}

				return await client.labrinth.users_v3.getAllProjects(user)
			} catch (error) {
				if (isUsingDashboardUserOverride.value) {
					return {
						projects: [],
						organizations: {},
					}
				}

				throw error
			}
		},
		enabled: shouldFetchDashboardAllProjects,
		refetchOnWindowFocus: false,
	})

	const areProjectsLoaded = computed(() => {
		if (hasProjectContext.value) {
			return true
		}

		if (hasOrganizationContext.value) {
			return options.organizationContext?.projects.value !== null
		}

		if (shouldFetchDashboardAllProjects.value) {
			return hasFetchedDashboardAllProjects.value
		}

		return true
	})

	const projectGroups = computed<AnalyticsDashboardProjectGroup[]>(() => {
		if (hasProjectContext.value && options.projectPageContext) {
			const project = options.projectPageContext.projectV2.value
			return project && isAnalyticsEligibleProject(project)
				? [
						{
							projects: [toAnalyticsDashboardProject(project)],
						},
					]
				: []
		}

		if (hasOrganizationContext.value && options.organizationContext?.projects.value) {
			return [
				{
					projects: getUniqueAnalyticsDashboardProjects(
						options.organizationContext.projects.value,
						new Set(),
					),
				},
			]
		}

		if (shouldFetchDashboardAllProjects.value) {
			const response = dashboardAllProjects.value
			if (!response) {
				return []
			}

			const seenProjectIds = new Set<string>()
			const personalProjects: AnalyticsDashboardProject[] = []
			const organizationGroupsById = new Map<string, AnalyticsDashboardProjectGroup>()

			for (const project of response.projects) {
				if (seenProjectIds.has(project.id) || !isAnalyticsEligibleProject(project)) {
					continue
				}

				seenProjectIds.add(project.id)
				const organizationId = getProjectOrganizationId(project)
				const analyticsProject = toAnalyticsDashboardProject(project)
				if (!organizationId) {
					personalProjects.push(analyticsProject)
					continue
				}

				const organizationGroup = organizationGroupsById.get(organizationId) ?? {
					key: organizationId,
					title: response.organizations[organizationId]?.name ?? UNKNOWN_ORGANIZATION_NAME,
					projects: [],
				}
				organizationGroup.projects.push(analyticsProject)
				organizationGroupsById.set(organizationId, organizationGroup)
			}

			const organizationGroups = [...organizationGroupsById.values()]
			if (personalProjects.length === 0) {
				return organizationGroups
			}

			return [
				{
					key: organizationGroups.length > 0 ? `user-${effectiveUserId.value}` : undefined,
					title: organizationGroups.length > 0 ? effectiveUsername.value : undefined,
					projects: personalProjects,
				},
				...organizationGroups,
			]
		}

		return []
	})

	const projects = computed<AnalyticsDashboardProject[]>(() =>
		projectGroups.value.flatMap((group) => group.projects),
	)

	const availableProjectIds = computed(() => projects.value.map((project) => project.id))
	const projectNamesById = computed(
		() => new Map(projects.value.map((project) => [project.id, project.name])),
	)
	const projectIconUrlsById = computed(
		() =>
			new Map(
				projects.value
					.filter((project) => project.iconUrl)
					.map((project) => [project.id, project.iconUrl as string]),
			),
	)
	const projectStatusById = computed(
		() => new Map(projects.value.map((project) => [project.id, project.status])),
	)
	const availableProjectStatuses = computed<ProjectStatusFilterValue[]>(() => {
		const presentStatuses = new Set(projects.value.map((project) => project.status))
		return PROJECT_STATUS_FILTER_VALUES.filter((status) => presentStatuses.has(status))
	})
	const sortedSelectedProjectIds = computed(() => sortStringValues(selectedProjectIds.value))
	const filterOptionProjectSources = computed<AnalyticsProjectVersionSource[] | null>(() => {
		if (hasProjectContext.value && options.projectPageContext) {
			const project =
				options.projectPageContext.projectV3.value ?? options.projectPageContext.projectV2.value
			return project ? [project] : []
		}

		if (hasOrganizationContext.value) {
			return options.organizationContext?.projects.value ?? null
		}

		if (shouldFetchDashboardAllProjects.value) {
			return dashboardAllProjects.value?.projects ?? null
		}

		return []
	})
	const filterOptionVersionIds = computed(() => {
		const projects = filterOptionProjectSources.value
		if (!projects) {
			return []
		}

		return getAnalyticsVersionIdsFromProjects(projects, sortedSelectedProjectIds.value)
	})
	const { data: filterOptionProjectVersions, isFetched: hasFetchedFilterOptionProjectVersions } =
		useQuery({
			queryKey: computed(() => [
				'analytics',
				'dashboard',
				analyticsQueryUserId.value,
				'filter-options',
				'versions',
				filterOptionVersionIds.value,
			]),
			queryFn: () =>
				fetchAnalyticsVersionMetadataByIds(filterOptionVersionIds.value, (ids) =>
					client.labrinth.versions_v3.getVersions(ids),
				),
			enabled: computed(
				() =>
					filterOptionProjectSources.value !== null && sortedSelectedProjectIds.value.length > 0,
			),
			placeholderData: [],
			gcTime: ANALYTICS_FILTER_OPTIONS_GC_TIME_MS,
			refetchOnWindowFocus: false,
		})

	const projectsById = computed(
		() => new Map(projects.value.map((project) => [project.id, project])),
	)
	const analyticsAllTimeStartDate = computed(() => {
		const fallbackStartDate = new Date(ANALYTICS_START_TIME)
		const filteredProjectIds = getProjectIdsMatchingStatusFilter(
			selectedProjectIds.value.length > 0 ? selectedProjectIds.value : availableProjectIds.value,
			projectStatusById.value,
			selectedFilters.value,
		)
		let startTime = Number.POSITIVE_INFINITY

		for (const projectId of filteredProjectIds) {
			const publishedAt = projectsById.value.get(projectId)?.publishedAt
			if (!publishedAt) {
				continue
			}

			const projectStartTime = new Date(publishedAt).getTime()
			if (Number.isFinite(projectStartTime)) {
				startTime = Math.min(startTime, projectStartTime)
			}
		}

		if (!Number.isFinite(startTime)) {
			return fallbackStartDate
		}

		const offsetStartDate = subtractAnalyticsCalendarMonths(
			new Date(startTime),
			ANALYTICS_ALL_TIME_START_OFFSET_MONTHS,
		)
		return new Date(Math.max(offsetStartDate.getTime(), ANALYTICS_START_TIME))
	})
	const hasExplicitProjectSelectionQuery = computed(() =>
		hasAnalyticsProjectSelectionQuery(route.query),
	)
	const hasExplicitBreakdownQuery = computed(() => hasAnalyticsBreakdownQuery(route.query))
	const hasExplicitProjectEventsVisibilityQuery = computed(() =>
		hasAnalyticsGraphProjectEventsVisibilityQuery(route.query),
	)
	const isAnalyticsQueryBuilderDefault = computed(() => {
		const isQueryBuilderDefault = isAnalyticsQueryBuilderStateDefault(
			{
				selectedProjectIds: selectedProjectIds.value,
				selectedTimeframeMode: selectedTimeframeMode.value,
				selectedTimeframe: selectedTimeframe.value,
				selectedLastTimeframeAmount: selectedLastTimeframeAmount.value,
				selectedLastTimeframeUnit: selectedLastTimeframeUnit.value,
				selectedCustomTimeframeStartDate: selectedCustomTimeframeStartDate.value,
				selectedCustomTimeframeEndDate: selectedCustomTimeframeEndDate.value,
				selectedGroupBy: selectedGroupBy.value,
				selectedBreakdowns: selectedBreakdowns.value,
				selectedFilters: selectedFilters.value,
			},
			availableProjectIds.value,
		)
		const isGraphDefault = isAnalyticsGraphStateDefault(
			{
				activeStat: activeStat.value,
				activeGraphViewMode: activeGraphViewMode.value,
				isRatioMode: isRatioMode.value,
				showChartEvents: showChartEvents.value,
				showProjectEvents: showProjectEvents.value,
				showPreviousPeriod: showPreviousPeriod.value,
				hiddenGraphDatasetIds: hiddenGraphDatasetIds.value,
				selectedGraphDatasetIds: hasExplicitGraphDatasetSelection.value
					? selectedGraphDatasetIds.value
					: null,
			},
			selectedProjectIds.value,
		)

		return isQueryBuilderDefault && isGraphDefault
	})
	const isRevenueTimeframeAvailable = computed(
		() =>
			getAnalyticsTimeframeDurationMs({
				mode: selectedTimeframeMode.value,
				preset: selectedTimeframe.value,
				lastAmount: selectedLastTimeframeAmount.value,
				lastUnit: selectedLastTimeframeUnit.value,
				customStartDate: selectedCustomTimeframeStartDate.value,
				customEndDate: selectedCustomTimeframeEndDate.value,
				nowTimestamp: queryRefreshTimestamp.value,
				allTimeStartTimestamp: analyticsAllTimeStartDate.value.getTime(),
			}) > REVENUE_MIN_TIMEFRAME_MS,
	)

	function isAnalyticsDashboardStatAvailableForTimeframe(stat: AnalyticsDashboardStat): boolean {
		return stat !== 'revenue' || isRevenueTimeframeAvailable.value
	}

	function getRelevantAnalyticsDashboardStats(
		breakdowns: readonly AnalyticsBreakdownPreset[],
		filters: AnalyticsSelectedFilters = selectedFilters.value,
	): readonly AnalyticsDashboardStat[] {
		return getEnabledAnalyticsStatsForState(breakdowns, filters).filter((stat) =>
			isAnalyticsDashboardStatAvailableForTimeframe(stat),
		)
	}

	function isAnalyticsDashboardStatRelevant(
		stat: AnalyticsDashboardStat,
		breakdowns: readonly AnalyticsBreakdownPreset[],
		filters: AnalyticsSelectedFilters = selectedFilters.value,
	): boolean {
		return getRelevantAnalyticsDashboardStats(breakdowns, filters).includes(stat)
	}

	function sanitizeAnalyticsSelectedFiltersForContext(
		breakdowns: readonly AnalyticsBreakdownPreset[],
		filters: AnalyticsSelectedFilters,
	): AnalyticsSelectedFilters {
		const nextFilters = sanitizeAnalyticsSelectedFilters(breakdowns, filters)
		if (hasProjectContext.value && nextFilters.project_status.length > 0) {
			return {
				...nextFilters,
				project_status: [],
			}
		}

		return nextFilters
	}

	const {
		replaceNextAnalyticsRouteNavigation,
		syncQueryBuilderRouteQuery,
		syncGraphRouteQuery,
		applyRouteQueryToState,
	} = useAnalyticsRouteSync({
		queryBuilder: {
			selectedProjectIds,
			selectedTimeframeMode,
			selectedTimeframe,
			selectedLastTimeframeAmount,
			selectedLastTimeframeUnit,
			selectedCustomTimeframeStartDate,
			selectedCustomTimeframeEndDate,
			selectedGroupBy,
			selectedBreakdowns,
			selectedFilters,
		},
		graph: {
			activeStat,
			activeGraphViewMode,
			isRatioMode,
			showChartEvents,
			showProjectEvents,
			showPreviousPeriod,
			hiddenGraphDatasetIds,
			hasExplicitGraphDatasetSelection,
			selectedGraphDatasetIds,
		},
		availableProjectIds,
		sanitizeSelectedFilters: sanitizeAnalyticsSelectedFiltersForContext,
	})

	function reconcileRevenueGroupBy(nextActiveStat: AnalyticsDashboardStat) {
		if (nextActiveStat === 'revenue') {
			if (!isRevenueHourlyGroupBy(selectedGroupBy.value)) {
				if (selectedGroupBy.value !== REVENUE_GROUP_BY_FALLBACK) {
					revenueHourlyGroupByBeforeOverride = null
				}
				return
			}

			revenueHourlyGroupByBeforeOverride = selectedGroupBy.value
			replaceNextAnalyticsRouteNavigation()
			selectedGroupBy.value = REVENUE_GROUP_BY_FALLBACK
			return
		}

		const groupByBeforeRevenue = revenueHourlyGroupByBeforeOverride
		revenueHourlyGroupByBeforeOverride = null
		if (groupByBeforeRevenue && selectedGroupBy.value === REVENUE_GROUP_BY_FALLBACK) {
			replaceNextAnalyticsRouteNavigation()
			selectedGroupBy.value = groupByBeforeRevenue
		}
	}

	watch(
		[selectedBreakdowns, selectedFilters, activeStat, isRevenueTimeframeAvailable],
		([nextBreakdowns, nextFilters, nextActiveStat]) => {
			if (isAnalyticsDashboardStatRelevant(nextActiveStat, nextBreakdowns, nextFilters)) {
				return
			}

			const fallbackStat = getRelevantAnalyticsDashboardStats(nextBreakdowns, nextFilters)[0]
			if (fallbackStat && fallbackStat !== nextActiveStat) {
				activeStat.value = fallbackStat
			}
		},
		{ deep: true, immediate: true },
	)

	watch(
		[selectedBreakdowns, selectedFilters],
		([nextBreakdowns, nextFilters]) => {
			const sanitizedFilters = sanitizeAnalyticsSelectedFiltersForContext(
				nextBreakdowns,
				nextFilters,
			)
			if (!areSelectedFiltersEqual(nextFilters, sanitizedFilters)) {
				replaceNextAnalyticsRouteNavigation()
				selectedFilters.value = sanitizedFilters
			}
		},
		{ deep: true, immediate: true },
	)

	function syncSelectedBreakdownsForProjectSelection(nextSelectedProjectIds: string[]) {
		const validBreakdowns = getAnalyticsBreakdownPresetsForProjectSelection(
			selectedBreakdowns.value,
			nextSelectedProjectIds,
		)
		if (!areStringArraysEqual(selectedBreakdowns.value, validBreakdowns)) {
			replaceNextAnalyticsRouteNavigation()
			selectedBreakdowns.value = validBreakdowns
			return
		}

		const defaultBreakdowns = getDefaultAnalyticsBreakdownPresets(nextSelectedProjectIds)
		if (
			!hasExplicitBreakdownQuery.value &&
			!areStringArraysEqual(selectedBreakdowns.value, defaultBreakdowns)
		) {
			replaceNextAnalyticsRouteNavigation()
			selectedBreakdowns.value = defaultBreakdowns
		}
	}

	function syncProjectEventsVisibilityForProjectSelection(nextSelectedProjectIds: string[]) {
		const defaultProjectEventsVisibility =
			getDefaultAnalyticsGraphProjectEventsVisibility(nextSelectedProjectIds)

		if (
			!hasExplicitProjectEventsVisibilityQuery.value &&
			showProjectEvents.value !== defaultProjectEventsVisibility
		) {
			replaceNextAnalyticsRouteNavigation()
			showProjectEvents.value = defaultProjectEventsVisibility
		}
	}

	watch(
		[projects, areProjectsLoaded],
		([nextProjects, nextAreProjectsLoaded]) => {
			if (nextProjects.length === 0) {
				if (nextAreProjectsLoaded) {
					syncSelectedBreakdownsForProjectSelection([])
					syncProjectEventsVisibilityForProjectSelection([])
				}
				if (nextAreProjectsLoaded && selectedProjectIds.value.length > 0) {
					replaceNextAnalyticsRouteNavigation()
					selectedProjectIds.value = []
				}
				return
			}

			const availableProjectIds = new Set(nextProjects.map((project) => project.id))
			if (!hasExplicitProjectSelectionQuery.value) {
				const nextSelectedProjectIds = nextProjects.map((project) => project.id)
				syncSelectedBreakdownsForProjectSelection(nextSelectedProjectIds)
				syncProjectEventsVisibilityForProjectSelection(nextSelectedProjectIds)
				if (!areStringArraysEqual(selectedProjectIds.value, nextSelectedProjectIds)) {
					replaceNextAnalyticsRouteNavigation()
					selectedProjectIds.value = nextSelectedProjectIds
				}
				return
			}

			const retainedSelection = selectedProjectIds.value.filter((id) => availableProjectIds.has(id))
			const nextSelectedProjectIds =
				retainedSelection.length > 0 ? retainedSelection : nextProjects.map((project) => project.id)

			syncSelectedBreakdownsForProjectSelection(nextSelectedProjectIds)
			syncProjectEventsVisibilityForProjectSelection(nextSelectedProjectIds)
			if (!areStringArraysEqual(selectedProjectIds.value, nextSelectedProjectIds)) {
				replaceNextAnalyticsRouteNavigation()
				selectedProjectIds.value = nextSelectedProjectIds
			}
		},
		{ immediate: true },
	)

	watch(
		[selectedProjectIds, hasExplicitBreakdownQuery],
		([nextSelectedProjectIds]) => {
			syncSelectedBreakdownsForProjectSelection(nextSelectedProjectIds)
		},
		{ deep: true, immediate: true },
	)

	watch(
		[selectedProjectIds, hasExplicitProjectEventsVisibilityQuery],
		([nextSelectedProjectIds]) => {
			syncProjectEventsVisibilityForProjectSelection(nextSelectedProjectIds)
		},
		{ deep: true, immediate: true },
	)

	watch(
		() => route.query,
		(nextQuery) => {
			applyRouteQueryToState(nextQuery)
		},
	)

	watch(
		[activeStat, selectedGroupBy],
		([nextActiveStat]) => {
			reconcileRevenueGroupBy(nextActiveStat)
		},
		{ flush: 'sync', immediate: true },
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
			selectedBreakdowns,
			selectedFilters,
			availableProjectIds,
		],
		() => {
			syncQueryBuilderRouteQuery()
		},
		{ deep: true, immediate: true },
	)

	watch(
		[
			activeStat,
			activeGraphViewMode,
			isRatioMode,
			showChartEvents,
			showProjectEvents,
			showPreviousPeriod,
			hiddenGraphDatasetIds,
			hasExplicitGraphDatasetSelection,
			selectedGraphDatasetIds,
		],
		() => {
			syncGraphRouteQuery()
		},
		{ deep: true },
	)

	const analyticsComparisonStartTime = computed(() => {
		if (selectedTimeframeMode.value === 'preset' && selectedTimeframe.value === 'all_time') {
			const fetchRequestStart = fetchRequest.value?.time_range.start
			return new Date(fetchRequestStart ?? analyticsAllTimeStartDate.value).getTime()
		}

		return ANALYTICS_START_TIME
	})
	const comparisonFetchRequest = computed(() =>
		buildComparisonFetchRequest(fetchRequest.value, analyticsComparisonStartTime.value),
	)
	const analyticsTimeSlicesFetchRequest = computed(
		() => comparisonFetchRequest.value ?? fetchRequest.value,
	)
	const hasPreviousPeriodComparison = computed(() => comparisonFetchRequest.value !== null)

	const {
		data: currentAnalyticsData,
		isPending: currentTimeSlicePending,
		isFetching: currentFetching,
		error: currentAnalyticsError,
		refetch: refetchCurrentTimeSlices,
	} = useQuery({
		queryKey: computed(() =>
			buildAnalyticsCurrentTimeSlicesQueryKey(
				analyticsQueryUserId.value,
				analyticsTimeSlicesFetchRequest.value,
				queryRefreshTimestamp.value,
			),
		),
		queryFn: () => {
			const nextFetchRequest = analyticsTimeSlicesFetchRequest.value
			if (!isAnalyticsFetchRequestReady(nextFetchRequest)) {
				return {
					metrics: [],
					project_events: [],
				}
			}

			return fetchAnalyticsData(nextFetchRequest, (request) =>
				client.labrinth.analytics_v3.fetch(request),
			)
		},
		enabled: computed(() => isAnalyticsFetchRequestReady(analyticsTimeSlicesFetchRequest.value)),
		refetchOnWindowFocus: false,
		gcTime: ANALYTICS_TIME_SLICES_GC_TIME_MS,
	})
	watch(currentAnalyticsError, (error) => {
		if (!error) {
			return
		}

		addNotification({
			title: 'Analytics failed to load',
			text: getAnalyticsFetchErrorMessage(error),
			type: 'error',
		})
	})
	const isCurrentTimeSliceLoading = computed(
		() =>
			isAnalyticsFetchRequestReady(analyticsTimeSlicesFetchRequest.value) &&
			currentTimeSlicePending.value,
	)
	const hasCompletedCurrentTimeSliceFetch = computed(
		() =>
			isAnalyticsFetchRequestReady(analyticsTimeSlicesFetchRequest.value) &&
			currentAnalyticsData.value !== undefined &&
			!currentTimeSlicePending.value &&
			!currentFetching.value,
	)
	watch(
		[
			hasInitializedAnalyticsFetchRequest,
			areProjectsLoaded,
			analyticsTimeSlicesFetchRequest,
			hasCompletedCurrentTimeSliceFetch,
		],
		([hasInitializedFetchRequest, nextAreProjectsLoaded, nextFetchRequest, hasCompletedFetch]) => {
			if (!hasInitializedFetchRequest || !nextAreProjectsLoaded) {
				hasCompletedAnalyticsLoading.value = false
				return
			}

			hasCompletedAnalyticsLoading.value = isAnalyticsFetchRequestReady(nextFetchRequest)
				? hasCompletedFetch
				: true
		},
		{ deep: true, immediate: true },
	)
	const revenueDailyPrefetchRequest = computed<Labrinth.Analytics.v3.FetchRequest | null>(() => {
		if (!isRevenueHourlyGroupBy(selectedGroupBy.value)) {
			return null
		}
		if (
			!isAnalyticsDashboardStatRelevant('revenue', selectedBreakdowns.value, selectedFilters.value)
		) {
			return null
		}

		const dailyFetchRequest = buildDailyAnalyticsFetchRequest(fetchRequest.value)
		return (
			buildComparisonFetchRequest(dailyFetchRequest, analyticsComparisonStartTime.value) ??
			dailyFetchRequest
		)
	})

	watch(
		[
			revenueDailyPrefetchRequest,
			analyticsQueryUserId,
			queryRefreshTimestamp,
			hasCompletedCurrentTimeSliceFetch,
		],
		([
			nextFetchRequest,
			nextAnalyticsQueryUserId,
			nextQueryRefreshTimestamp,
			hasCompletedCurrentFetch,
		]) => {
			if (!isAnalyticsFetchRequestReady(nextFetchRequest) || !hasCompletedCurrentFetch) {
				return
			}

			void queryClient
				.prefetchQuery({
					queryKey: buildAnalyticsCurrentTimeSlicesQueryKey(
						nextAnalyticsQueryUserId,
						nextFetchRequest,
						nextQueryRefreshTimestamp,
					),
					queryFn: () =>
						fetchAnalyticsData(nextFetchRequest, (request) =>
							client.labrinth.analytics_v3.fetch(request),
						),
					gcTime: ANALYTICS_PREFETCH_GC_TIME_MS,
				})
				.catch(() => {})
		},
		{ deep: true, immediate: true },
	)

	const analyticsFacetsRequest = computed<Labrinth.Analytics.v3.FetchRequest | null>(() => {
		const nextFetchRequest = fetchRequest.value
		if (!nextFetchRequest || sortedSelectedProjectIds.value.length === 0) {
			return null
		}

		return buildAnalyticsFacetsRequest(sortedSelectedProjectIds.value, nextFetchRequest.time_range)
	})

	const {
		data: analyticsFacetsData,
		isFetched: hasFetchedAnalyticsFilterOptions,
		isFetching: isAnalyticsFilterOptionsFetching,
	} = useQuery({
		queryKey: computed(() => [
			'analytics',
			'dashboard',
			analyticsQueryUserId.value,
			'filter-options',
			'facets',
			analyticsFacetsRequest.value,
			queryRefreshTimestamp.value,
		]),
		queryFn: () => {
			const nextRequest = analyticsFacetsRequest.value
			if (!isAnalyticsFetchRequestReady(nextRequest)) {
				return {
					facets: {
						project_views: {
							domain: [],
							site_path: [],
							monetized: [],
							country: [],
						},
						project_downloads: {
							project_id: [],
							domain: [],
							user_agent: [],
							version_id: [],
							monetized: [],
							country: [],
							reason: [],
							game_version: [],
							loader: [],
						},
						project_playtime: {
							version_id: [],
							loader: [],
							game_version: [],
							country: [],
						},
					},
				}
			}

			return client.labrinth.analytics_v3.fetchFacets(nextRequest)
		},
		enabled: computed(
			() =>
				hasCompletedAnalyticsLoading.value &&
				isAnalyticsFetchRequestReady(analyticsFacetsRequest.value),
		),
		gcTime: ANALYTICS_FILTER_OPTIONS_GC_TIME_MS,
		refetchOnWindowFocus: false,
	})

	const { data: analyticsDownloadCountTimeSlices } = useQuery({
		queryKey: computed(() => [
			'analytics',
			'dashboard',
			analyticsQueryUserId.value,
			'filter-options',
			'download-counts-fallback',
			analyticsFacetsRequest.value,
			queryRefreshTimestamp.value,
		]),
		queryFn: () => {
			const nextRequest = analyticsFacetsRequest.value
			if (!isAnalyticsFetchRequestReady(nextRequest)) {
				return []
			}

			return fetchAnalyticsTimeSlices(nextRequest, (request) =>
				client.labrinth.analytics_v3.fetch(request),
			)
		},
		enabled: computed(
			() =>
				hasFetchedAnalyticsFilterOptions.value &&
				isAnalyticsFetchRequestReady(analyticsFacetsRequest.value),
		),
		placeholderData: [],
		gcTime: ANALYTICS_FILTER_OPTIONS_GC_TIME_MS,
		refetchOnWindowFocus: false,
	})

	const analyticsFacetsFilterOptionSummary = computed(() =>
		getAnalyticsFacetsFilterOptionSummary(analyticsFacetsData.value?.facets),
	)
	const projectVersionFilterOptionSummary = computed(() =>
		getProjectVersionFilterOptionSummary(filterOptionProjectVersions.value ?? []),
	)
	const filterOptions = computed<AnalyticsDashboardFilterOptions>(() => ({
		countries: analyticsFacetsFilterOptionSummary.value.countries,
		downloadSources: analyticsFacetsFilterOptionSummary.value.downloadSources,
		downloadReasons: analyticsFacetsFilterOptionSummary.value.downloadReasons,
		gameVersions: sortStringValues([
			...new Set([
				...projectVersionFilterOptionSummary.value.gameVersions,
				...analyticsFacetsFilterOptionSummary.value.gameVersions,
			]),
		]),
		loaderTypes: sortStringValues([
			...new Set([
				...projectVersionFilterOptionSummary.value.loaderTypes,
				...analyticsFacetsFilterOptionSummary.value.loaderTypes,
			]),
		]),
		versionIds: sortStringValues([
			...new Set([
				...projectVersionFilterOptionSummary.value.versionIds,
				...analyticsFacetsFilterOptionSummary.value.versionIds,
			]),
		]),
	}))
	const isAnalyticsFilterOptionsLoading = computed(
		() => isAnalyticsFilterOptionsFetching.value && !hasFetchedAnalyticsFilterOptions.value,
	)

	watch(
		[
			selectedFilters,
			filterOptions,
			hasFetchedFilterOptionProjectVersions,
			hasFetchedAnalyticsFilterOptions,
		],
		([
			nextSelectedFilters,
			nextFilterOptions,
			hasFetchedVersionFilterOptions,
			hasFetchedAnalyticsOptions,
		]) => {
			if (!hasFetchedVersionFilterOptions || !hasFetchedAnalyticsOptions) {
				return
			}

			const sanitizedFilters = sanitizeAnalyticsSelectedFiltersForAvailableOptions(
				nextSelectedFilters,
				nextFilterOptions,
			)
			if (!areSelectedFiltersEqual(nextSelectedFilters, sanitizedFilters)) {
				selectedFilters.value = sanitizedFilters
			}
		},
		{ deep: true },
	)

	const timeSlices = shallowRef<Labrinth.Analytics.v3.TimeSlice[]>([])
	const previousTimeSlices = shallowRef<Labrinth.Analytics.v3.TimeSlice[]>([])
	const projectEvents = shallowRef<Labrinth.Analytics.v3.ProjectAnalyticsEvent[]>([])
	const displayedSelectedProjectIds = ref<string[]>([...selectedProjectIds.value])
	const displayedSelectedGroupBy = ref<AnalyticsGroupByPreset>(selectedGroupBy.value)
	const displayedSelectedBreakdowns = ref<AnalyticsSelectedBreakdowns>([
		...selectedBreakdowns.value,
	])
	const displayedSelectedFilters = ref<AnalyticsSelectedFilters>(
		cloneAnalyticsSelectedFilters(selectedFilters.value),
	)
	const displayedFetchRequest = ref<Labrinth.Analytics.v3.FetchRequest | null>(
		cloneAnalyticsFetchRequest(fetchRequest.value),
	)
	const displayedFilterOptions = ref<AnalyticsDashboardFilterOptions>(
		cloneAnalyticsFilterOptions(filterOptions.value),
	)
	const displayedTimeSlices = shallowRef<Labrinth.Analytics.v3.TimeSlice[]>([])
	const displayedPreviousTimeSlices = shallowRef<Labrinth.Analytics.v3.TimeSlice[]>([])
	const displayedProjectEvents = shallowRef<Labrinth.Analytics.v3.ProjectAnalyticsEvent[]>([])

	function commitDisplayedAnalyticsState() {
		displayedSelectedProjectIds.value = [...selectedProjectIds.value]
		displayedSelectedGroupBy.value = selectedGroupBy.value
		displayedSelectedBreakdowns.value = [...selectedBreakdowns.value]
		displayedSelectedFilters.value = cloneAnalyticsSelectedFilters(selectedFilters.value)
		displayedFetchRequest.value = cloneAnalyticsFetchRequest(fetchRequest.value)
		displayedFilterOptions.value = cloneAnalyticsFilterOptions(filterOptions.value)
		displayedTimeSlices.value = timeSlices.value
		displayedPreviousTimeSlices.value = previousTimeSlices.value
		displayedProjectEvents.value = projectEvents.value
	}

	watch(
		currentAnalyticsData,
		(nextAnalyticsData) => {
			if (nextAnalyticsData === undefined) {
				return
			}
			const splitTimeSlices = splitAnalyticsTimeSlices(
				nextAnalyticsData.metrics,
				fetchRequest.value,
				analyticsComparisonStartTime.value,
			)
			timeSlices.value = splitTimeSlices.currentTimeSlices
			previousTimeSlices.value = splitTimeSlices.previousTimeSlices
			projectEvents.value = getAnalyticsProjectEventsInTimeRange(
				nextAnalyticsData.project_events,
				fetchRequest.value,
			)
		},
		{ immediate: true },
	)

	watch(fetchRequest, (nextFetchRequest) => {
		if (isAnalyticsFetchRequestReady(nextFetchRequest)) {
			previousTimeSlices.value = []
			return
		}
		timeSlices.value = []
		previousTimeSlices.value = []
		projectEvents.value = []
	})

	const analyticsVersionIds = computed(() => {
		const versionIds = new Set<string>()
		for (const versionId of selectedFilters.value.version_id) {
			const normalizedVersionId = versionId.trim()
			if (normalizedVersionId.length > 0) {
				versionIds.add(normalizedVersionId)
			}
		}

		if (
			selectedBreakdowns.value.includes('version_id') ||
			selectedFilters.value.version_id.length > 0
		) {
			addVersionIdsFromTimeSlices(versionIds, timeSlices.value)
			addVersionIdsFromTimeSlices(versionIds, previousTimeSlices.value)
		}

		return sortStringValues([...versionIds])
	})

	const { data: versions } = useQuery({
		queryKey: computed(() => [
			'analytics',
			'dashboard',
			analyticsQueryUserId.value,
			'versions',
			analyticsVersionIds.value,
		]),
		queryFn: () =>
			fetchAnalyticsVersionMetadataByIds(analyticsVersionIds.value, (ids) =>
				client.labrinth.versions_v3.getVersions(ids),
			),
		enabled: computed(() => analyticsVersionIds.value.length > 0),
		placeholderData: [],
		gcTime: ANALYTICS_FILTER_OPTIONS_GC_TIME_MS,
		refetchOnWindowFocus: false,
	})

	const allVersionMetadata = computed(() => {
		const versionsById = new Map<string, AnalyticsVersionMetadata>()
		for (const version of filterOptionProjectVersions.value ?? []) {
			versionsById.set(version.id, version)
		}
		for (const version of versions.value ?? []) {
			versionsById.set(version.id, version)
		}
		return [...versionsById.values()]
	})

	const versionNumbersById = computed(
		() => new Map(allVersionMetadata.value.map((version) => [version.id, version.versionNumber])),
	)
	const versionPublishedDatesById = computed(
		() => new Map(allVersionMetadata.value.map((version) => [version.id, version.datePublished])),
	)
	const versionProjectNamesById = computed(() => {
		const projectNames = projectNamesById.value
		const versionProjectNames = new Map<string, string>()
		for (const version of allVersionMetadata.value) {
			const projectName = projectNames.get(version.projectId)
			if (projectName) {
				versionProjectNames.set(version.id, projectName)
			}
		}
		addVersionProjectNamesFromTimeSlices(versionProjectNames, timeSlices.value, projectNames)
		addVersionProjectNamesFromTimeSlices(
			versionProjectNames,
			previousTimeSlices.value,
			projectNames,
		)
		return versionProjectNames
	})
	const versionProjectIconUrlsById = computed(() => {
		const projectIconUrls = projectIconUrlsById.value
		const versionProjectIconUrls = new Map<string, string>()
		for (const version of allVersionMetadata.value) {
			const projectIconUrl = projectIconUrls.get(version.projectId)
			if (projectIconUrl) {
				versionProjectIconUrls.set(version.id, projectIconUrl)
			}
		}
		return versionProjectIconUrls
	})
	const downloadCountTimeSlices = computed(() => {
		const countTimeSlices = analyticsDownloadCountTimeSlices.value ?? []
		return countTimeSlices.length > 0 ? countTimeSlices : timeSlices.value
	})
	const projectDownloadsById = computed(() =>
		getProjectDownloadsByIdFromTimeSlices(downloadCountTimeSlices.value),
	)
	const projectVersionDownloadsById = computed(() =>
		getProjectVersionDownloadsByIdFromTimeSlices(downloadCountTimeSlices.value),
	)
	const countryDownloadsByCode = computed(() =>
		getCountryDownloadsByCodeFromTimeSlices(downloadCountTimeSlices.value),
	)
	const gameVersionDownloadsByVersion = computed(() =>
		getGameVersionDownloadsByVersionFromTimeSlices(downloadCountTimeSlices.value),
	)

	const selectedProjectIdSet = computed(() => new Set(selectedProjectIds.value))
	const availableProjectIdSet = computed(() => new Set(availableProjectIds.value))

	const currentTotals = computed<AnalyticsDashboardTotals>(() =>
		computeTotals(
			timeSlices.value,
			selectedProjectIdSet.value,
			availableProjectIdSet.value,
			projectStatusById.value,
			selectedFilters.value,
		),
	)
	const previousTotals = computed<AnalyticsDashboardTotals>(() =>
		computeTotals(
			previousTimeSlices.value,
			selectedProjectIdSet.value,
			availableProjectIdSet.value,
			projectStatusById.value,
			selectedFilters.value,
		),
	)

	const percentChanges = computed<AnalyticsDashboardPercentChanges>(() => ({
		views: getPercentChange(currentTotals.value.views, previousTotals.value.views),
		downloads: getPercentChange(currentTotals.value.downloads, previousTotals.value.downloads),
		revenue: getPercentChange(currentTotals.value.revenue, previousTotals.value.revenue),
		playtime: getPercentChange(currentTotals.value.playtime, previousTotals.value.playtime),
	}))

	const isLoading = computed(() => isCurrentTimeSliceLoading.value)
	const isRefetching = computed(() => currentFetching.value)
	watch(
		[
			isLoading,
			currentAnalyticsData,
			fetchRequest,
			selectedProjectIds,
			selectedGroupBy,
			selectedBreakdowns,
			selectedFilters,
			filterOptions,
		],
		() => {
			if (isLoading.value) {
				return
			}
			if (
				isAnalyticsFetchRequestReady(fetchRequest.value) &&
				currentAnalyticsData.value === undefined
			) {
				return
			}

			commitDisplayedAnalyticsState()
		},
		{ flush: 'post', immediate: true },
	)

	async function refreshAnalyticsQuery() {
		if (!isAnalyticsFetchRequestReady(fetchRequest.value)) {
			return
		}

		const fetchRequestKey = JSON.stringify(fetchRequest.value)
		const now = Date.now()
		queryRefreshTimestamp.value =
			now > queryRefreshTimestamp.value ? now : queryRefreshTimestamp.value + 1
		await nextTick()

		if (fetchRequest.value === null || JSON.stringify(fetchRequest.value) !== fetchRequestKey) {
			return
		}

		await refetchCurrentTimeSlices()
	}

	function resetAnalyticsQueryBuilder() {
		if (isAnalyticsQueryBuilderDefault.value) {
			return
		}

		const defaultQueryState = buildDefaultAnalyticsQueryBuilderState(availableProjectIds.value)
		const defaultGraphState = buildDefaultAnalyticsGraphState(defaultQueryState.selectedProjectIds)

		selectedProjectIds.value = defaultQueryState.selectedProjectIds
		selectedTimeframeMode.value = defaultQueryState.selectedTimeframeMode
		selectedTimeframe.value = defaultQueryState.selectedTimeframe
		selectedLastTimeframeAmount.value = defaultQueryState.selectedLastTimeframeAmount
		selectedLastTimeframeUnit.value = defaultQueryState.selectedLastTimeframeUnit
		selectedCustomTimeframeStartDate.value = defaultQueryState.selectedCustomTimeframeStartDate
		selectedCustomTimeframeEndDate.value = defaultQueryState.selectedCustomTimeframeEndDate
		selectedGroupBy.value = defaultQueryState.selectedGroupBy
		selectedBreakdowns.value = defaultQueryState.selectedBreakdowns
		selectedFilters.value = defaultQueryState.selectedFilters
		activeStat.value = defaultGraphState.activeStat
		activeGraphViewMode.value = defaultGraphState.activeGraphViewMode
		isRatioMode.value = defaultGraphState.isRatioMode
		showChartEvents.value = defaultGraphState.showChartEvents
		showProjectEvents.value = defaultGraphState.showProjectEvents
		showPreviousPeriod.value = defaultGraphState.showPreviousPeriod
		hiddenGraphDatasetIds.value = defaultGraphState.hiddenGraphDatasetIds
		hasExplicitGraphDatasetSelection.value = false
		isGraphDatasetSelectionActive.value = false
		selectedGraphDatasetIds.value = []
		queryResetToken.value += 1
	}

	function setFetchRequest(nextFetchRequest: Labrinth.Analytics.v3.FetchRequest) {
		hasInitializedAnalyticsFetchRequest.value = true

		if (areAnalyticsFetchRequestsEqual(fetchRequest.value, nextFetchRequest)) {
			return
		}

		fetchRequest.value = nextFetchRequest
	}

	function getVersionDisplayName(versionId: string): string {
		return versionNumbersById.value.get(versionId) ?? versionId
	}

	function getVersionPublishedDate(versionId: string): string | undefined {
		return versionPublishedDatesById.value.get(versionId)
	}

	function getVersionProjectName(versionId: string): string | undefined {
		return versionProjectNamesById.value.get(versionId)
	}

	function getVersionProjectIconUrl(versionId: string): string | undefined {
		return versionProjectIconUrlsById.value.get(versionId)
	}

	function setActiveStat(nextStat: AnalyticsDashboardStat) {
		if (
			!isAnalyticsDashboardStatRelevant(nextStat, selectedBreakdowns.value, selectedFilters.value)
		) {
			return
		}

		activeStat.value = nextStat
	}

	return {
		hasProjectContext,
		projectGroups,
		projects,
		selectedProjectIds,
		selectedTimeframeMode,
		selectedTimeframe,
		selectedLastTimeframeAmount,
		selectedLastTimeframeUnit,
		selectedCustomTimeframeStartDate,
		selectedCustomTimeframeEndDate,
		selectedGroupBy,
		analyticsAllTimeStartDate,
		selectedBreakdowns,
		selectedFilters,
		queryRefreshTimestamp,
		queryResetToken,
		isAnalyticsQueryBuilderDefault,
		fetchRequest,
		displayedSelectedProjectIds,
		displayedSelectedGroupBy,
		displayedSelectedBreakdowns,
		displayedSelectedFilters,
		displayedFetchRequest,
		displayedFilterOptions,
		filterOptions,
		isAnalyticsFilterOptionsLoading,
		versionNumbersById,
		versionPublishedDatesById,
		versionProjectNamesById,
		versionProjectIconUrlsById,
		projectStatusById,
		availableProjectStatuses,
		projectDownloadsById,
		projectVersionDownloadsById,
		gameVersionDownloadsByVersion,
		countryDownloadsByCode,
		timeSlices,
		displayedTimeSlices,
		previousTimeSlices,
		displayedPreviousTimeSlices,
		projectEvents,
		displayedProjectEvents,
		hasCompletedAnalyticsLoading,
		isLoading,
		isRefetching,
		activeStat,
		activeGraphViewMode,
		isRatioMode,
		showChartEvents,
		showProjectEvents,
		showPreviousPeriod,
		isMobileLayout,
		hiddenGraphDatasetIds,
		hasExplicitGraphDatasetSelection,
		isGraphDatasetSelectionActive,
		selectedGraphDatasetIds,
		defaultGraphDatasetIds,
		topGraphDatasetIds,
		currentTotals,
		previousTotals,
		percentChanges,
		hasPreviousPeriodComparison,
		getRelevantAnalyticsDashboardStats,
		isAnalyticsDashboardStatRelevant,
		refreshAnalyticsQuery,
		resetAnalyticsQueryBuilder,
		getVersionDisplayName,
		getVersionPublishedDate,
		getVersionProjectName,
		getVersionProjectIconUrl,
		setFetchRequest,
		setActiveStat,
	}
}
