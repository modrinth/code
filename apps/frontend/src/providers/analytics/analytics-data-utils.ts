import type { Labrinth } from '@modrinth/api-client'

import type { ProjectStatusFilterValue } from '~/components/analytics-dashboard/query-builder/query-filter-utils'

import { getProjectIdsMatchingStatusFilter } from './analytics-project-utils'
import type {
	AnalyticsDashboardTotals,
	AnalyticsFetchData,
	AnalyticsGroupByPreset,
	AnalyticsLastTimeframeUnit,
	AnalyticsProjectFetchRequest,
	AnalyticsSelectedFilters,
	AnalyticsTimeframeMode,
	AnalyticsTimeframePreset,
	AnalyticsTimeSliceSplit,
} from './analytics-types'

const ANALYTICS_START_TIMESTAMP = '2023-01-01T00:00:00.000Z'
export const ANALYTICS_START_DATE_INPUT_VALUE = ANALYTICS_START_TIMESTAMP.slice(0, 10)
export const ANALYTICS_START_TIME = new Date(ANALYTICS_START_TIMESTAMP).getTime()
export const REVENUE_MIN_TIMEFRAME_MS = 1 * 24 * 60 * 60 * 1000 // need at least 1 day in timeframe range to show revenue
const ANALYTICS_DAY_MS = 24 * 60 * 60 * 1000
const ANALYTICS_MAX_TIME_SLICES = 256 // controls granularity allowed in "group by" for timeframe ranges
const ANALYTICS_PROJECT_IDS_FETCH_BATCH_SIZE = 40
const ANALYTICS_PROJECT_IDS_FETCH_BATCH_DELAY_MS = 300

function isProjectAnalyticsPoint(
	dataPoint: Labrinth.Analytics.v3.AnalyticsData,
): dataPoint is Labrinth.Analytics.v3.ProjectAnalytics {
	return 'source_project' in dataPoint
}

export function buildComparisonFetchRequest(
	fetchRequest: Labrinth.Analytics.v3.FetchRequest | null,
	minStartTime = ANALYTICS_START_TIME,
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

	if (previousStart.getTime() < minStartTime) {
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

export function isAnalyticsFetchRequestReady(
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

export function splitAnalyticsTimeSlices(
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
	fetchRequest: Labrinth.Analytics.v3.FetchRequest | null,
	minStartTime = ANALYTICS_START_TIME,
): AnalyticsTimeSliceSplit {
	if (
		!isAnalyticsFetchRequestReady(fetchRequest) ||
		!buildComparisonFetchRequest(fetchRequest, minStartTime)
	) {
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

export function getAnalyticsProjectEventsInTimeRange(
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

export async function fetchAnalyticsData(
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

export async function fetchAnalyticsTimeSlices(
	fetchRequest: AnalyticsProjectFetchRequest,
	fetchAnalytics: (
		request: Labrinth.Analytics.v3.FetchRequest,
	) => Promise<Labrinth.Analytics.v3.FetchResponse>,
): Promise<Labrinth.Analytics.v3.TimeSlice[]> {
	const response = await fetchAnalyticsData(fetchRequest, fetchAnalytics)
	return response.metrics
}

export function areAnalyticsFetchRequestsEqual(
	left: Labrinth.Analytics.v3.FetchRequest | null,
	right: Labrinth.Analytics.v3.FetchRequest,
): boolean {
	return JSON.stringify(left) === JSON.stringify(right)
}

export function buildAnalyticsCurrentTimeSlicesQueryKey(
	userId: string | undefined,
	nextFetchRequest: Labrinth.Analytics.v3.FetchRequest | null,
	refreshTimestamp: number,
) {
	return ['analytics', 'dashboard', userId, 'current', nextFetchRequest, refreshTimestamp]
}

export function isRevenueHourlyGroupBy(groupBy: AnalyticsGroupByPreset): boolean {
	return groupBy === '1h' || groupBy === '6h'
}

export function buildDailyAnalyticsFetchRequest(
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

export function buildAnalyticsFacetsRequest(
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

export function getAnalyticsTimeframeDurationMs({
	mode,
	preset,
	lastAmount,
	lastUnit,
	customStartDate,
	customEndDate,
	nowTimestamp,
	allTimeStartTimestamp = ANALYTICS_START_TIME,
}: {
	mode: AnalyticsTimeframeMode
	preset: AnalyticsTimeframePreset
	lastAmount: number
	lastUnit: AnalyticsLastTimeframeUnit
	customStartDate: string
	customEndDate: string
	nowTimestamp: number
	allTimeStartTimestamp?: number
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
			case 'all_time': {
				const allTimeDurationMs = nowTimestamp - allTimeStartTimestamp
				return Math.max(0, allTimeDurationMs)
			}
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

export function getPercentChange(currentValue: number, previousValue: number): number {
	if (previousValue === 0) {
		if (currentValue === 0) {
			return 0
		}
		return 100
	}

	return ((currentValue - previousValue) / previousValue) * 100
}

export function computeTotals(
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

export function getProjectDownloadsByIdFromTimeSlices(
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
): Map<string, number> {
	const projectDownloadsById = new Map<string, number>()

	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!isProjectAnalyticsPoint(dataPoint) || dataPoint.metric_kind !== 'downloads') {
				continue
			}

			projectDownloadsById.set(
				dataPoint.source_project,
				(projectDownloadsById.get(dataPoint.source_project) ?? 0) + dataPoint.downloads,
			)
		}
	}

	return projectDownloadsById
}

function getDownloadFieldCountsFromTimeSlices(
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
	getKey: (
		dataPoint: Extract<Labrinth.Analytics.v3.ProjectMetrics, { metric_kind: 'downloads' }>,
	) => string | null | undefined,
): Map<string, number> {
	const downloadsByValue = new Map<string, number>()

	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!isProjectAnalyticsPoint(dataPoint) || dataPoint.metric_kind !== 'downloads') {
				continue
			}

			const key = getKey(dataPoint)?.trim()
			if (!key) {
				continue
			}

			downloadsByValue.set(key, (downloadsByValue.get(key) ?? 0) + dataPoint.downloads)
		}
	}

	return downloadsByValue
}

export function getProjectVersionDownloadsByIdFromTimeSlices(
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
): Map<string, number> {
	return getDownloadFieldCountsFromTimeSlices(timeSlices, (dataPoint) => dataPoint.version_id)
}

export function getGameVersionDownloadsByVersionFromTimeSlices(
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
): Map<string, number> {
	return getDownloadFieldCountsFromTimeSlices(timeSlices, (dataPoint) => dataPoint.game_version)
}

export function getCountryDownloadsByCodeFromTimeSlices(
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
): Map<string, number> {
	const countryDownloadsByCode = new Map<string, number>()
	const downloadsByCountry = getDownloadFieldCountsFromTimeSlices(
		timeSlices,
		(dataPoint) => dataPoint.country,
	)

	for (const [country, downloads] of downloadsByCountry.entries()) {
		const countryCode = country.toUpperCase()
		countryDownloadsByCode.set(
			countryCode,
			(countryDownloadsByCode.get(countryCode) ?? 0) + downloads,
		)
	}

	return countryDownloadsByCode
}

export function cloneAnalyticsFetchRequest(
	fetchRequest: Labrinth.Analytics.v3.FetchRequest | null,
): Labrinth.Analytics.v3.FetchRequest | null {
	return fetchRequest ? JSON.parse(JSON.stringify(fetchRequest)) : null
}

export function addVersionIdsFromTimeSlices(
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

export function addVersionProjectNamesFromTimeSlices(
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
