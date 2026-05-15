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
const ANALYTICS_START_TIME = new Date(ANALYTICS_START_TIMESTAMP).getTime()
const REVENUE_GROUP_BY_FALLBACK: AnalyticsGroupByPreset = 'day'
const REVENUE_MIN_TIMEFRAME_MS = 1 * 24 * 60 * 60 * 1000 // need at least 1 day in timeframe range to show revenue
const ANALYTICS_DAY_MS = 24 * 60 * 60 * 1000
const ANALYTICS_MAX_TIME_SLICES = 256 // controls granularity allowed in "group by" for timeframe ranges

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
	showAllLegendEntries: Ref<boolean>
	hiddenGraphDatasetIds: Ref<string[]>
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
	const queryClient = useQueryClient()
	const route = useRoute()
	const router = useRouter()
	const initialQueryState = readAnalyticsQueryBuilderState(route.query, [])
	const initialGraphState = readAnalyticsGraphState(route.query)

	const activeStat = ref<AnalyticsDashboardStat>(initialGraphState.activeStat)
	const activeGraphViewMode = ref<AnalyticsGraphViewMode>(initialGraphState.activeGraphViewMode)
	const isRatioMode = ref(initialGraphState.isRatioMode)
	const showChartEvents = ref(initialGraphState.showChartEvents)
	const showAllLegendEntries = ref(initialGraphState.showAllLegendEntries)
	const hiddenGraphDatasetIds = ref<string[]>(initialGraphState.hiddenGraphDatasetIds)
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
			showAllLegendEntries: showAllLegendEntries.value,
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
			showAllLegendEntries: showAllLegendEntries.value,
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
		() => route.query,
		(nextQuery) => {
			const nextQueryState = readAnalyticsQueryBuilderState(nextQuery, availableProjectIds.value)
			const nextGraphState = readAnalyticsGraphState(nextQuery)
			const availableProjectIdSet = new Set(availableProjectIds.value)
			const nextSelectedProjectIds = nextQueryState.selectedProjectIds.filter((projectId) =>
				availableProjectIdSet.has(projectId),
			)
			const nextSelectedFilters = sanitizeAnalyticsSelectedFiltersForContext(
				nextQueryState.selectedBreakdown,
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
			const shouldUpdateSelectedBreakdown =
				selectedBreakdown.value !== nextQueryState.selectedBreakdown
			const shouldUpdateSelectedFilters = !areSelectedFiltersEqual(
				selectedFilters.value,
				nextSelectedFilters,
			)
			const shouldUpdateActiveStat = activeStat.value !== nextGraphState.activeStat
			const shouldUpdateActiveGraphViewMode =
				activeGraphViewMode.value !== nextGraphState.activeGraphViewMode
			const shouldUpdateIsRatioMode = isRatioMode.value !== nextGraphState.isRatioMode
			const shouldUpdateShowChartEvents = showChartEvents.value !== nextGraphState.showChartEvents
			const shouldUpdateShowAllLegendEntries =
				showAllLegendEntries.value !== nextGraphState.showAllLegendEntries
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
				shouldUpdateShowAllLegendEntries ||
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
				selectedBreakdown.value = nextQueryState.selectedBreakdown
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
			if (shouldUpdateShowAllLegendEntries) {
				showAllLegendEntries.value = nextGraphState.showAllLegendEntries
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
		[
			activeStat,
			activeGraphViewMode,
			isRatioMode,
			showChartEvents,
			showAllLegendEntries,
			hiddenGraphDatasetIds,
		],
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
					queryFn: async () => {
						const response = await client.labrinth.analytics_v3.fetch(nextFetchRequest)
						return response.metrics
					},
				})
				.catch(() => {})
		},
		{ deep: true, immediate: true },
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
				analyticsQueryUserId.value,
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
				analyticsQueryUserId.value,
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
		queryKey: computed(() => [
			'analytics',
			'dashboard',
			analyticsQueryUserId.value,
			'previous',
			previousFetchRequest.value,
		]),
		queryFn: async () => {
			const response = await client.labrinth.analytics_v3.fetch(
				previousFetchRequest.value as Labrinth.Analytics.v3.FetchRequest,
			)
			return response.metrics
		},
		enabled: computed(() => previousFetchRequest.value !== null),
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

		for (const versionId of filterOptionProjectVersionIds.value) {
			versionIds.delete(versionId)
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
	const versionProjectNamesById = computed(() => {
		const projectNames = projectNamesById.value
		const versionProjectNames = new Map<string, string>()
		for (const version of allVersionMetadata.value) {
			const projectName = projectNames.get(version.project_id)
			if (projectName) {
				versionProjectNames.set(version.id, projectName)
			}
		}
		return versionProjectNames
	})
	const versionProjectIconUrlsById = computed(() => {
		const projectIconUrls = projectIconUrlsById.value
		const versionProjectIconUrls = new Map<string, string>()
		for (const version of allVersionMetadata.value) {
			const projectIconUrl = projectIconUrls.get(version.project_id)
			if (projectIconUrl) {
				versionProjectIconUrls.set(version.id, projectIconUrl)
			}
		}
		return versionProjectIconUrls
	})
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
		showAllLegendEntries.value = defaultGraphState.showAllLegendEntries
		hiddenGraphDatasetIds.value = defaultGraphState.hiddenGraphDatasetIds
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
		showAllLegendEntries,
		hiddenGraphDatasetIds,
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
