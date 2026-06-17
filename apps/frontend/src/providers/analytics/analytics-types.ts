import type { Labrinth } from '@modrinth/api-client'
import type { LocationQueryValueRaw } from 'vue-router'

import type { ProjectStatusFilterValue } from '~/components/analytics-dashboard/query-builder/query-filter-utils'

export type AnalyticsQueryFilterCategory =
	| 'project'
	| 'project_status'
	| 'country'
	| 'monetization'
	| 'user_agent'
	| 'download_reason'
	| 'version_id'
	| 'game_version'
	| 'loader_type'
	| 'dependent_project_id'
	| 'dependent_project_type'

export type AnalyticsTimeframePreset =
	| 'today'
	| 'yesterday'
	| 'last_7_days'
	| 'last_14_days'
	| 'last_30_days'
	| 'last_90_days'
	| 'last_180_days'
	| 'year_to_date'
	| 'all_time'

export type AnalyticsTimeframeMode = 'preset' | 'last' | 'custom_range' | 'custom_datetime_range'
export type AnalyticsLastTimeframeUnit = 'hours' | 'days' | 'weeks' | 'months'

export type AnalyticsGroupByPreset = '1h' | '6h' | 'day' | 'week' | 'month' | 'year'

export type AnalyticsBreakdownPreset =
	| 'none'
	| 'project'
	| 'country'
	| 'monetization'
	| 'user_agent'
	| 'download_reason'
	| 'version_id'
	| 'loader'
	| 'game_version'
	| 'dependent_project_download'

export type AnalyticsSelectedBreakdowns = Exclude<AnalyticsBreakdownPreset, 'none'>[]
export type AnalyticsDashboardStat = 'views' | 'downloads' | 'revenue' | 'playtime'
export type AnalyticsGraphViewMode = 'line' | 'area' | 'bar'
export type AnalyticsTableSortColumn =
	| 'date'
	| 'project'
	| 'dependent_on'
	| 'breakdown'
	| `breakdown_${Exclude<AnalyticsBreakdownPreset, 'none'>}`
	| 'views'
	| 'downloads'
	| 'revenue'
	| 'playtime'
export type AnalyticsTableSortDirection = 'asc' | 'desc'

export type AnalyticsSelectedFilters = Record<AnalyticsQueryFilterCategory, string[]>

export type AnalyticsQueryBuilderState = {
	selectedProjectIds: string[]
	selectedTimeframeMode: AnalyticsTimeframeMode
	selectedTimeframe: AnalyticsTimeframePreset
	selectedLastTimeframeAmount: number
	selectedLastTimeframeUnit: AnalyticsLastTimeframeUnit
	selectedCustomTimeframeStartDate: string
	selectedCustomTimeframeEndDate: string
	selectedGroupBy: AnalyticsGroupByPreset
	selectedBreakdowns: AnalyticsSelectedBreakdowns
	selectedFilters: AnalyticsSelectedFilters
}

export type AnalyticsGraphState = {
	activeStat: AnalyticsDashboardStat
	activeGraphViewMode: AnalyticsGraphViewMode
	isRatioMode: boolean
	showChartEvents: boolean
	showProjectEvents: boolean
	showPreviousPeriod: boolean
	hiddenGraphDatasetIds: string[]
	selectedGraphDatasetIds: string[] | null
}

export type AnalyticsTableSortState = {
	sortColumn: AnalyticsTableSortColumn | undefined
	sortDirection: AnalyticsTableSortDirection
}

export type MutableRouteQuery = Record<
	string,
	LocationQueryValueRaw | LocationQueryValueRaw[] | undefined
>

export type ProjectTypeMetadata = {
	project_type?: string | null
	project_types?: readonly string[] | null
	projectTypes?: readonly string[] | null
}

export type AnalyticsProjectFetchRequest = Labrinth.Analytics.v3.FetchRequest & {
	project_ids: string[]
}

export type AnalyticsDashboardProjectSource = ProjectTypeMetadata & {
	id: string
	name?: string | null
	title?: string | null
	organization?: string | null
	icon_url?: string | null
	downloads?: number | null
	status?: string | null
	published?: string | null
}

export type AnalyticsProjectVersionSource = {
	id: string
	versions?: readonly string[] | null
}

export interface AnalyticsDashboardProject {
	id: string
	name: string
	iconUrl?: string
	organizationId?: string
	downloads: number
	status: ProjectStatusFilterValue
	publishedAt?: string
	projectTypes: string[]
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
	dependentProjectTypes: string[]
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
	dependentProjectId: ReadonlySet<string>
	dependentProjectType: ReadonlySet<string>
}

export interface AnalyticsFacetsFilterOptionSummary {
	countries: string[]
	downloadSources: string[]
	downloadReasons: string[]
	gameVersions: string[]
	loaderTypes: string[]
	dependentProjectTypes: string[]
	versionIds: string[]
	projectDownloadsById: Map<string, number>
	projectVersionDownloadsById: Map<string, number>
	gameVersionDownloadsByVersion: Map<string, number>
	countryDownloadsByCode: Map<string, number>
}

export interface ProjectVersionFilterOptionSummary {
	gameVersions: string[]
	loaderTypes: string[]
	versionIds: string[]
}

export interface AnalyticsVersionMetadata {
	id: string
	versionNumber: string
	datePublished: string
	projectId: string
	downloads: number
	gameVersions: string[]
	loaders: string[]
}

export type AnalyticsTimeSliceSplit = {
	currentTimeSlices: Labrinth.Analytics.v3.TimeSlice[]
	previousTimeSlices: Labrinth.Analytics.v3.TimeSlice[]
}

export type AnalyticsFetchData = {
	metrics: Labrinth.Analytics.v3.TimeSlice[]
	projects: Record<string, Labrinth.Projects.v3.Project>
	project_events: Labrinth.Analytics.v3.ProjectAnalyticsEvent[]
}
