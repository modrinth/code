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

import type { OrganizationContext } from '../organization-context'
import {
	type AnalyticsBreakdownPreset,
	type AnalyticsDashboardStat,
	type AnalyticsGraphViewMode,
	type AnalyticsGroupByPreset,
	type AnalyticsLastTimeframeUnit,
	type AnalyticsSelectedBreakdowns,
	type AnalyticsSelectedFilters,
	type AnalyticsTimeframeMode,
	type AnalyticsTimeframePreset,
	areSelectedFiltersEqual,
	areStringArraysEqual,
	buildAnalyticsQueryBuilderRouteQuery,
	buildDefaultAnalyticsGraphState,
	buildDefaultAnalyticsQueryBuilderState,
	getAnalyticsBreakdownPresetsForProjectSelection,
	getDefaultAnalyticsBreakdownPresets,
	getDefaultAnalyticsGraphProjectEventsVisibility,
	hasAnalyticsBreakdownQuery,
	hasAnalyticsGraphProjectEventsVisibilityQuery,
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
	AnalyticsSelectedBreakdowns,
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
const ANALYTICS_PROJECT_IDS_FETCH_BATCH_SIZE = 2000
const ANALYTICS_PROJECT_IDS_FETCH_BATCH_DELAY_MS = 300
const ANALYTICS_MOBILE_LAYOUT_QUERY = '(pointer: coarse), (max-width: 800px)'

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
	organization?: string | null
	icon_url?: string | null
	downloads?: number | null
	status?: string | null
}

type AnalyticsProjectVersionSource = {
	id: string
	versions?: readonly string[] | null
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

const UNKNOWN_ORGANIZATION_NAME = 'Organization'

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

interface AnalyticsFacetsFilterOptionSummary {
	countries: string[]
	downloadSources: string[]
	downloadReasons: string[]
	gameVersions: string[]
	loaderTypes: string[]
	versionIds: string[]
	projectDownloadsById: Map<string, number>
	projectVersionDownloadsById: Map<string, number>
	gameVersionDownloadsByVersion: Map<string, number>
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

type AnalyticsQueryBuilderRouteNavigationMode = 'push' | 'replace'

type AnalyticsTimeSliceSplit = {
	currentTimeSlices: Labrinth.Analytics.v3.TimeSlice[]
	previousTimeSlices: Labrinth.Analytics.v3.TimeSlice[]
}

type AnalyticsFetchData = {
	metrics: Labrinth.Analytics.v3.TimeSlice[]
	project_events: Labrinth.Analytics.v3.ProjectAnalyticsEvent[]
}

function buildComparisonFetchRequest(
	fetchRequest: Labrinth.Analytics.v3.FetchRequest | null,
): AnalyticsProjectFetchRequest | null {
	if (!isAnalyticsFetchRequestReady(fetchRequest)) {
		return null
	}

	const startTimestamp = new Date(fetchRequest.time_range.start).getTime()
	const endTimestamp = new Date(fetchRequest.time_range.end).getTime()
	const duration = endTimestamp - startTimestamp

	if (!Number.isFinite(duration) || duration <= 0) {
		return null
	}

	const previousStart = new Date(startTimestamp - duration)

	if (previousStart.getTime() < ANALYTICS_START_TIME) {
		return null
	}

	return {
		...fetchRequest,
		time_range: {
			start: previousStart.toISOString(),
			end: fetchRequest.time_range.end,
			resolution:
				'slices' in fetchRequest.time_range.resolution
					? {
							slices: fetchRequest.time_range.resolution.slices * 2,
						}
					: fetchRequest.time_range.resolution,
		},
	}
}

function isAnalyticsFetchRequestReady(
	fetchRequest: Labrinth.Analytics.v3.FetchRequest | null,
): fetchRequest is AnalyticsProjectFetchRequest {
	return Array.isArray(fetchRequest?.project_ids) && fetchRequest.project_ids.length > 0
}

function getAnalyticsTimeSliceCount(
	timeRange: Labrinth.Analytics.v3.TimeRange,
	fallback: number,
): number {
	if ('slices' in timeRange.resolution) {
		return Math.max(1, timeRange.resolution.slices)
	}

	const startTime = new Date(timeRange.start).getTime()
	const endTime = new Date(timeRange.end).getTime()
	const bucketMs = timeRange.resolution.minutes * 60 * 1000
	if (bucketMs > 0 && endTime > startTime) {
		return Math.max(1, Math.floor((endTime - startTime) / bucketMs))
	}

	return Math.max(1, fallback)
}

function splitAnalyticsTimeSlices(
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
	fetchRequest: Labrinth.Analytics.v3.FetchRequest | null,
): AnalyticsTimeSliceSplit {
	if (!isAnalyticsFetchRequestReady(fetchRequest) || !buildComparisonFetchRequest(fetchRequest)) {
		return {
			currentTimeSlices: timeSlices,
			previousTimeSlices: [],
		}
	}

	const currentSliceCount = getAnalyticsTimeSliceCount(fetchRequest.time_range, timeSlices.length)
	const currentStartIndex = Math.max(0, timeSlices.length - currentSliceCount)
	const previousStartIndex = Math.max(0, currentStartIndex - currentSliceCount)

	return {
		currentTimeSlices: timeSlices.slice(currentStartIndex),
		previousTimeSlices: timeSlices.slice(previousStartIndex, currentStartIndex),
	}
}

function getAnalyticsProjectEventsInTimeRange(
	projectEvents: Labrinth.Analytics.v3.ProjectAnalyticsEvent[],
	fetchRequest: Labrinth.Analytics.v3.FetchRequest | null,
): Labrinth.Analytics.v3.ProjectAnalyticsEvent[] {
	if (!isAnalyticsFetchRequestReady(fetchRequest)) {
		return projectEvents
	}

	const startTime = new Date(fetchRequest.time_range.start).getTime()
	const endTime = new Date(fetchRequest.time_range.end).getTime()
	if (!Number.isFinite(startTime) || !Number.isFinite(endTime) || endTime < startTime) {
		return []
	}

	return projectEvents.filter((event) => {
		const eventTime = new Date(event.timestamp).getTime()
		return Number.isFinite(eventTime) && eventTime >= startTime && eventTime <= endTime
	})
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

function mergeAnalyticsProjectEvents(
	projectEventGroups: Labrinth.Analytics.v3.ProjectAnalyticsEvent[][],
): Labrinth.Analytics.v3.ProjectAnalyticsEvent[] {
	const mergedProjectEvents: Labrinth.Analytics.v3.ProjectAnalyticsEvent[] = []

	for (const projectEvents of projectEventGroups) {
		for (const projectEvent of projectEvents) {
			mergedProjectEvents.push(projectEvent)
		}
	}

	return mergedProjectEvents.sort((left, right) => {
		const timestampDifference =
			new Date(left.timestamp).getTime() - new Date(right.timestamp).getTime()
		return (
			timestampDifference ||
			left.project_id.localeCompare(right.project_id) ||
			left.kind.localeCompare(right.kind)
		)
	})
}

function waitForAnalyticsFetchBatchDelay(): Promise<void> {
	return new Promise((resolve) => setTimeout(resolve, ANALYTICS_PROJECT_IDS_FETCH_BATCH_DELAY_MS))
}

async function fetchAnalyticsData(
	fetchRequest: AnalyticsProjectFetchRequest,
	fetchAnalytics: (
		request: Labrinth.Analytics.v3.FetchRequest,
	) => Promise<Labrinth.Analytics.v3.FetchResponse>,
): Promise<AnalyticsFetchData> {
	const fetchRequests = buildAnalyticsFetchRequestBatches(fetchRequest)
	const timeSliceGroups: Labrinth.Analytics.v3.TimeSlice[][] = []
	const projectEventGroups: Labrinth.Analytics.v3.ProjectAnalyticsEvent[][] = []

	for (let index = 0; index < fetchRequests.length; index++) {
		if (index > 0) {
			await waitForAnalyticsFetchBatchDelay()
		}

		const response = await fetchAnalytics(fetchRequests[index])
		timeSliceGroups.push(response.metrics)
		projectEventGroups.push(response.project_events ?? [])
	}

	return {
		metrics: mergeAnalyticsTimeSlices(timeSliceGroups),
		project_events: mergeAnalyticsProjectEvents(projectEventGroups),
	}
}

async function fetchAnalyticsTimeSlices(
	fetchRequest: AnalyticsProjectFetchRequest,
	fetchAnalytics: (
		request: Labrinth.Analytics.v3.FetchRequest,
	) => Promise<Labrinth.Analytics.v3.FetchResponse>,
): Promise<Labrinth.Analytics.v3.TimeSlice[]> {
	const response = await fetchAnalyticsData(fetchRequest, fetchAnalytics)
	return response.metrics
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
	refreshTimestamp: number,
) {
	return ['analytics', 'dashboard', userId, 'current', nextFetchRequest, refreshTimestamp]
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

function buildAnalyticsFacetsRequest(
	projectIds: string[],
	timeRange: Labrinth.Analytics.v3.TimeRange,
): Labrinth.Analytics.v3.FetchRequest {
	return {
		time_range: {
			start: timeRange.start,
			end: timeRange.end,
			resolution: {
				slices: 1,
			},
		},
		project_ids: projectIds,
		return_metrics: {
			project_downloads: {
				bucket_by: [
					'project_id',
					'domain',
					'user_agent',
					'version_id',
					'monetized',
					'country',
					'reason',
					'game_version',
					'loader',
				],
			},
		},
	}
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

	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!isProjectAnalyticsPoint(dataPoint)) {
				continue
			}

			if (!filteredProjectIds.has(dataPoint.source_project)) {
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

function getProjectOrganizationId(project: AnalyticsDashboardProjectSource): string | undefined {
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

function getAnalyticsVersionIdsFromProjects(
	projects: readonly AnalyticsProjectVersionSource[],
	projectIds: readonly string[],
): string[] {
	const selectedProjectIds = new Set(projectIds)
	const versionIds = new Set<string>()

	for (const project of projects) {
		if (!selectedProjectIds.has(project.id)) {
			continue
		}

		for (const versionId of project.versions ?? []) {
			const normalizedVersionId = versionId.trim()
			if (normalizedVersionId.length > 0) {
				versionIds.add(normalizedVersionId)
			}
		}
	}

	return sortStringValues([...versionIds])
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

function getEmptyAnalyticsFacetsFilterOptionSummary(): AnalyticsFacetsFilterOptionSummary {
	return {
		countries: [],
		downloadSources: [],
		downloadReasons: [],
		gameVersions: [],
		loaderTypes: [],
		versionIds: [],
		projectDownloadsById: new Map(),
		projectVersionDownloadsById: new Map(),
		gameVersionDownloadsByVersion: new Map(),
		countryDownloadsByCode: new Map(),
	}
}

function getAnalyticsFacetValues<T>(
	facets: Labrinth.Analytics.v3.AnalyticsFacet<T>[] | null | undefined,
): T[] {
	return facets?.map((facet) => facet.value) ?? []
}

function getAnalyticsFacetDownloadsByValue<T>(
	facets: Labrinth.Analytics.v3.AnalyticsFacet<T>[] | null | undefined,
	getKey: (value: T) => string,
): Map<string, number> {
	const downloadsByValue = new Map<string, number>()
	for (const facet of facets ?? []) {
		const key = getKey(facet.value)
		if (key.length === 0) {
			continue
		}

		const downloads = Number.isFinite(facet.downloads) ? facet.downloads : 0
		downloadsByValue.set(key, (downloadsByValue.get(key) ?? 0) + downloads)
	}

	return downloadsByValue
}

function getAnalyticsFacetsFilterOptionSummary(
	facets: Labrinth.Analytics.v3.AnalyticsFacets | null | undefined,
): AnalyticsFacetsFilterOptionSummary {
	if (!facets) {
		return getEmptyAnalyticsFacetsFilterOptionSummary()
	}

	const downloadCountries = getAnalyticsFacetValues(facets.project_downloads.country)
	const downloadGameVersions = getAnalyticsFacetValues(facets.project_downloads.game_version)
	const downloadLoaders = getAnalyticsFacetValues(facets.project_downloads.loader)
	const downloadVersionIds = getAnalyticsFacetValues(facets.project_downloads.version_id)
	const viewCountries = getAnalyticsFacetValues(facets.project_views.country)
	const playtimeCountries = getAnalyticsFacetValues(facets.project_playtime.country)
	const playtimeGameVersions = getAnalyticsFacetValues(facets.project_playtime.game_version)
	const playtimeLoaders = getAnalyticsFacetValues(facets.project_playtime.loader)
	const playtimeVersionIds = getAnalyticsFacetValues(facets.project_playtime.version_id)
	const countries = new Set([...viewCountries, ...downloadCountries, ...playtimeCountries])
	const gameVersions = new Set([...downloadGameVersions, ...playtimeGameVersions])
	const loaderTypes = new Set<string>()
	for (const loader of [...downloadLoaders, ...playtimeLoaders]) {
		const normalizedLoader = loader.trim().toLowerCase()
		if (normalizedLoader.length > 0 && normalizedLoader !== 'mrpack') {
			loaderTypes.add(normalizedLoader)
		}
	}

	return {
		countries: sortStringValues(
			[...countries]
				.map((country) => country.trim().toUpperCase())
				.filter((country) => country.length > 0),
		),
		downloadSources: sortStringValues(getAnalyticsFacetValues(facets.project_downloads.user_agent)),
		downloadReasons: sortStringValues(getAnalyticsFacetValues(facets.project_downloads.reason)),
		gameVersions: sortStringValues(
			[...gameVersions]
				.map((gameVersion) => gameVersion.trim())
				.filter((gameVersion) => gameVersion.length > 0),
		),
		loaderTypes: sortStringValues([...loaderTypes]),
		versionIds: sortStringValues([...new Set([...downloadVersionIds, ...playtimeVersionIds])]),
		projectDownloadsById: getAnalyticsFacetDownloadsByValue(
			facets.project_downloads.project_id,
			(projectId) => projectId.trim(),
		),
		projectVersionDownloadsById: getAnalyticsFacetDownloadsByValue(
			facets.project_downloads.version_id,
			(versionId) => versionId.trim(),
		),
		gameVersionDownloadsByVersion: getAnalyticsFacetDownloadsByValue(
			facets.project_downloads.game_version,
			(gameVersion) => gameVersion.trim(),
		),
		countryDownloadsByCode: getAnalyticsFacetDownloadsByValue(
			facets.project_downloads.country,
			(country) => country.trim().toUpperCase(),
		),
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
	let nextAnalyticsRouteNavigationMode: AnalyticsQueryBuilderRouteNavigationMode = 'replace'
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
			selectedBreakdowns: selectedBreakdowns.value,
			selectedFilters: selectedFilters.value,
		}
	}

	function getSelectedAnalyticsGraphState() {
		return {
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
				!nextHasExplicitBreakdownQuery &&
				!areStringArraysEqual(selectedBreakdowns.value, defaultBreakdowns)
			) {
				replaceNextAnalyticsRouteNavigation()
				selectedBreakdowns.value = defaultBreakdowns
			}
		},
		{ deep: true, immediate: true },
	)

	watch(
		[selectedProjectIds, hasExplicitProjectEventsVisibilityQuery],
		([nextSelectedProjectIds, nextHasExplicitProjectEventsVisibilityQuery]) => {
			const defaultProjectEventsVisibility =
				getDefaultAnalyticsGraphProjectEventsVisibility(nextSelectedProjectIds)

			if (
				!nextHasExplicitProjectEventsVisibilityQuery &&
				showProjectEvents.value !== defaultProjectEventsVisibility
			) {
				showProjectEvents.value = defaultProjectEventsVisibility
			}
		},
		{ deep: true, immediate: true },
	)

	watch(
		() => route.query,
		(nextQuery) => {
			const nextQueryState = readAnalyticsQueryBuilderState(nextQuery, availableProjectIds.value)
			const availableProjectIdSet = new Set(availableProjectIds.value)
			const nextSelectedProjectIds = nextQueryState.selectedProjectIds.filter((projectId) =>
				availableProjectIdSet.has(projectId),
			)
			const nextGraphState = readAnalyticsGraphState(nextQuery, nextSelectedProjectIds)
			const nextSelectedBreakdowns = getAnalyticsBreakdownPresetsForProjectSelection(
				nextQueryState.selectedBreakdowns,
				nextSelectedProjectIds,
			)
			const nextSelectedFilters = sanitizeAnalyticsSelectedFiltersForContext(
				nextSelectedBreakdowns,
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
			const shouldUpdateSelectedBreakdowns = !areStringArraysEqual(
				selectedBreakdowns.value,
				nextSelectedBreakdowns,
			)
			const shouldUpdateSelectedFilters = !areSelectedFiltersEqual(
				selectedFilters.value,
				nextSelectedFilters,
			)
			const shouldUpdateActiveStat = activeStat.value !== nextGraphState.activeStat
			const shouldUpdateActiveGraphViewMode =
				activeGraphViewMode.value !== nextGraphState.activeGraphViewMode
			const shouldUpdateIsRatioMode = isRatioMode.value !== nextGraphState.isRatioMode
			const shouldUpdateShowChartEvents = showChartEvents.value !== nextGraphState.showChartEvents
			const shouldUpdateShowProjectEvents =
				showProjectEvents.value !== nextGraphState.showProjectEvents
			const shouldUpdateShowPreviousPeriod =
				showPreviousPeriod.value !== nextGraphState.showPreviousPeriod
			const shouldUpdateHiddenGraphDatasetIds = !areStringArraysEqual(
				hiddenGraphDatasetIds.value,
				nextGraphState.hiddenGraphDatasetIds,
			)
			const nextHasExplicitGraphDatasetSelection = nextGraphState.selectedGraphDatasetIds !== null
			const nextSelectedGraphDatasetIds = nextGraphState.selectedGraphDatasetIds ?? []
			const shouldUpdateHasExplicitGraphDatasetSelection =
				hasExplicitGraphDatasetSelection.value !== nextHasExplicitGraphDatasetSelection
			const shouldUpdateSelectedGraphDatasetIds =
				(nextHasExplicitGraphDatasetSelection || hasExplicitGraphDatasetSelection.value) &&
				!areStringArraysEqual(selectedGraphDatasetIds.value, nextSelectedGraphDatasetIds)
			const hasRouteStateUpdate =
				shouldUpdateSelectedProjectIds ||
				shouldUpdateSelectedTimeframeMode ||
				shouldUpdateSelectedTimeframe ||
				shouldUpdateSelectedLastTimeframeAmount ||
				shouldUpdateSelectedLastTimeframeUnit ||
				shouldUpdateSelectedCustomTimeframeStartDate ||
				shouldUpdateSelectedCustomTimeframeEndDate ||
				shouldUpdateSelectedGroupBy ||
				shouldUpdateSelectedBreakdowns ||
				shouldUpdateSelectedFilters ||
				shouldUpdateActiveStat ||
				shouldUpdateActiveGraphViewMode ||
				shouldUpdateIsRatioMode ||
				shouldUpdateShowChartEvents ||
				shouldUpdateShowProjectEvents ||
				shouldUpdateShowPreviousPeriod ||
				shouldUpdateHiddenGraphDatasetIds ||
				shouldUpdateHasExplicitGraphDatasetSelection ||
				shouldUpdateSelectedGraphDatasetIds

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
			if (shouldUpdateSelectedBreakdowns) {
				selectedBreakdowns.value = nextSelectedBreakdowns
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
			if (shouldUpdateShowProjectEvents) {
				showProjectEvents.value = nextGraphState.showProjectEvents
			}
			if (shouldUpdateShowPreviousPeriod) {
				showPreviousPeriod.value = nextGraphState.showPreviousPeriod
			}
			if (shouldUpdateHiddenGraphDatasetIds) {
				hiddenGraphDatasetIds.value = nextGraphState.hiddenGraphDatasetIds
			}
			if (shouldUpdateHasExplicitGraphDatasetSelection) {
				hasExplicitGraphDatasetSelection.value = nextHasExplicitGraphDatasetSelection
			}
			if (shouldUpdateSelectedGraphDatasetIds) {
				selectedGraphDatasetIds.value = nextSelectedGraphDatasetIds
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
			selectedBreakdowns,
			selectedFilters,
			availableProjectIds,
		],
		() => {
			syncAnalyticsRouteQuery(consumeAnalyticsRouteNavigationMode())
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
			syncAnalyticsRouteQuery('replace')
		},
		{ deep: true },
	)

	const comparisonFetchRequest = computed(() => buildComparisonFetchRequest(fetchRequest.value))
	const analyticsTimeSlicesFetchRequest = computed(
		() => comparisonFetchRequest.value ?? fetchRequest.value,
	)
	const hasPreviousPeriodComparison = computed(() => comparisonFetchRequest.value !== null)

	const {
		data: currentAnalyticsData,
		isPending: currentTimeSlicePending,
		isFetching: currentFetching,
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
		gcTime: ANALYTICS_TIME_SLICES_GC_TIME_MS,
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
		([
			hasInitializedFetchRequest,
			nextAreProjectsLoaded,
			nextFetchRequest,
			hasCompletedFetch,
		]) => {
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
		return buildComparisonFetchRequest(dailyFetchRequest) ?? dailyFetchRequest
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
						fetchAnalyticsTimeSlices(nextFetchRequest, (request) =>
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
				fetchAnalyticsVersionMetadataByIds(
					filterOptionVersionIds.value,
					(ids) => client.labrinth.versions_v3.getVersions(ids),
				),
			enabled: computed(
				() =>
					filterOptionProjectSources.value !== null && sortedSelectedProjectIds.value.length > 0,
			),
			placeholderData: [],
			gcTime: ANALYTICS_FILTER_OPTIONS_GC_TIME_MS,
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
	const projectDownloadsById = computed(
		() => analyticsFacetsFilterOptionSummary.value.projectDownloadsById,
	)
	const projectVersionDownloadsById = computed(
		() => analyticsFacetsFilterOptionSummary.value.projectVersionDownloadsById,
	)
	const countryDownloadsByCode = computed(
		() => analyticsFacetsFilterOptionSummary.value.countryDownloadsByCode,
	)
	const gameVersionDownloadsByVersion = computed(
		() => analyticsFacetsFilterOptionSummary.value.gameVersionDownloadsByVersion,
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
