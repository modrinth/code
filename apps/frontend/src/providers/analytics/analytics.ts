import type { Labrinth } from '@modrinth/api-client'
import { createContext, injectModrinthClient, type ProjectPageContext } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
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
	type AnalyticsDashboardStat,
	type AnalyticsGraphViewMode,
	type AnalyticsGroupByPreset,
	type AnalyticsLastTimeframeUnit,
	type AnalyticsSelectedFilters,
	type AnalyticsTimeframeMode,
	type AnalyticsTimeframePreset,
	areSelectedFiltersEqual,
	areStringArraysEqual,
	buildAnalyticsQueryBuilderRouteQuery,
	buildDefaultAnalyticsGraphState,
	buildDefaultAnalyticsQueryBuilderState,
	getAnalyticsBreakdownPresetForProjectSelection,
	getDefaultAnalyticsBreakdownPreset,
	hasAnalyticsBreakdownQuery,
	hasAnalyticsProjectSelectionQuery,
	hasAnalyticsQueryBuilderRouteChange,
	isAnalyticsGraphStateDefault,
	isAnalyticsQueryBuilderStateDefault,
	readAnalyticsGraphState,
	readAnalyticsQueryBuilderState,
} from './query-builder-url'

export type {
	AnalyticsBreakdownPreset,
	AnalyticsDashboardStat,
	AnalyticsGraphState,
	AnalyticsGraphViewMode,
	AnalyticsGroupByPreset,
	AnalyticsLastTimeframeUnit,
	AnalyticsQueryFilterCategory,
	AnalyticsSelectedFilters,
	AnalyticsTimeframeMode,
	AnalyticsTimeframePreset,
} from './query-builder-url'

const MINECRAFT_JAVA_SERVER_PROJECT_TYPE = 'minecraft_java_server'
const ANALYTICS_START_TIMESTAMP = '2023-01-01T00:00:00.000Z'
export const ANALYTICS_START_DATE_INPUT_VALUE = ANALYTICS_START_TIMESTAMP.slice(0, 10)
const ANALYTICS_START_TIME = new Date(ANALYTICS_START_TIMESTAMP).getTime()
const REVENUE_GROUP_BY_FALLBACK: AnalyticsGroupByPreset = 'day'
const REVENUE_MIN_TIMEFRAME_MS = 1 * 24 * 60 * 60 * 1000 // need at least 1 day in timeframe range to show revenue
const ANALYTICS_DAY_MS = 24 * 60 * 60 * 1000
const ANALYTICS_MAX_TIME_SLICES = 256 // controls granularity allowed in "group by" for timeframe ranges
const ANALYTICS_TIME_SLICES_GC_TIME_MS = 30 * 1000
const ANALYTICS_PREFETCH_GC_TIME_MS = 15 * 1000
const ANALYTICS_FILTER_OPTIONS_GC_TIME_MS = 60 * 1000
const ANALYTICS_PROJECT_IDS_FETCH_BATCH_SIZE = 20
const ANALYTICS_PROJECT_IDS_FETCH_BATCH_DELAY_MS = 300

type ProjectTypeMetadata = {
	project_type?: string | null
	project_types?: readonly string[] | null
}

type AnalyticsProjectFetchRequest = Labrinth.Analytics.v3.FetchRequest & {
	project_ids: string[]
}

type AnalyticsDashboardProjectSource = ProjectTypeMetadata & {
	id: string
	name?: string | null
	title?: string | null
	icon_url?: string | null
	downloads?: number | null
	status?: string | null
}

type AnalyticsDashboardUserProjectSource = AnalyticsDashboardProjectSource & {
	organization?: string | null
}

export interface AnalyticsDashboardProject {
	id: string
	name: string
	iconUrl?: string
	downloads: number
	status: ProjectStatusFilterValue
}

export interface AnalyticsDashboardProjectGroup {
	key?: string
	title?: string
	projects: AnalyticsDashboardProject[]
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

export interface NormalizedAnalyticsSelectedFilters {
	country: ReadonlySet<string>
	monetization: ReadonlySet<string>
	userAgent: ReadonlySet<string>
	downloadReason: ReadonlySet<string>
	versionId: ReadonlySet<string>
	gameVersion: ReadonlySet<string>
	loaderType: ReadonlySet<string>
}

interface AnalyticsDataFilterOptionSummary {
	countries: string[]
	downloadSources: string[]
	downloadReasons: string[]
	gameVersions: string[]
	loaderTypes: string[]
	versionIds: string[]
	countryDownloadsByCode: Map<string, number>
}

interface ProjectVersionFilterOptionSummary {
	gameVersions: string[]
	loaderTypes: string[]
	versionIds: string[]
}

interface AnalyticsVersionMetadata {
	id: string
	versionNumber: string
	datePublished: string
	projectId: string
	downloads: number
	gameVersions: string[]
	loaders: string[]
}

function isProjectAnalyticsPoint(
	dataPoint: Labrinth.Analytics.v3.AnalyticsData,
): dataPoint is Labrinth.Analytics.v3.ProjectAnalytics {
	return 'source_project' in dataPoint
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
	isAnalyticsFilterOptionsLoading: ComputedRef<boolean>
	versionNumbersById: ComputedRef<Map<string, string>>
	versionPublishedDatesById: ComputedRef<Map<string, string>>
	versionProjectNamesById: ComputedRef<Map<string, string>>
	versionProjectIconUrlsById: ComputedRef<Map<string, string>>
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
	activeGraphViewMode: Ref<AnalyticsGraphViewMode>
	isRatioMode: Ref<boolean>
	showChartEvents: Ref<boolean>
	hiddenGraphDatasetIds: Ref<string[]>
	isGraphDatasetSelectionActive: Ref<boolean>
	selectedGraphDatasetIds: Ref<string[]>
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

type AnalyticsQueryBuilderRouteNavigationMode = 'push' | 'replace'

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

function buildAnalyticsFetchRequestBatches(
	fetchRequest: AnalyticsProjectFetchRequest,
): AnalyticsProjectFetchRequest[] {
	if (fetchRequest.project_ids.length <= ANALYTICS_PROJECT_IDS_FETCH_BATCH_SIZE) {
		return [fetchRequest]
	}

	const requests: AnalyticsProjectFetchRequest[] = []
	for (
		let index = 0;
		index < fetchRequest.project_ids.length;
		index += ANALYTICS_PROJECT_IDS_FETCH_BATCH_SIZE
	) {
		requests.push({
			...fetchRequest,
			project_ids: fetchRequest.project_ids.slice(
				index,
				index + ANALYTICS_PROJECT_IDS_FETCH_BATCH_SIZE,
			),
		})
	}

	return requests
}

function mergeAnalyticsTimeSlices(
	timeSliceGroups: Labrinth.Analytics.v3.TimeSlice[][],
): Labrinth.Analytics.v3.TimeSlice[] {
	const mergedTimeSlices: Labrinth.Analytics.v3.TimeSlice[] = []

	for (const timeSlices of timeSliceGroups) {
		timeSlices.forEach((timeSlice, index) => {
			if (!mergedTimeSlices[index]) {
				mergedTimeSlices[index] = []
			}

			for (const dataPoint of timeSlice) {
				mergedTimeSlices[index].push(dataPoint)
			}
		})
	}

	return mergedTimeSlices
}

function waitForAnalyticsFetchBatchDelay(): Promise<void> {
	return new Promise((resolve) => setTimeout(resolve, ANALYTICS_PROJECT_IDS_FETCH_BATCH_DELAY_MS))
}

async function fetchAnalyticsTimeSlices(
	fetchRequest: AnalyticsProjectFetchRequest,
	fetchAnalytics: (
		request: Labrinth.Analytics.v3.FetchRequest,
	) => Promise<Labrinth.Analytics.v3.FetchResponse>,
): Promise<Labrinth.Analytics.v3.TimeSlice[]> {
	const fetchRequests = buildAnalyticsFetchRequestBatches(fetchRequest)
	const timeSliceGroups: Labrinth.Analytics.v3.TimeSlice[][] = []

	for (let index = 0; index < fetchRequests.length; index++) {
		if (index > 0) {
			await waitForAnalyticsFetchBatchDelay()
		}

		const response = await fetchAnalytics(fetchRequests[index])
		timeSliceGroups.push(response.metrics)
	}

	return mergeAnalyticsTimeSlices(timeSliceGroups)
}

function areAnalyticsFetchRequestsEqual(
	left: Labrinth.Analytics.v3.FetchRequest | null,
	right: Labrinth.Analytics.v3.FetchRequest,
): boolean {
	return JSON.stringify(left) === JSON.stringify(right)
}

function buildAnalyticsCurrentTimeSlicesQueryKey(
	userId: string | undefined,
	nextFetchRequest: Labrinth.Analytics.v3.FetchRequest | null,
) {
	return ['analytics', 'dashboard', userId, 'current', nextFetchRequest]
}

function isRevenueHourlyGroupBy(groupBy: AnalyticsGroupByPreset): boolean {
	return groupBy === '1h' || groupBy === '6h'
}

function buildDailyAnalyticsFetchRequest(
	nextFetchRequest: Labrinth.Analytics.v3.FetchRequest | null,
): Labrinth.Analytics.v3.FetchRequest | null {
	if (!isAnalyticsFetchRequestReady(nextFetchRequest)) {
		return null
	}

	const startTime = new Date(nextFetchRequest.time_range.start).getTime()
	const endTime = new Date(nextFetchRequest.time_range.end).getTime()
	const durationMs = endTime - startTime
	if (!Number.isFinite(durationMs) || durationMs <= 0) {
		return null
	}

	const desiredSlices = Math.max(1, Math.floor(durationMs / ANALYTICS_DAY_MS))

	return {
		...nextFetchRequest,
		time_range: {
			...nextFetchRequest.time_range,
			resolution: {
				slices: Math.min(ANALYTICS_MAX_TIME_SLICES, desiredSlices),
			},
		},
	}
}

function buildAnalyticsFilterOptionsRequests(
	projectIds: string[],
	endTimestamp: string,
): Labrinth.Analytics.v3.FetchRequest[] {
	const buildRequest = (
		returnMetrics: Labrinth.Analytics.v3.ReturnMetrics,
	): Labrinth.Analytics.v3.FetchRequest => ({
		time_range: {
			start: ANALYTICS_START_TIMESTAMP,
			end: endTimestamp,
			resolution: {
				slices: 1,
			},
		},
		project_ids: projectIds,
		return_metrics: returnMetrics,
	})

	return [
		buildRequest({
			project_views: {
				bucket_by: ['country'],
			},
		}),
		buildRequest({
			project_downloads: {
				bucket_by: ['country'],
			},
		}),
		buildRequest({
			project_playtime: {
				bucket_by: ['country'],
			},
		}),
		buildRequest({
			project_downloads: {
				bucket_by: ['user_agent'],
			},
		}),
		buildRequest({
			project_downloads: {
				bucket_by: ['reason'],
			},
		}),
	]
}

function addAnalyticsDays(date: Date, days: number): Date {
	const nextDate = new Date(date)
	nextDate.setDate(nextDate.getDate() + days)
	return nextDate
}

function parseAnalyticsDateInputValue(value: string): Date | null {
	const parsedDate = new Date(`${value}T00:00:00`)
	return Number.isNaN(parsedDate.getTime()) ? null : parsedDate
}

function parseAnalyticsDateTimeInputValue(value: string): Date | null {
	const parsedDate = new Date(value)
	return Number.isNaN(parsedDate.getTime()) ? null : parsedDate
}

function getAnalyticsTimeframeDurationMs({
	mode,
	preset,
	lastAmount,
	lastUnit,
	customStartDate,
	customEndDate,
	nowTimestamp,
}: {
	mode: AnalyticsTimeframeMode
	preset: AnalyticsTimeframePreset
	lastAmount: number
	lastUnit: AnalyticsLastTimeframeUnit
	customStartDate: string
	customEndDate: string
	nowTimestamp: number
}): number {
	if (mode === 'preset') {
		switch (preset) {
			case 'today':
			case 'yesterday':
				return 24 * 60 * 60 * 1000
			case 'last_7_days':
				return 7 * 24 * 60 * 60 * 1000
			case 'last_14_days':
				return 14 * 24 * 60 * 60 * 1000
			case 'last_30_days':
				return 30 * 24 * 60 * 60 * 1000
			case 'last_90_days':
				return 90 * 24 * 60 * 60 * 1000
			case 'last_180_days':
				return 180 * 24 * 60 * 60 * 1000
			case 'year_to_date': {
				const now = new Date(nowTimestamp)
				const yearStart = new Date(now.getFullYear(), 0, 1)
				yearStart.setHours(0, 0, 0, 0)
				return now.getTime() - yearStart.getTime()
			}
			case 'all_time':
				return REVENUE_MIN_TIMEFRAME_MS
		}
	}

	if (mode === 'last') {
		const amount = Math.max(1, Math.floor(lastAmount))
		switch (lastUnit) {
			case 'hours':
				return amount * 60 * 60 * 1000
			case 'days':
				return amount * 24 * 60 * 60 * 1000
			case 'weeks':
				return amount * 7 * 24 * 60 * 60 * 1000
			case 'months':
				return REVENUE_MIN_TIMEFRAME_MS
		}
	}

	if (mode === 'custom_range') {
		const start = parseAnalyticsDateInputValue(customStartDate)
		const inclusiveEnd = parseAnalyticsDateInputValue(customEndDate)
		if (!start || !inclusiveEnd) {
			return 0
		}

		return addAnalyticsDays(inclusiveEnd, 1).getTime() - start.getTime()
	}

	const start = parseAnalyticsDateTimeInputValue(customStartDate)
	const end = parseAnalyticsDateTimeInputValue(customEndDate)
	if (!start || !end) {
		return 0
	}

	return end.getTime() - start.getTime()
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
	const normalizedFilters = normalizeAnalyticsSelectedFilters(filters)

	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!isProjectAnalyticsPoint(dataPoint)) {
				continue
			}

			if (!filteredProjectIds.has(dataPoint.source_project)) {
				continue
			}

			if (!doesAnalyticsPointMatchNormalizedFilters(dataPoint, normalizedFilters)) {
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
	return !isServerProject(project) && getProjectStatusFilterValue(project.status) !== 'draft'
}

function getSingleQueryValue(value: unknown): string | undefined {
	if (typeof value !== 'string') {
		return undefined
	}

	const normalizedValue = value.trim()
	return normalizedValue.length > 0 ? normalizedValue : undefined
}

function toAnalyticsDashboardProject(
	project: AnalyticsDashboardProjectSource,
): AnalyticsDashboardProject {
	return {
		id: project.id,
		name: project.name ?? project.title ?? project.id,
		iconUrl: project.icon_url ?? undefined,
		downloads: project.downloads ?? 0,
		status: getProjectStatusFilterValue(project.status),
	}
}

function getUniqueAnalyticsDashboardProjects(
	projects: AnalyticsDashboardProjectSource[],
	seenProjectIds: Set<string>,
): AnalyticsDashboardProject[] {
	const analyticsProjects: AnalyticsDashboardProject[] = []

	for (const project of projects) {
		if (seenProjectIds.has(project.id) || !isAnalyticsEligibleProject(project)) {
			continue
		}

		seenProjectIds.add(project.id)
		analyticsProjects.push(toAnalyticsDashboardProject(project))
	}

	return analyticsProjects
}

function getProjectOrganizationId(
	project: AnalyticsDashboardUserProjectSource,
): string | undefined {
	return typeof project.organization === 'string' && project.organization.trim().length > 0
		? project.organization
		: undefined
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

function toAnalyticsVersionMetadata(
	version: Labrinth.Versions.v3.Version,
): AnalyticsVersionMetadata {
	return {
		id: version.id,
		versionNumber: version.version_number,
		datePublished: version.date_published,
		projectId: version.project_id,
		downloads: version.downloads,
		gameVersions: [...version.game_versions],
		loaders:
			version.mrpack_loaders && version.mrpack_loaders.length > 0
				? [...version.mrpack_loaders]
				: [...version.loaders],
	}
}

function getProjectVersionFilterOptionSummary(
	versions: AnalyticsVersionMetadata[],
): ProjectVersionFilterOptionSummary {
	const gameVersions = new Set<string>()
	const loaders = new Set<string>()
	const versionIds = new Set<string>()

	for (const version of versions) {
		versionIds.add(version.id)

		for (const gameVersion of version.gameVersions) {
			const normalizedGameVersion = gameVersion.trim()
			if (normalizedGameVersion.length > 0) {
				gameVersions.add(normalizedGameVersion)
			}
		}

		for (const loader of version.loaders) {
			const normalizedLoader = loader.trim().toLowerCase()
			if (normalizedLoader.length > 0 && normalizedLoader !== 'mrpack') {
				loaders.add(normalizedLoader)
			}
		}
	}

	return {
		gameVersions: sortStringValues([...gameVersions]),
		loaderTypes: sortStringValues([...loaders]),
		versionIds: sortStringValues([...versionIds]),
	}
}

async function fetchAnalyticsVersionMetadata(
	projectIds: string[],
	getProjects: (ids: string[]) => Promise<Labrinth.Projects.v3.Project[]>,
	getVersions: (ids: string[]) => Promise<Labrinth.Versions.v3.Version[]>,
): Promise<AnalyticsVersionMetadata[]> {
	const projects = await fetchSegmentedWith(projectIds, getProjects)
	const versionIds = sortStringValues([...new Set(projects.flatMap((project) => project.versions))])
	return fetchAnalyticsVersionMetadataByIds(versionIds, getVersions)
}

async function fetchAnalyticsVersionMetadataByIds(
	versionIds: string[],
	getVersions: (ids: string[]) => Promise<Labrinth.Versions.v3.Version[]>,
): Promise<AnalyticsVersionMetadata[]> {
	const metadata: AnalyticsVersionMetadata[] = []
	const segmentSize = 800

	for (let index = 0; index < versionIds.length; index += segmentSize) {
		const versions = await getVersions(versionIds.slice(index, index + segmentSize))
		metadata.push(...versions.map(toAnalyticsVersionMetadata))
	}

	return metadata
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
		user_agent: [...filters.user_agent],
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

function getAnalyticsDataFilterOptionSummary(
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
): AnalyticsDataFilterOptionSummary {
	const countries = new Set<string>()
	const downloadSources = new Set<string>()
	const downloadReasons = new Set<string>()
	const gameVersions = new Set<string>()
	const loaderTypes = new Set<string>()
	const versionIds = new Set<string>()
	const countryDownloadsByCode = new Map<string, number>()

	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!isProjectAnalyticsPoint(dataPoint)) {
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
					if (dataPoint.metric_kind === 'downloads') {
						countryDownloadsByCode.set(
							country,
							(countryDownloadsByCode.get(country) ?? 0) + dataPoint.downloads,
						)
					}
				}
			}

			if (dataPoint.metric_kind === 'downloads' && dataPoint.user_agent) {
				const downloadSource = dataPoint.user_agent.trim()
				if (downloadSource.length > 0) {
					downloadSources.add(downloadSource)
				}
			}

			if (dataPoint.metric_kind === 'downloads' && dataPoint.reason) {
				const downloadReason = dataPoint.reason.trim()
				if (downloadReason.length > 0) {
					downloadReasons.add(downloadReason)
				}
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

			if (
				(dataPoint.metric_kind === 'downloads' || dataPoint.metric_kind === 'playtime') &&
				dataPoint.loader
			) {
				const loader = dataPoint.loader.trim().toLowerCase()
				if (loader.length > 0 && loader !== 'mrpack') {
					loaderTypes.add(loader)
				}
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

	return {
		countries: sortStringValues([...countries]),
		downloadSources: sortStringValues([...downloadSources]),
		downloadReasons: sortStringValues([...downloadReasons]),
		gameVersions: sortStringValues([...gameVersions]),
		loaderTypes: sortStringValues([...loaderTypes]),
		versionIds: sortStringValues([...versionIds]),
		countryDownloadsByCode,
	}
}

function addVersionIdsFromTimeSlices(
	versionIds: Set<string>,
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
) {
	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!isProjectAnalyticsPoint(dataPoint)) {
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

function addVersionProjectNamesFromTimeSlices(
	versionProjectNames: Map<string, string>,
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
	projectNamesById: Map<string, string>,
) {
	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!isProjectAnalyticsPoint(dataPoint)) {
				continue
			}

			if (
				(dataPoint.metric_kind === 'downloads' || dataPoint.metric_kind === 'playtime') &&
				dataPoint.version_id
			) {
				const versionId = dataPoint.version_id.trim()
				const projectName = projectNamesById.get(dataPoint.source_project)
				if (versionId.length > 0 && projectName && !versionProjectNames.has(versionId)) {
					versionProjectNames.set(versionId, projectName)
				}
			}
		}
	}
}

export function doesAnalyticsPointMatchFilters(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
	filters: AnalyticsSelectedFilters,
): boolean {
	return doesAnalyticsPointMatchNormalizedFilters(
		dataPoint,
		normalizeAnalyticsSelectedFilters(filters),
	)
}

export function normalizeAnalyticsSelectedFilters(
	filters: AnalyticsSelectedFilters,
): NormalizedAnalyticsSelectedFilters {
	return {
		country: normalizeAnalyticsFilterValues(filters.country),
		monetization: normalizeAnalyticsFilterValues(filters.monetization),
		userAgent: normalizeAnalyticsFilterValues(filters.user_agent),
		downloadReason: normalizeAnalyticsFilterValues(filters.download_reason),
		versionId: normalizeAnalyticsFilterValues(filters.version_id),
		gameVersion: normalizeAnalyticsFilterValues(filters.game_version),
		loaderType: normalizeAnalyticsFilterValues(filters.loader_type),
	}
}

function normalizeAnalyticsFilterValues(values: string[]): ReadonlySet<string> {
	const normalizedValues = new Set<string>()
	for (const value of values) {
		const normalizedValue = value.trim().toLowerCase()
		if (normalizedValue.length > 0) {
			normalizedValues.add(normalizedValue)
		}
	}
	return normalizedValues
}

export function doesAnalyticsPointMatchNormalizedFilters(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
	filters: NormalizedAnalyticsSelectedFilters,
): boolean {
	switch (dataPoint.metric_kind) {
		case 'views':
			return (
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.country,
					getCountryFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.monetization,
					getMonetizationFilterValue,
				)
			)
		case 'downloads':
			return (
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.country,
					getCountryFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.monetization,
					getMonetizationFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.userAgent,
					getDownloadSourceFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.downloadReason,
					getDownloadReasonFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.versionId,
					getVersionFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.gameVersion,
					getGameVersionFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(dataPoint, filters.loaderType, getLoaderFilterValue)
			)
		case 'playtime':
			return (
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.country,
					getCountryFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.versionId,
					getVersionFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.gameVersion,
					getGameVersionFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(dataPoint, filters.loaderType, getLoaderFilterValue)
			)
		case 'revenue':
			return true
		default:
			return true
	}
}

function doesAnalyticsPointMatchNormalizedFilter(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
	filterValues: ReadonlySet<string>,
	getPointValue: (dataPoint: Labrinth.Analytics.v3.ProjectAnalytics) => string | null | undefined,
): boolean {
	if (filterValues.size === 0) {
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
	return filterValues.has(normalizedPointValue)
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

	return dataPoint.user_agent ?? null
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
	const queryClient = useQueryClient()
	const route = useRoute()
	const router = useRouter()
	const initialQueryState = readAnalyticsQueryBuilderState(route.query, [])
	const initialGraphState = readAnalyticsGraphState(route.query)

	const activeStat = ref<AnalyticsDashboardStat>(initialGraphState.activeStat)
	const activeGraphViewMode = ref<AnalyticsGraphViewMode>(initialGraphState.activeGraphViewMode)
	const isRatioMode = ref(initialGraphState.isRatioMode)
	const showChartEvents = ref(initialGraphState.showChartEvents)
	const hiddenGraphDatasetIds = ref<string[]>(initialGraphState.hiddenGraphDatasetIds)
	const isGraphDatasetSelectionActive = ref(false)
	const selectedGraphDatasetIds = ref<string[]>([])
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
	let revenueHourlyGroupByBeforeOverride: AnalyticsGroupByPreset | null = null
	let nextAnalyticsRouteNavigationMode: AnalyticsQueryBuilderRouteNavigationMode = 'replace'

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
		enabled: shouldFetchEffectiveUser,
		placeholderData: null,
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

	const { data: userProjects, isFetched: hasFetchedUserProjects } = useQuery({
		queryKey: computed(() => ['analytics', 'dashboard', effectiveUserId.value, 'projects']),
		queryFn: async () => {
			try {
				return await client.labrinth.users_v2.getProjects(effectiveUserId.value ?? '')
			} catch (error) {
				if (isUsingDashboardUserOverride.value) {
					return []
				}

				throw error
			}
		},
		enabled: computed(
			() =>
				Boolean(effectiveUserId.value) && !hasProjectContext.value && !hasOrganizationContext.value,
		),
		placeholderData: [],
	})

	const { data: userOrganizations, isFetched: hasFetchedUserOrganizations } = useQuery({
		queryKey: computed(() => ['analytics', 'dashboard', effectiveUserId.value, 'organizations']),
		queryFn: async () => {
			try {
				return await client.labrinth.users_v2.getOrganizations(effectiveUserId.value ?? '')
			} catch (error) {
				if (isUsingDashboardUserOverride.value) {
					return []
				}

				throw error
			}
		},
		enabled: computed(
			() =>
				Boolean(effectiveUserId.value) && !hasProjectContext.value && !hasOrganizationContext.value,
		),
		placeholderData: [],
	})

	const userOrganizationIds = computed(() =>
		(userOrganizations.value ?? []).map((organization) => organization.id),
	)
	const userProjectOrganizationIds = computed(() => {
		const organizationIds: string[] = []
		const seenOrganizationIds = new Set<string>()

		for (const project of userProjects.value ?? []) {
			const organizationId = getProjectOrganizationId(project)
			if (!organizationId || seenOrganizationIds.has(organizationId)) {
				continue
			}

			seenOrganizationIds.add(organizationId)
			organizationIds.push(organizationId)
		}

		return organizationIds
	})
	const extraUserProjectOrganizationIds = computed(() => {
		const organizationIds = new Set(userOrganizationIds.value)
		return userProjectOrganizationIds.value.filter(
			(organizationId) => !organizationIds.has(organizationId),
		)
	})

	const {
		data: extraUserProjectOrganizations,
		isFetched: hasFetchedExtraUserProjectOrganizations,
	} = useQuery({
		queryKey: computed(() => [
			'analytics',
			'dashboard',
			effectiveUserId.value,
			'project-organizations',
			extraUserProjectOrganizationIds.value,
		]),
		queryFn: async () => {
			try {
				return await client.labrinth.organizations_v3.getMultiple(
					extraUserProjectOrganizationIds.value,
				)
			} catch (error) {
				if (isUsingDashboardUserOverride.value) {
					return []
				}

				throw error
			}
		},
		enabled: computed(
			() =>
				Boolean(effectiveUserId.value) &&
				hasFetchedUserProjects.value &&
				extraUserProjectOrganizationIds.value.length > 0 &&
				!hasProjectContext.value &&
				!hasOrganizationContext.value,
		),
		placeholderData: [],
	})

	const { data: userOrganizationProjects, isFetched: hasFetchedUserOrganizationProjects } =
		useQuery({
			queryKey: computed(() => [
				'analytics',
				'dashboard',
				effectiveUserId.value,
				'organization-projects',
				userOrganizationIds.value,
			]),
			queryFn: async () =>
				Promise.all(
					(userOrganizations.value ?? []).map(async (organization) => {
						try {
							return {
								organization,
								projects: await client.labrinth.organizations_v3.getProjects(organization.id),
							}
						} catch (error) {
							if (isUsingDashboardUserOverride.value) {
								return {
									organization,
									projects: [],
								}
							}

							throw error
						}
					}),
				),
			enabled: computed(
				() =>
					Boolean(effectiveUserId.value) &&
					hasFetchedUserOrganizations.value &&
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

		const areExtraUserProjectOrganizationsLoaded =
			extraUserProjectOrganizationIds.value.length === 0 ||
			hasFetchedExtraUserProjectOrganizations.value

		return (
			hasFetchedUserProjects.value &&
			hasFetchedUserOrganizations.value &&
			hasFetchedUserOrganizationProjects.value &&
			areExtraUserProjectOrganizationsLoaded
		)
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

		const seenProjectIds = new Set<string>()
		const personalProjects = getUniqueAnalyticsDashboardProjects(
			(userProjects.value ?? []).filter((project) => !getProjectOrganizationId(project)),
			seenProjectIds,
		)
		const organizationGroups: AnalyticsDashboardProjectGroup[] = []

		const userProjectsByOrganizationId = new Map<string, AnalyticsDashboardProjectSource[]>()
		for (const project of userProjects.value ?? []) {
			const organizationId = getProjectOrganizationId(project)
			if (!organizationId) {
				continue
			}

			const projects = userProjectsByOrganizationId.get(organizationId) ?? []
			projects.push(project)
			userProjectsByOrganizationId.set(organizationId, projects)
		}

		const organizationProjectsById = new Map<string, AnalyticsDashboardProjectSource[]>()
		for (const group of userOrganizationProjects.value ?? []) {
			organizationProjectsById.set(group.organization.id, group.projects)
		}

		const organizationNamesById = new Map<string, string>()
		const orderedOrganizationIds: string[] = []
		for (const organization of userOrganizations.value ?? []) {
			organizationNamesById.set(organization.id, organization.name)
			orderedOrganizationIds.push(organization.id)
		}
		for (const organization of extraUserProjectOrganizations.value ?? []) {
			organizationNamesById.set(organization.id, organization.name)
			if (!orderedOrganizationIds.includes(organization.id)) {
				orderedOrganizationIds.push(organization.id)
			}
		}
		for (const organizationId of userProjectOrganizationIds.value) {
			if (!orderedOrganizationIds.includes(organizationId)) {
				orderedOrganizationIds.push(organizationId)
			}
		}

		for (const organizationId of orderedOrganizationIds) {
			const projects = getUniqueAnalyticsDashboardProjects(
				[
					...(organizationProjectsById.get(organizationId) ?? []),
					...(userProjectsByOrganizationId.get(organizationId) ?? []),
				],
				seenProjectIds,
			)
			if (projects.length === 0) {
				continue
			}

			organizationGroups.push({
				key: organizationId,
				title: organizationNamesById.get(organizationId) ?? organizationId,
				projects,
			})
		}

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
	const hasExplicitProjectSelectionQuery = computed(() =>
		hasAnalyticsProjectSelectionQuery(route.query),
	)
	const hasExplicitBreakdownQuery = computed(() => hasAnalyticsBreakdownQuery(route.query))
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
				selectedBreakdown: selectedBreakdown.value,
				selectedFilters: selectedFilters.value,
			},
			availableProjectIds.value,
		)
		const isGraphDefault = isAnalyticsGraphStateDefault({
			activeStat: activeStat.value,
			activeGraphViewMode: activeGraphViewMode.value,
			isRatioMode: isRatioMode.value,
			showChartEvents: showChartEvents.value,
			hiddenGraphDatasetIds: hiddenGraphDatasetIds.value,
		})

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
			}) > REVENUE_MIN_TIMEFRAME_MS,
	)

	function isAnalyticsDashboardStatAvailableForTimeframe(stat: AnalyticsDashboardStat): boolean {
		return stat !== 'revenue' || isRevenueTimeframeAvailable.value
	}

	function getRelevantAnalyticsDashboardStats(
		breakdown: AnalyticsBreakdownPreset,
		filters: AnalyticsSelectedFilters = selectedFilters.value,
	): readonly AnalyticsDashboardStat[] {
		return getEnabledAnalyticsStatsForState(breakdown, filters).filter((stat) =>
			isAnalyticsDashboardStatAvailableForTimeframe(stat),
		)
	}

	function isAnalyticsDashboardStatRelevant(
		stat: AnalyticsDashboardStat,
		breakdown: AnalyticsBreakdownPreset,
		filters: AnalyticsSelectedFilters = selectedFilters.value,
	): boolean {
		return getRelevantAnalyticsDashboardStats(breakdown, filters).includes(stat)
	}

	function sanitizeAnalyticsSelectedFiltersForContext(
		breakdown: AnalyticsBreakdownPreset,
		filters: AnalyticsSelectedFilters,
	): AnalyticsSelectedFilters {
		const nextFilters = sanitizeAnalyticsSelectedFilters(breakdown, filters)
		if (hasProjectContext.value && nextFilters.project_status.length > 0) {
			return {
				...nextFilters,
				project_status: [],
			}
		}

		return nextFilters
	}

	function replaceNextAnalyticsRouteNavigation() {
		nextAnalyticsRouteNavigationMode = 'replace'
	}

	function consumeAnalyticsRouteNavigationMode(): AnalyticsQueryBuilderRouteNavigationMode {
		const navigationMode = nextAnalyticsRouteNavigationMode
		nextAnalyticsRouteNavigationMode = 'push'
		return navigationMode
	}

	function getSelectedAnalyticsQueryBuilderState() {
		return {
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
		}
	}

	function getSelectedAnalyticsGraphState() {
		return {
			activeStat: activeStat.value,
			activeGraphViewMode: activeGraphViewMode.value,
			isRatioMode: isRatioMode.value,
			showChartEvents: showChartEvents.value,
			hiddenGraphDatasetIds: hiddenGraphDatasetIds.value,
		}
	}

	function syncAnalyticsRouteQuery(navigationMode: AnalyticsQueryBuilderRouteNavigationMode) {
		if (import.meta.server) {
			return
		}

		const nextRouteQuery = buildAnalyticsQueryBuilderRouteQuery(
			route.query,
			getSelectedAnalyticsQueryBuilderState(),
			availableProjectIds.value,
			getSelectedAnalyticsGraphState(),
		)

		const hasAnalyticsQueryChange = hasAnalyticsQueryBuilderRouteChange(route.query, nextRouteQuery)

		if (!hasAnalyticsQueryChange) return

		if (navigationMode === 'replace') {
			router.replace({
				path: route.path,
				query: nextRouteQuery,
			})
		} else {
			router.push({
				path: route.path,
				query: nextRouteQuery,
			})
		}
	}

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
		[selectedBreakdown, selectedFilters, activeStat, isRevenueTimeframeAvailable],
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
			const sanitizedFilters = sanitizeAnalyticsSelectedFiltersForContext(
				nextBreakdown,
				nextFilters,
			)
			if (!areSelectedFiltersEqual(nextFilters, sanitizedFilters)) {
				replaceNextAnalyticsRouteNavigation()
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
					replaceNextAnalyticsRouteNavigation()
					selectedProjectIds.value = []
				}
				return
			}

			const availableProjectIds = new Set(nextProjects.map((project) => project.id))
			if (!hasExplicitProjectSelectionQuery.value) {
				const nextSelectedProjectIds = nextProjects.map((project) => project.id)
				if (!areStringArraysEqual(selectedProjectIds.value, nextSelectedProjectIds)) {
					replaceNextAnalyticsRouteNavigation()
					selectedProjectIds.value = nextSelectedProjectIds
				}
				return
			}

			const retainedSelection = selectedProjectIds.value.filter((id) => availableProjectIds.has(id))
			const nextSelectedProjectIds =
				retainedSelection.length > 0 ? retainedSelection : nextProjects.map((project) => project.id)

			if (!areStringArraysEqual(selectedProjectIds.value, nextSelectedProjectIds)) {
				replaceNextAnalyticsRouteNavigation()
				selectedProjectIds.value = nextSelectedProjectIds
			}
		},
		{ immediate: true },
	)

	watch(
		[selectedProjectIds, hasExplicitBreakdownQuery],
		([nextSelectedProjectIds, nextHasExplicitBreakdownQuery]) => {
			const validBreakdown = getAnalyticsBreakdownPresetForProjectSelection(
				selectedBreakdown.value,
				nextSelectedProjectIds,
			)
			if (selectedBreakdown.value !== validBreakdown) {
				replaceNextAnalyticsRouteNavigation()
				selectedBreakdown.value = validBreakdown
				return
			}

			const defaultBreakdown = getDefaultAnalyticsBreakdownPreset(nextSelectedProjectIds)
			if (!nextHasExplicitBreakdownQuery && selectedBreakdown.value !== defaultBreakdown) {
				replaceNextAnalyticsRouteNavigation()
				selectedBreakdown.value = defaultBreakdown
			}
		},
		{ deep: true, immediate: true },
	)

	watch(
		() => route.query,
		(nextQuery) => {
			const nextQueryState = readAnalyticsQueryBuilderState(nextQuery, availableProjectIds.value)
			const nextGraphState = readAnalyticsGraphState(nextQuery)
			const availableProjectIdSet = new Set(availableProjectIds.value)
			const nextSelectedProjectIds = nextQueryState.selectedProjectIds.filter((projectId) =>
				availableProjectIdSet.has(projectId),
			)
			const nextSelectedBreakdown = getAnalyticsBreakdownPresetForProjectSelection(
				nextQueryState.selectedBreakdown,
				nextSelectedProjectIds,
			)
			const nextSelectedFilters = sanitizeAnalyticsSelectedFiltersForContext(
				nextSelectedBreakdown,
				nextQueryState.selectedFilters,
			)
			const shouldUpdateSelectedProjectIds = !areStringArraysEqual(
				selectedProjectIds.value,
				nextSelectedProjectIds,
			)
			const shouldUpdateSelectedTimeframeMode =
				selectedTimeframeMode.value !== nextQueryState.selectedTimeframeMode
			const shouldUpdateSelectedTimeframe =
				selectedTimeframe.value !== nextQueryState.selectedTimeframe
			const shouldUpdateSelectedLastTimeframeAmount =
				selectedLastTimeframeAmount.value !== nextQueryState.selectedLastTimeframeAmount
			const shouldUpdateSelectedLastTimeframeUnit =
				selectedLastTimeframeUnit.value !== nextQueryState.selectedLastTimeframeUnit
			const shouldUpdateSelectedCustomTimeframeStartDate =
				selectedCustomTimeframeStartDate.value !== nextQueryState.selectedCustomTimeframeStartDate
			const shouldUpdateSelectedCustomTimeframeEndDate =
				selectedCustomTimeframeEndDate.value !== nextQueryState.selectedCustomTimeframeEndDate
			const shouldUpdateSelectedGroupBy = selectedGroupBy.value !== nextQueryState.selectedGroupBy
			const shouldUpdateSelectedBreakdown = selectedBreakdown.value !== nextSelectedBreakdown
			const shouldUpdateSelectedFilters = !areSelectedFiltersEqual(
				selectedFilters.value,
				nextSelectedFilters,
			)
			const shouldUpdateActiveStat = activeStat.value !== nextGraphState.activeStat
			const shouldUpdateActiveGraphViewMode =
				activeGraphViewMode.value !== nextGraphState.activeGraphViewMode
			const shouldUpdateIsRatioMode = isRatioMode.value !== nextGraphState.isRatioMode
			const shouldUpdateShowChartEvents = showChartEvents.value !== nextGraphState.showChartEvents
			const shouldUpdateHiddenGraphDatasetIds = !areStringArraysEqual(
				hiddenGraphDatasetIds.value,
				nextGraphState.hiddenGraphDatasetIds,
			)
			const hasRouteStateUpdate =
				shouldUpdateSelectedProjectIds ||
				shouldUpdateSelectedTimeframeMode ||
				shouldUpdateSelectedTimeframe ||
				shouldUpdateSelectedLastTimeframeAmount ||
				shouldUpdateSelectedLastTimeframeUnit ||
				shouldUpdateSelectedCustomTimeframeStartDate ||
				shouldUpdateSelectedCustomTimeframeEndDate ||
				shouldUpdateSelectedGroupBy ||
				shouldUpdateSelectedBreakdown ||
				shouldUpdateSelectedFilters ||
				shouldUpdateActiveStat ||
				shouldUpdateActiveGraphViewMode ||
				shouldUpdateIsRatioMode ||
				shouldUpdateShowChartEvents ||
				shouldUpdateHiddenGraphDatasetIds

			if (hasRouteStateUpdate) {
				replaceNextAnalyticsRouteNavigation()
			}

			if (shouldUpdateSelectedProjectIds) {
				selectedProjectIds.value = nextSelectedProjectIds
			}
			if (shouldUpdateSelectedTimeframeMode) {
				selectedTimeframeMode.value = nextQueryState.selectedTimeframeMode
			}
			if (shouldUpdateSelectedTimeframe) {
				selectedTimeframe.value = nextQueryState.selectedTimeframe
			}
			if (shouldUpdateSelectedLastTimeframeAmount) {
				selectedLastTimeframeAmount.value = nextQueryState.selectedLastTimeframeAmount
			}
			if (shouldUpdateSelectedLastTimeframeUnit) {
				selectedLastTimeframeUnit.value = nextQueryState.selectedLastTimeframeUnit
			}
			if (shouldUpdateSelectedCustomTimeframeStartDate) {
				selectedCustomTimeframeStartDate.value = nextQueryState.selectedCustomTimeframeStartDate
			}
			if (shouldUpdateSelectedCustomTimeframeEndDate) {
				selectedCustomTimeframeEndDate.value = nextQueryState.selectedCustomTimeframeEndDate
			}
			if (shouldUpdateSelectedGroupBy) {
				selectedGroupBy.value = nextQueryState.selectedGroupBy
			}
			if (shouldUpdateSelectedBreakdown) {
				selectedBreakdown.value = nextSelectedBreakdown
			}
			if (shouldUpdateSelectedFilters) {
				selectedFilters.value = nextSelectedFilters
			}
			if (shouldUpdateActiveStat) {
				activeStat.value = nextGraphState.activeStat
			}
			if (shouldUpdateActiveGraphViewMode) {
				activeGraphViewMode.value = nextGraphState.activeGraphViewMode
			}
			if (shouldUpdateIsRatioMode) {
				isRatioMode.value = nextGraphState.isRatioMode
			}
			if (shouldUpdateShowChartEvents) {
				showChartEvents.value = nextGraphState.showChartEvents
			}
			if (shouldUpdateHiddenGraphDatasetIds) {
				hiddenGraphDatasetIds.value = nextGraphState.hiddenGraphDatasetIds
			}

			if (!hasRouteStateUpdate) {
				syncAnalyticsRouteQuery('replace')
			}
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
			selectedBreakdown,
			selectedFilters,
			availableProjectIds,
		],
		() => {
			syncAnalyticsRouteQuery(consumeAnalyticsRouteNavigationMode())
		},
		{ deep: true, immediate: true },
	)

	watch(
		[activeStat, activeGraphViewMode, isRatioMode, showChartEvents, hiddenGraphDatasetIds],
		() => {
			syncAnalyticsRouteQuery('replace')
		},
		{ deep: true },
	)

	const {
		data: currentTimeSliceData,
		isPending: currentTimeSlicePending,
		isFetching: currentFetching,
		refetch: refetchCurrentTimeSlices,
	} = useQuery({
		queryKey: computed(() =>
			buildAnalyticsCurrentTimeSlicesQueryKey(analyticsQueryUserId.value, fetchRequest.value),
		),
		queryFn: () => {
			if (!isAnalyticsFetchRequestReady(fetchRequest.value)) {
				return []
			}

			return fetchAnalyticsTimeSlices(fetchRequest.value, (request) =>
				client.labrinth.analytics_v3.fetch(request),
			)
		},
		enabled: computed(() => isAnalyticsFetchRequestReady(fetchRequest.value)),
		gcTime: ANALYTICS_TIME_SLICES_GC_TIME_MS,
	})
	const isCurrentTimeSliceLoading = computed(
		() => isAnalyticsFetchRequestReady(fetchRequest.value) && currentTimeSlicePending.value,
	)
	const revenueDailyPrefetchRequest = computed<Labrinth.Analytics.v3.FetchRequest | null>(() => {
		if (!isRevenueHourlyGroupBy(selectedGroupBy.value)) {
			return null
		}
		if (
			!isAnalyticsDashboardStatRelevant('revenue', selectedBreakdown.value, selectedFilters.value)
		) {
			return null
		}

		return buildDailyAnalyticsFetchRequest(fetchRequest.value)
	})

	watch(
		[revenueDailyPrefetchRequest, analyticsQueryUserId],
		([nextFetchRequest, nextAnalyticsQueryUserId]) => {
			if (!isAnalyticsFetchRequestReady(nextFetchRequest)) {
				return
			}

			void queryClient
				.prefetchQuery({
					queryKey: buildAnalyticsCurrentTimeSlicesQueryKey(
						nextAnalyticsQueryUserId,
						nextFetchRequest,
					),
					queryFn: () =>
						fetchAnalyticsTimeSlices(nextFetchRequest, (request) =>
							client.labrinth.analytics_v3.fetch(request),
						),
					gcTime: ANALYTICS_PREFETCH_GC_TIME_MS,
				})
				.catch(() => {})
		},
		{ deep: true, immediate: true },
	)

	const analyticsFilterOptionsRequests = computed<Labrinth.Analytics.v3.FetchRequest[] | null>(
		() => {
			if (sortedSelectedProjectIds.value.length === 0) {
				return null
			}

			return buildAnalyticsFilterOptionsRequests(
				sortedSelectedProjectIds.value,
				new Date(queryRefreshTimestamp.value).toISOString(),
			)
		},
	)

	const {
		data: analyticsFilterOptionsData,
		isFetched: hasFetchedAnalyticsFilterOptions,
		isFetching: isAnalyticsFilterOptionsFetching,
	} = useQuery({
		queryKey: computed(() => [
			'analytics',
			'dashboard',
			analyticsQueryUserId.value,
			'filter-options',
			'analytics-fields',
			sortedSelectedProjectIds.value,
			queryRefreshTimestamp.value,
		]),
		queryFn: async () => {
			const requests = (analyticsFilterOptionsRequests.value ?? []).filter(
				isAnalyticsFetchRequestReady,
			)
			const timeSliceGroups = await Promise.all(
				requests.map((request) =>
					fetchAnalyticsTimeSlices(request, (nextRequest) =>
						client.labrinth.analytics_v3.fetch(nextRequest),
					),
				),
			)
			return timeSliceGroups.flat()
		},
		enabled: computed(() => analyticsFilterOptionsRequests.value !== null),
		gcTime: ANALYTICS_FILTER_OPTIONS_GC_TIME_MS,
	})

	const { data: filterOptionProjectVersions, isFetched: hasFetchedFilterOptionProjectVersions } =
		useQuery({
			queryKey: computed(() => [
				'analytics',
				'dashboard',
				analyticsQueryUserId.value,
				'filter-options',
				'versions',
				sortedSelectedProjectIds.value,
			]),
			queryFn: () =>
				fetchAnalyticsVersionMetadata(
					sortedSelectedProjectIds.value,
					(ids) => client.labrinth.projects_v3.getMultiple(ids),
					(ids) => client.labrinth.versions_v3.getVersions(ids),
				),
			enabled: computed(() => sortedSelectedProjectIds.value.length > 0),
			placeholderData: [],
			gcTime: ANALYTICS_FILTER_OPTIONS_GC_TIME_MS,
		})

	const analyticsDataFilterOptionSummary = computed(() =>
		getAnalyticsDataFilterOptionSummary(analyticsFilterOptionsData.value ?? []),
	)
	const projectVersionFilterOptionSummary = computed(() =>
		getProjectVersionFilterOptionSummary(filterOptionProjectVersions.value ?? []),
	)
	const filterOptions = computed<AnalyticsDashboardFilterOptions>(() => ({
		countries: analyticsDataFilterOptionSummary.value.countries,
		downloadSources: analyticsDataFilterOptionSummary.value.downloadSources,
		downloadReasons: analyticsDataFilterOptionSummary.value.downloadReasons,
		gameVersions: sortStringValues([
			...new Set([
				...projectVersionFilterOptionSummary.value.gameVersions,
				...analyticsDataFilterOptionSummary.value.gameVersions,
			]),
		]),
		loaderTypes: sortStringValues([
			...new Set([
				...projectVersionFilterOptionSummary.value.loaderTypes,
				...analyticsDataFilterOptionSummary.value.loaderTypes,
			]),
		]),
		versionIds: sortStringValues([
			...new Set([
				...projectVersionFilterOptionSummary.value.versionIds,
				...analyticsDataFilterOptionSummary.value.versionIds,
			]),
		]),
	}))
	const isAnalyticsFilterOptionsLoading = computed(() => isAnalyticsFilterOptionsFetching.value)

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

	const previousFetchRequest = computed(() => buildPreviousFetchRequest(fetchRequest.value))
	const hasPreviousPeriodComparison = computed(() => previousFetchRequest.value !== null)

	const {
		data: previousTimeSliceData,
		isFetching: previousFetching,
		refetch: refetchPreviousTimeSlices,
	} = useQuery({
		queryKey: computed(() => [
			'analytics',
			'dashboard',
			analyticsQueryUserId.value,
			'previous',
			previousFetchRequest.value,
		]),
		queryFn: () => {
			if (!isAnalyticsFetchRequestReady(previousFetchRequest.value)) {
				return []
			}

			return fetchAnalyticsTimeSlices(previousFetchRequest.value, (request) =>
				client.labrinth.analytics_v3.fetch(request),
			)
		},
		enabled: computed(() => isAnalyticsFetchRequestReady(previousFetchRequest.value)),
		gcTime: ANALYTICS_TIME_SLICES_GC_TIME_MS,
	})

	const timeSlices = shallowRef<Labrinth.Analytics.v3.TimeSlice[]>([])
	const previousTimeSlices = shallowRef<Labrinth.Analytics.v3.TimeSlice[]>([])
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
	const displayedTimeSlices = shallowRef<Labrinth.Analytics.v3.TimeSlice[]>([])

	function commitDisplayedAnalyticsState() {
		displayedSelectedProjectIds.value = [...selectedProjectIds.value]
		displayedSelectedGroupBy.value = selectedGroupBy.value
		displayedSelectedBreakdown.value = selectedBreakdown.value
		displayedSelectedFilters.value = cloneAnalyticsSelectedFilters(selectedFilters.value)
		displayedFetchRequest.value = cloneAnalyticsFetchRequest(fetchRequest.value)
		displayedFilterOptions.value = cloneAnalyticsFilterOptions(filterOptions.value)
		displayedTimeSlices.value = timeSlices.value
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

		if (selectedBreakdown.value === 'version_id' || selectedFilters.value.version_id.length > 0) {
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
	const projectVersionDownloadsById = computed(
		() => new Map(allVersionMetadata.value.map((version) => [version.id, version.downloads])),
	)
	const countryDownloadsByCode = computed(
		() => analyticsDataFilterOptionSummary.value.countryDownloadsByCode,
	)
	const gameVersionDownloadsByVersion = computed(() => {
		const downloadsByVersion = new Map<string, number>()

		for (const version of allVersionMetadata.value) {
			for (const gameVersion of version.gameVersions) {
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
		const defaultGraphState = buildDefaultAnalyticsGraphState()

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
		activeStat.value = defaultGraphState.activeStat
		activeGraphViewMode.value = defaultGraphState.activeGraphViewMode
		isRatioMode.value = defaultGraphState.isRatioMode
		showChartEvents.value = defaultGraphState.showChartEvents
		hiddenGraphDatasetIds.value = defaultGraphState.hiddenGraphDatasetIds
		isGraphDatasetSelectionActive.value = false
		selectedGraphDatasetIds.value = []
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

	function getVersionProjectName(versionId: string): string | undefined {
		return versionProjectNamesById.value.get(versionId)
	}

	function getVersionProjectIconUrl(versionId: string): string | undefined {
		return versionProjectIconUrlsById.value.get(versionId)
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
		isAnalyticsFilterOptionsLoading,
		versionNumbersById,
		versionPublishedDatesById,
		versionProjectNamesById,
		versionProjectIconUrlsById,
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
		activeGraphViewMode,
		isRatioMode,
		showChartEvents,
		hiddenGraphDatasetIds,
		isGraphDatasetSelectionActive,
		selectedGraphDatasetIds,
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
