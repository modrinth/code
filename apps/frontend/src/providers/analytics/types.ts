import type { Labrinth } from '@modrinth/api-client'

import type { ProjectStatusFilterValue } from '~/components/analytics-dashboard/query-builder/query-filter'

export type ProjectTypeMetadata = {
	project_type?: string | null
	project_types?: readonly string[] | null
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
}

export type AnalyticsProjectVersionSource = {
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

export interface AnalyticsFacetsFilterOptionSummary {
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
	project_events: Labrinth.Analytics.v3.ProjectAnalyticsEvent[]
}
