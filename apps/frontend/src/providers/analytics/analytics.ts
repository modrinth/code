import type { Labrinth } from '@modrinth/api-client'
import { createContext, injectModrinthClient, type ProjectPageContext } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import type { ComputedRef, Ref } from 'vue'

import {
	getEnabledAnalyticsStatsForState,
	getProjectStatusFilterValue,
	PROJECT_STATUS_FILTER_VALUES,
	type ProjectStatusFilterValue,
	sanitizeAnalyticsSelectedFilters,
} from '~/components/analytics/query-builder/query-filter/queryFilter'
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
	buildDefaultAnalyticsQueryBuilderState,
	hasAnalyticsQueryBuilderRouteChange,
	isAnalyticsQueryBuilderStateDefault,
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
const ANALYTICS_START_TIMESTAMP = '2023-01-01T00:00:00.000Z'
const ANALYTICS_START_TIME = new Date(ANALYTICS_START_TIMESTAMP).getTime()

type ProjectTypeMetadata = {
	project_type?: string | null
	project_types?: readonly string[] | null
}

type AnalyticsProjectFetchRequest = Labrinth.Analytics.v3.FetchRequest & {
	project_ids: string[]
}

export interface AnalyticsDashboardProject {
	id: string
	name: string
	downloads: number
	status: ProjectStatusFilterValue
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
	downloadSources: string[]
	downloadReasons: string[]
	gameVersions: string[]
	loaderTypes: string[]
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
	queryResetToken: Ref<number>
	isAnalyticsQueryBuilderDefault: ComputedRef<boolean>
	fetchRequest: Ref<Labrinth.Analytics.v3.FetchRequest | null>
	displayedSelectedProjectIds: Ref<string[]>
	displayedSelectedGroupBy: Ref<AnalyticsGroupByPreset>
	displayedSelectedBreakdown: Ref<AnalyticsBreakdownPreset>
	displayedSelectedFilters: Ref<AnalyticsSelectedFilters>
	displayedFetchRequest: Ref<Labrinth.Analytics.v3.FetchRequest | null>
	displayedFilterOptions: Ref<AnalyticsDashboardFilterOptions>
	filterOptions: ComputedRef<AnalyticsDashboardFilterOptions>
	versionNumbersById: ComputedRef<Map<string, string>>
	versionPublishedDatesById: ComputedRef<Map<string, string>>
	projectStatusById: ComputedRef<Map<string, ProjectStatusFilterValue>>
	availableProjectStatuses: ComputedRef<ProjectStatusFilterValue[]>
	projectVersionDownloadsById: ComputedRef<Map<string, number>>
	gameVersionDownloadsByVersion: ComputedRef<Map<string, number>>
	countryDownloadsByCode: ComputedRef<Map<string, number>>
	timeSlices: Ref<Labrinth.Analytics.v3.TimeSlice[]>
	displayedTimeSlices: Ref<Labrinth.Analytics.v3.TimeSlice[]>
	previousTimeSlices: Ref<Labrinth.Analytics.v3.TimeSlice[]>
	isLoading: ComputedRef<boolean>
	isRefetching: ComputedRef<boolean>
	activeStat: Ref<AnalyticsDashboardStat>
	currentTotals: ComputedRef<AnalyticsDashboardTotals>
	previousTotals: ComputedRef<AnalyticsDashboardTotals>
	percentChanges: ComputedRef<AnalyticsDashboardPercentChanges>
	hasPreviousPeriodComparison: ComputedRef<boolean>
	getRelevantAnalyticsDashboardStats: (
		breakdown: AnalyticsBreakdownPreset,
		filters?: AnalyticsSelectedFilters,
	) => readonly AnalyticsDashboardStat[]
	isAnalyticsDashboardStatRelevant: (
		stat: AnalyticsDashboardStat,
		breakdown: AnalyticsBreakdownPreset,
		filters?: AnalyticsSelectedFilters,
	) => boolean
	refreshAnalyticsQuery: () => Promise<void>
	resetAnalyticsQueryBuilder: () => void
	getVersionDisplayName: (versionId: string) => string
	getVersionPublishedDate: (versionId: string) => string | undefined
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
	if (!isAnalyticsFetchRequestReady(fetchRequest)) {
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

	if (previousStart.getTime() < ANALYTICS_START_TIME) {
		return null
	}

	return {
		time_range: {
			start: previousStart.toISOString(),
			end: previousEnd.toISOString(),
			resolution: fetchRequest.time_range.resolution,
		},
		return_metrics: fetchRequest.return_metrics,
		project_ids: fetchRequest.project_ids,
	}
}

function isAnalyticsFetchRequestReady(
	fetchRequest: Labrinth.Analytics.v3.FetchRequest | null,
): fetchRequest is AnalyticsProjectFetchRequest {
	return Array.isArray(fetchRequest?.project_ids) && fetchRequest.project_ids.length > 0
}

function areAnalyticsFetchRequestsEqual(
	left: Labrinth.Analytics.v3.FetchRequest | null,
	right: Labrinth.Analytics.v3.FetchRequest,
): boolean {
	return JSON.stringify(left) === JSON.stringify(right)
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
	projectStatusById: Map<string, ProjectStatusFilterValue>,
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
	const filteredProjectIds = new Set(
		getProjectIdsMatchingStatusFilter([...effectiveProjectIds], projectStatusById, filters),
	)
	if (filteredProjectIds.size === 0) {
		return totals
	}

	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!('source_project' in dataPoint)) {
				continue
			}

			if (!filteredProjectIds.has(dataPoint.source_project)) {
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

function isAnalyticsEligibleProject(
	project: ProjectTypeMetadata & { status?: string | null },
): boolean {
	return !isServerProject(project)
	// && project.status !== 'draft'
}

export function doesProjectStatusMatchFilters(
	status: string | null | undefined,
	filters: AnalyticsSelectedFilters,
): boolean {
	if (filters.project_status.length === 0) {
		return true
	}

	return filters.project_status.includes(getProjectStatusFilterValue(status))
}

export function getProjectIdsMatchingStatusFilter(
	projectIds: string[],
	projectStatusById: Map<string, ProjectStatusFilterValue>,
	filters: AnalyticsSelectedFilters,
): string[] {
	if (filters.project_status.length === 0) {
		return projectIds
	}

	return projectIds.filter((projectId) =>
		doesProjectStatusMatchFilters(projectStatusById.get(projectId), filters),
	)
}

function sortStringValues(values: string[]): string[] {
	return [...values].sort((left, right) => left.localeCompare(right))
}

function getProjectVersionGameVersions(versions: Labrinth.Versions.v3.Version[]): string[] {
	const gameVersions = new Set<string>()

	for (const version of versions) {
		for (const gameVersion of version.game_versions) {
			const normalizedGameVersion = gameVersion.trim()
			if (normalizedGameVersion.length > 0) {
				gameVersions.add(normalizedGameVersion)
			}
		}
	}

	return sortStringValues([...gameVersions])
}

function getProjectVersionLoaders(versions: Labrinth.Versions.v3.Version[]): string[] {
	const loaders = new Set<string>()

	for (const version of versions) {
		const versionLoaders =
			version.mrpack_loaders && version.mrpack_loaders.length > 0
				? version.mrpack_loaders
				: version.loaders

		for (const loader of versionLoaders) {
			const normalizedLoader = loader.trim().toLowerCase()
			if (normalizedLoader.length > 0 && normalizedLoader !== 'mrpack') {
				loaders.add(normalizedLoader)
			}
		}
	}

	return sortStringValues([...loaders])
}

function retainAvailableSelectedFilterValues(
	values: string[],
	availableValues: string[],
): string[] {
	const availableValueSet = new Set(availableValues)
	return values.filter((value) => availableValueSet.has(value))
}

function sanitizeAnalyticsSelectedFiltersForAvailableOptions(
	filters: AnalyticsSelectedFilters,
	filterOptions: AnalyticsDashboardFilterOptions,
): AnalyticsSelectedFilters {
	return {
		...filters,
		download_reason: retainAvailableSelectedFilterValues(
			filters.download_reason,
			filterOptions.downloadReasons,
		),
		game_version: retainAvailableSelectedFilterValues(
			filters.game_version,
			filterOptions.gameVersions,
		),
		loader_type: retainAvailableSelectedFilterValues(
			filters.loader_type,
			filterOptions.loaderTypes,
		),
	}
}

function cloneAnalyticsSelectedFilters(
	filters: AnalyticsSelectedFilters,
): AnalyticsSelectedFilters {
	return {
		project: [...filters.project],
		project_status: [...filters.project_status],
		country: [...filters.country],
		monetization: [...filters.monetization],
		download_source: [...filters.download_source],
		download_reason: [...filters.download_reason],
		version_id: [...filters.version_id],
		game_version: [...filters.game_version],
		loader_type: [...filters.loader_type],
	}
}

function cloneAnalyticsFilterOptions(
	filterOptions: AnalyticsDashboardFilterOptions,
): AnalyticsDashboardFilterOptions {
	return {
		countries: [...filterOptions.countries],
		downloadSources: [...filterOptions.downloadSources],
		downloadReasons: [...filterOptions.downloadReasons],
		gameVersions: [...filterOptions.gameVersions],
		loaderTypes: [...filterOptions.loaderTypes],
		versionIds: [...filterOptions.versionIds],
	}
}

function cloneAnalyticsFetchRequest(
	fetchRequest: Labrinth.Analytics.v3.FetchRequest | null,
): Labrinth.Analytics.v3.FetchRequest | null {
	return fetchRequest ? JSON.parse(JSON.stringify(fetchRequest)) : null
}

function cloneAnalyticsTimeSlices(
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
): Labrinth.Analytics.v3.TimeSlice[] {
	return timeSlices.map((slice) => [...slice])
}

function getCountryFilterOptions(timeSlices: Labrinth.Analytics.v3.TimeSlice[]): string[] {
	const countries = new Set<string>()

	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!('source_project' in dataPoint)) {
				continue
			}

			if (
				(dataPoint.metric_kind === 'views' ||
					dataPoint.metric_kind === 'downloads' ||
					dataPoint.metric_kind === 'playtime') &&
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

function getCountryDownloadsByCode(
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
): Map<string, number> {
	const downloadsByCountry = new Map<string, number>()

	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!('source_project' in dataPoint)) {
				continue
			}

			if (dataPoint.metric_kind !== 'downloads' || !dataPoint.country) {
				continue
			}

			const country = dataPoint.country.trim().toUpperCase()
			if (country.length === 0) {
				continue
			}

			downloadsByCountry.set(country, (downloadsByCountry.get(country) ?? 0) + dataPoint.downloads)
		}
	}

	return downloadsByCountry
}

function getDownloadSourceFilterOptions(timeSlices: Labrinth.Analytics.v3.TimeSlice[]): string[] {
	const downloadSources = new Set<string>()

	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!('source_project' in dataPoint)) {
				continue
			}

			if (dataPoint.metric_kind === 'downloads' && dataPoint.domain) {
				const downloadSource = dataPoint.domain.trim()
				if (downloadSource.length > 0) {
					downloadSources.add(downloadSource)
				}
			}
		}
	}

	return sortStringValues([...downloadSources])
}

function getDownloadReasonFilterOptions(timeSlices: Labrinth.Analytics.v3.TimeSlice[]): string[] {
	const downloadReasons = new Set<string>()

	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!('source_project' in dataPoint)) {
				continue
			}

			if (dataPoint.metric_kind === 'downloads' && dataPoint.reason) {
				downloadReasons.add(dataPoint.reason)
			}
		}
	}

	return sortStringValues([...downloadReasons])
}

function getAnalyticsGameVersionFilterOptions(
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
): string[] {
	const gameVersions = new Set<string>()

	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!('source_project' in dataPoint)) {
				continue
			}

			if (
				(dataPoint.metric_kind === 'downloads' || dataPoint.metric_kind === 'playtime') &&
				dataPoint.game_version
			) {
				const gameVersion = dataPoint.game_version.trim()
				if (gameVersion.length > 0) {
					gameVersions.add(gameVersion)
				}
			}
		}
	}

	return sortStringValues([...gameVersions])
}

function getAnalyticsLoaderFilterOptions(timeSlices: Labrinth.Analytics.v3.TimeSlice[]): string[] {
	const loaders = new Set<string>()

	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!('source_project' in dataPoint)) {
				continue
			}

			if (
				(dataPoint.metric_kind === 'downloads' || dataPoint.metric_kind === 'playtime') &&
				dataPoint.loader
			) {
				const loader = dataPoint.loader.trim().toLowerCase()
				if (loader.length > 0 && loader !== 'mrpack') {
					loaders.add(loader)
				}
			}
		}
	}

	return sortStringValues([...loaders])
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
		doesAnalyticsPointMatchFilter(
			dataPoint,
			filters.download_reason,
			getDownloadReasonFilterValue,
		) &&
		doesAnalyticsPointMatchFilter(dataPoint, filters.version_id, getVersionFilterValue) &&
		doesAnalyticsPointMatchFilter(dataPoint, filters.game_version, getGameVersionFilterValue) &&
		doesAnalyticsPointMatchFilter(dataPoint, filters.loader_type, getLoaderFilterValue)
	)
}

function doesAnalyticsPointMatchFilter(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
	filterValues: string[],
	getPointValue: (dataPoint: Labrinth.Analytics.v3.ProjectAnalytics) => string | null | undefined,
): boolean {
	if (filterValues.length === 0) {
		return true
	}

	const pointValue = getPointValue(dataPoint)
	if (pointValue === undefined) {
		return true
	}
	if (pointValue === null) {
		return false
	}

	const normalizedPointValue = pointValue.trim().toLowerCase()
	return filterValues.some((value) => value.trim().toLowerCase() === normalizedPointValue)
}

function getCountryFilterValue(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
): string | null | undefined {
	if (
		dataPoint.metric_kind !== 'views' &&
		dataPoint.metric_kind !== 'downloads' &&
		dataPoint.metric_kind !== 'playtime'
	) {
		return undefined
	}

	return dataPoint.country ?? null
}

function getMonetizationFilterValue(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
): string | null | undefined {
	if (dataPoint.metric_kind !== 'views' && dataPoint.metric_kind !== 'downloads') {
		return undefined
	}
	if (typeof dataPoint.monetized !== 'boolean') {
		return null
	}

	return dataPoint.monetized ? 'monetized' : 'unmonetized'
}

function getDownloadSourceFilterValue(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
): string | null | undefined {
	if (dataPoint.metric_kind !== 'downloads') {
		return undefined
	}

	return dataPoint.domain ?? null
}

function getDownloadReasonFilterValue(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
): string | null | undefined {
	if (dataPoint.metric_kind !== 'downloads') {
		return undefined
	}

	return dataPoint.reason ?? null
}

function getVersionFilterValue(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
): string | null | undefined {
	if (dataPoint.metric_kind !== 'downloads' && dataPoint.metric_kind !== 'playtime') {
		return undefined
	}

	return dataPoint.version_id ?? null
}

function getGameVersionFilterValue(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
): string | null | undefined {
	if (dataPoint.metric_kind !== 'downloads' && dataPoint.metric_kind !== 'playtime') {
		return undefined
	}

	return dataPoint.game_version ?? null
}

function getLoaderFilterValue(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
): string | null | undefined {
	if (dataPoint.metric_kind !== 'downloads' && dataPoint.metric_kind !== 'playtime') {
		return undefined
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
	const queryResetToken = ref(0)
	const fetchRequest = ref<Labrinth.Analytics.v3.FetchRequest | null>(null)

	const hasProjectContext = computed(() => Boolean(options.projectPageContext))
	const hasOrganizationContext = computed(
		() => !hasProjectContext.value && Boolean(options.organizationContext),
	)

	const { data: userProjects, isFetched: hasFetchedUserProjects } = useQuery({
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

	const areProjectsLoaded = computed(() => {
		if (hasProjectContext.value) {
			return true
		}

		if (hasOrganizationContext.value) {
			return options.organizationContext?.projects.value !== null
		}

		return hasFetchedUserProjects.value
	})

	const projects = computed<AnalyticsDashboardProject[]>(() => {
		if (hasProjectContext.value && options.projectPageContext) {
			const project = options.projectPageContext.projectV2.value
			return project && isAnalyticsEligibleProject(project)
				? [
						{
							id: project.id,
							name: project.title,
							downloads: project.downloads ?? 0,
							status: getProjectStatusFilterValue(project.status),
						},
					]
				: []
		}

		if (hasOrganizationContext.value && options.organizationContext?.projects.value) {
			return options.organizationContext.projects.value
				.filter((project) => isAnalyticsEligibleProject(project))
				.map((project) => ({
					id: project.id,
					name: project.name,
					downloads: project.downloads ?? 0,
					status: getProjectStatusFilterValue(project.status),
				}))
		}

		return (userProjects.value ?? [])
			.filter((project) => isAnalyticsEligibleProject(project))
			.map((project) => ({
				id: project.id,
				name: project.title,
				downloads: project.downloads ?? 0,
				status: getProjectStatusFilterValue(project.status),
			}))
	})

	const availableProjectIds = computed(() => projects.value.map((project) => project.id))
	const projectStatusById = computed(
		() => new Map(projects.value.map((project) => [project.id, project.status])),
	)
	const availableProjectStatuses = computed<ProjectStatusFilterValue[]>(() => {
		const presentStatuses = new Set(projects.value.map((project) => project.status))
		return PROJECT_STATUS_FILTER_VALUES.filter((status) => presentStatuses.has(status))
	})
	const sortedSelectedProjectIds = computed(() => sortStringValues(selectedProjectIds.value))
	const isAnalyticsQueryBuilderDefault = computed(() =>
		isAnalyticsQueryBuilderStateDefault(
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
		),
	)

	function getRelevantAnalyticsDashboardStats(
		breakdown: AnalyticsBreakdownPreset,
		filters: AnalyticsSelectedFilters = selectedFilters.value,
	): readonly AnalyticsDashboardStat[] {
		return getEnabledAnalyticsStatsForState(breakdown, filters)
	}

	function isAnalyticsDashboardStatRelevant(
		stat: AnalyticsDashboardStat,
		breakdown: AnalyticsBreakdownPreset,
		filters: AnalyticsSelectedFilters = selectedFilters.value,
	): boolean {
		return getRelevantAnalyticsDashboardStats(breakdown, filters).includes(stat)
	}

	watch(
		[selectedBreakdown, selectedFilters, activeStat],
		([nextBreakdown, nextFilters, nextActiveStat]) => {
			if (isAnalyticsDashboardStatRelevant(nextActiveStat, nextBreakdown, nextFilters)) {
				return
			}

			const fallbackStat = getRelevantAnalyticsDashboardStats(nextBreakdown, nextFilters)[0]
			if (fallbackStat && fallbackStat !== nextActiveStat) {
				activeStat.value = fallbackStat
			}
		},
		{ deep: true, immediate: true },
	)

	watch(
		[selectedBreakdown, selectedFilters],
		([nextBreakdown, nextFilters]) => {
			const sanitizedFilters = sanitizeAnalyticsSelectedFilters(nextBreakdown, nextFilters)
			if (!areSelectedFiltersEqual(nextFilters, sanitizedFilters)) {
				selectedFilters.value = sanitizedFilters
			}
		},
		{ deep: true, immediate: true },
	)

	watch(
		[projects, areProjectsLoaded],
		([nextProjects, nextAreProjectsLoaded]) => {
			if (nextProjects.length === 0) {
				if (nextAreProjectsLoaded && selectedProjectIds.value.length > 0) {
					selectedProjectIds.value = []
				}
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
			const availableProjectIdSet = new Set(availableProjectIds.value)
			const nextSelectedProjectIds = nextQueryState.selectedProjectIds.filter((projectId) =>
				availableProjectIdSet.has(projectId),
			)
			const nextSelectedFilters = sanitizeAnalyticsSelectedFilters(
				nextQueryState.selectedBreakdown,
				nextQueryState.selectedFilters,
			)

			if (!areStringArraysEqual(selectedProjectIds.value, nextSelectedProjectIds)) {
				selectedProjectIds.value = nextSelectedProjectIds
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
			if (!areSelectedFiltersEqual(selectedFilters.value, nextSelectedFilters)) {
				selectedFilters.value = nextSelectedFilters
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
		queryFn: async () => {
			const response = await client.labrinth.analytics_v3.fetch(
				fetchRequest.value as Labrinth.Analytics.v3.FetchRequest,
			)
			return response.metrics
		},
		enabled: computed(() => isAnalyticsFetchRequestReady(fetchRequest.value)),
	})
	const isCurrentTimeSliceLoading = computed(
		() => isAnalyticsFetchRequestReady(fetchRequest.value) && currentTimeSlicePending.value,
	)

	const analyticsFilterOptionsRequest = computed<Labrinth.Analytics.v3.FetchRequest | null>(() => {
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
					bucket_by: ['country', 'domain', 'reason', 'game_version', 'loader'],
				},
				project_playtime: {
					bucket_by: ['country', 'game_version', 'loader'],
				},
			},
		}
	})

	const { data: analyticsFilterOptionsData, isFetched: hasFetchedAnalyticsFilterOptions } =
		useQuery({
			queryKey: computed(() => [
				'analytics',
				'dashboard',
				'filter-options',
				'analytics-fields',
				analyticsFilterOptionsRequest.value,
			]),
			queryFn: async () => {
				const response = await client.labrinth.analytics_v3.fetch(
					analyticsFilterOptionsRequest.value as Labrinth.Analytics.v3.FetchRequest,
				)
				return response.metrics
			},
			enabled: computed(() => analyticsFilterOptionsRequest.value !== null),
		})

	const { data: filterOptionProjectVersions, isFetched: hasFetchedFilterOptionProjectVersions } =
		useQuery({
			queryKey: computed(() => [
				'analytics',
				'dashboard',
				'filter-options',
				'versions',
				sortedSelectedProjectIds.value,
			]),
			queryFn: async () => {
				const projectVersions = await Promise.all(
					sortedSelectedProjectIds.value.map((projectId) =>
						client.labrinth.versions_v3.getProjectVersions(projectId, {
							include_changelog: false,
							apiVersion: 3,
						}),
					),
				)

				return projectVersions.flat()
			},
			enabled: computed(() => sortedSelectedProjectIds.value.length > 0),
			placeholderData: [],
		})

	const filterOptions = computed<AnalyticsDashboardFilterOptions>(() => ({
		countries: getCountryFilterOptions(analyticsFilterOptionsData.value ?? []),
		downloadSources: getDownloadSourceFilterOptions(analyticsFilterOptionsData.value ?? []),
		downloadReasons: getDownloadReasonFilterOptions(analyticsFilterOptionsData.value ?? []),
		gameVersions: sortStringValues([
			...new Set([
				...getProjectVersionGameVersions(filterOptionProjectVersions.value ?? []),
				...getAnalyticsGameVersionFilterOptions(analyticsFilterOptionsData.value ?? []),
			]),
		]),
		loaderTypes: sortStringValues([
			...new Set([
				...getProjectVersionLoaders(filterOptionProjectVersions.value ?? []),
				...getAnalyticsLoaderFilterOptions(analyticsFilterOptionsData.value ?? []),
			]),
		]),
		versionIds: sortStringValues([
			...new Set((filterOptionProjectVersions.value ?? []).map((version) => version.id)),
		]),
	}))

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

	const filterOptionProjectVersionIds = computed(
		() => new Set((filterOptionProjectVersions.value ?? []).map((version) => version.id)),
	)

	const previousFetchRequest = computed(() => buildPreviousFetchRequest(fetchRequest.value))
	const hasPreviousPeriodComparison = computed(() => previousFetchRequest.value !== null)

	const {
		data: previousTimeSliceData,
		isFetching: previousFetching,
		refetch: refetchPreviousTimeSlices,
	} = useQuery({
		queryKey: computed(() => ['analytics', 'dashboard', 'previous', previousFetchRequest.value]),
		queryFn: async () => {
			const response = await client.labrinth.analytics_v3.fetch(
				previousFetchRequest.value as Labrinth.Analytics.v3.FetchRequest,
			)
			return response.metrics
		},
		enabled: computed(() => previousFetchRequest.value !== null),
	})

	const timeSlices = ref<Labrinth.Analytics.v3.TimeSlice[]>([])
	const previousTimeSlices = ref<Labrinth.Analytics.v3.TimeSlice[]>([])
	const displayedSelectedProjectIds = ref<string[]>([...selectedProjectIds.value])
	const displayedSelectedGroupBy = ref<AnalyticsGroupByPreset>(selectedGroupBy.value)
	const displayedSelectedBreakdown = ref<AnalyticsBreakdownPreset>(selectedBreakdown.value)
	const displayedSelectedFilters = ref<AnalyticsSelectedFilters>(
		cloneAnalyticsSelectedFilters(selectedFilters.value),
	)
	const displayedFetchRequest = ref<Labrinth.Analytics.v3.FetchRequest | null>(
		cloneAnalyticsFetchRequest(fetchRequest.value),
	)
	const displayedFilterOptions = ref<AnalyticsDashboardFilterOptions>(
		cloneAnalyticsFilterOptions(filterOptions.value),
	)
	const displayedTimeSlices = ref<Labrinth.Analytics.v3.TimeSlice[]>([])

	function commitDisplayedAnalyticsState() {
		displayedSelectedProjectIds.value = [...selectedProjectIds.value]
		displayedSelectedGroupBy.value = selectedGroupBy.value
		displayedSelectedBreakdown.value = selectedBreakdown.value
		displayedSelectedFilters.value = cloneAnalyticsSelectedFilters(selectedFilters.value)
		displayedFetchRequest.value = cloneAnalyticsFetchRequest(fetchRequest.value)
		displayedFilterOptions.value = cloneAnalyticsFilterOptions(filterOptions.value)
		displayedTimeSlices.value = cloneAnalyticsTimeSlices(timeSlices.value)
	}

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
		if (isAnalyticsFetchRequestReady(nextFetchRequest)) {
			previousTimeSlices.value = []
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

		addVersionIdsFromTimeSlices(versionIds, timeSlices.value)
		addVersionIdsFromTimeSlices(versionIds, previousTimeSlices.value)

		for (const versionId of filterOptionProjectVersionIds.value) {
			versionIds.delete(versionId)
		}

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

	const allVersionMetadata = computed(() => {
		const versionsById = new Map<string, Labrinth.Versions.v3.Version>()
		for (const version of filterOptionProjectVersions.value ?? []) {
			versionsById.set(version.id, version)
		}
		for (const version of versions.value ?? []) {
			versionsById.set(version.id, version)
		}
		return [...versionsById.values()]
	})

	const versionNumbersById = computed(
		() => new Map(allVersionMetadata.value.map((version) => [version.id, version.version_number])),
	)
	const versionPublishedDatesById = computed(
		() => new Map(allVersionMetadata.value.map((version) => [version.id, version.date_published])),
	)
	const projectVersionDownloadsById = computed(
		() => new Map(allVersionMetadata.value.map((version) => [version.id, version.downloads])),
	)
	const countryDownloadsByCode = computed(() =>
		getCountryDownloadsByCode(analyticsFilterOptionsData.value ?? []),
	)
	const gameVersionDownloadsByVersion = computed(() => {
		const downloadsByVersion = new Map<string, number>()

		for (const version of allVersionMetadata.value) {
			for (const gameVersion of version.game_versions) {
				const normalizedGameVersion = gameVersion.trim()
				if (normalizedGameVersion.length === 0) {
					continue
				}

				downloadsByVersion.set(
					normalizedGameVersion,
					(downloadsByVersion.get(normalizedGameVersion) ?? 0) + version.downloads,
				)
			}
		}

		return downloadsByVersion
	})

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
	const isRefetching = computed(() => currentFetching.value || previousFetching.value)
	watch(
		[
			isLoading,
			currentTimeSliceData,
			fetchRequest,
			selectedProjectIds,
			selectedGroupBy,
			selectedBreakdown,
			selectedFilters,
			filterOptions,
		],
		() => {
			if (isLoading.value) {
				return
			}
			if (
				isAnalyticsFetchRequestReady(fetchRequest.value) &&
				currentTimeSliceData.value === undefined
			) {
				return
			}

			commitDisplayedAnalyticsState()
		},
		{ deep: true, flush: 'post', immediate: true },
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

		const refetches = [refetchCurrentTimeSlices()]
		if (previousFetchRequest.value !== null) {
			refetches.push(refetchPreviousTimeSlices())
		}

		await Promise.all(refetches)
	}

	function resetAnalyticsQueryBuilder() {
		if (isAnalyticsQueryBuilderDefault.value) {
			return
		}

		const defaultQueryState = buildDefaultAnalyticsQueryBuilderState(availableProjectIds.value)

		selectedProjectIds.value = defaultQueryState.selectedProjectIds
		selectedTimeframeMode.value = defaultQueryState.selectedTimeframeMode
		selectedTimeframe.value = defaultQueryState.selectedTimeframe
		selectedLastTimeframeAmount.value = defaultQueryState.selectedLastTimeframeAmount
		selectedLastTimeframeUnit.value = defaultQueryState.selectedLastTimeframeUnit
		selectedCustomTimeframeStartDate.value = defaultQueryState.selectedCustomTimeframeStartDate
		selectedCustomTimeframeEndDate.value = defaultQueryState.selectedCustomTimeframeEndDate
		selectedGroupBy.value = defaultQueryState.selectedGroupBy
		selectedBreakdown.value = defaultQueryState.selectedBreakdown
		selectedFilters.value = defaultQueryState.selectedFilters
		queryResetToken.value += 1
	}

	function setFetchRequest(nextFetchRequest: Labrinth.Analytics.v3.FetchRequest) {
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

	function setActiveStat(nextStat: AnalyticsDashboardStat) {
		if (
			!isAnalyticsDashboardStatRelevant(nextStat, selectedBreakdown.value, selectedFilters.value)
		) {
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
		queryResetToken,
		isAnalyticsQueryBuilderDefault,
		fetchRequest,
		displayedSelectedProjectIds,
		displayedSelectedGroupBy,
		displayedSelectedBreakdown,
		displayedSelectedFilters,
		displayedFetchRequest,
		displayedFilterOptions,
		filterOptions,
		versionNumbersById,
		versionPublishedDatesById,
		projectStatusById,
		availableProjectStatuses,
		projectVersionDownloadsById,
		gameVersionDownloadsByVersion,
		countryDownloadsByCode,
		timeSlices,
		displayedTimeSlices,
		previousTimeSlices,
		isLoading,
		isRefetching,
		activeStat,
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
		setFetchRequest,
		setActiveStat,
	}
}
