import type { Labrinth } from '@modrinth/api-client'

import {
	type AnalyticsBreakdownPreset,
	type AnalyticsDashboardProject,
	type AnalyticsDashboardStat,
	type AnalyticsGroupByPreset,
	type AnalyticsSelectedFilters,
	doesAnalyticsPointMatchNormalizedFilters,
	normalizeAnalyticsSelectedFilters,
} from '~/providers/analytics/analytics'

import { ALL_BREAKDOWN_VALUE, getAnalyticsBreakdownValue } from '../breakdown'

export type ChartDataset = {
	projectId: string
	label: string
	projectName?: string
	data: number[]
	borderColor: string
	backgroundColor: string
	borderDash?: number[]
}

const LOADER_CHART_COLORS: Record<string, string> = {
	fabric: 'var(--color-platform-fabric)',
	'legacy-fabric': 'var(--color-platform-fabric)',
	quilt: 'var(--color-platform-quilt)',
	forge: 'var(--color-platform-forge)',
	neoforge: 'var(--color-platform-neoforge)',
	neo_forge: 'var(--color-platform-neoforge)',
	liteloader: 'var(--color-platform-liteloader)',
	bukkit: 'var(--color-platform-bukkit)',
	bungeecord: 'var(--color-platform-bungeecord)',
	folia: 'var(--color-platform-folia)',
	paper: 'var(--color-platform-paper)',
	purpur: 'var(--color-platform-purpur)',
	spigot: 'var(--color-platform-spigot)',
	velocity: 'var(--color-platform-velocity)',
	waterfall: 'var(--color-platform-waterfall)',
	sponge: 'var(--color-platform-sponge)',
	ornithe: 'var(--color-platform-ornithe)',
	'bta-babric': 'var(--color-platform-bta-babric)',
	nilloader: 'var(--color-platform-nilloader)',
}

const REGION_CODE_PATTERN = /^[a-z]{2}$/i
const OTHER_COUNTRY_CODE = 'XX'
const UNKNOWN_BREAKDOWN_LABEL = 'Unknown'
const ALL_PROJECTS_DATASET_ID = 'all'
const ALL_PROJECTS_DATASET_LABEL = 'All projects'
const regionDisplayNamesByLocale = new Map<string, Intl.DisplayNames | null>()

function getRegionDisplayNames(locale: string): Intl.DisplayNames | null {
	if (regionDisplayNamesByLocale.has(locale)) {
		return regionDisplayNamesByLocale.get(locale) ?? null
	}

	try {
		const displayNames = new Intl.DisplayNames(locale, { type: 'region' })
		regionDisplayNamesByLocale.set(locale, displayNames)
		return displayNames
	} catch {
		regionDisplayNamesByLocale.set(locale, null)
		return null
	}
}

function formatCountryCode(countryCode: string): string {
	const normalized = countryCode.trim().toUpperCase()
	if (normalized === OTHER_COUNTRY_CODE) {
		return UNKNOWN_BREAKDOWN_LABEL
	}

	if (!REGION_CODE_PATTERN.test(normalized)) {
		return countryCode
	}

	const locale = new Intl.DateTimeFormat().resolvedOptions().locale || 'en'
	const localizedDisplayNames = getRegionDisplayNames(locale)
	const localizedValue = localizedDisplayNames?.of(normalized)
	if (localizedValue && localizedValue !== normalized) {
		return localizedValue
	}

	const englishDisplayNames = getRegionDisplayNames('en')
	const englishValue = englishDisplayNames?.of(normalized)
	if (englishValue && englishValue !== normalized) {
		return englishValue
	}

	return countryCode
}

export function formatBreakdownLabel(
	breakdownValue: string,
	selectedBreakdown: AnalyticsBreakdownPreset,
	getVersionDisplayName: (versionId: string) => string = (versionId) => versionId,
): string {
	if (breakdownValue.trim().toLowerCase() === 'other') {
		return UNKNOWN_BREAKDOWN_LABEL
	}
	if (selectedBreakdown === 'country') {
		return formatCountryCode(breakdownValue)
	}
	if (selectedBreakdown === 'version_id') {
		return getVersionDisplayName(breakdownValue)
	}

	return breakdownValue
}

export function shouldCapitalizeBreakdownLabel(
	selectedBreakdown: AnalyticsBreakdownPreset,
): boolean {
	return (
		selectedBreakdown === 'download_reason' ||
		selectedBreakdown === 'monetization' ||
		selectedBreakdown === 'loader' ||
		selectedBreakdown === 'country'
	)
}

function getBreakdownColor(
	breakdownValue: string,
	selectedBreakdown: AnalyticsBreakdownPreset,
	fallbackColor: string,
): string {
	if (selectedBreakdown !== 'loader') {
		return fallbackColor
	}

	const normalizedLoader = breakdownValue.trim().toLowerCase()
	return LOADER_CHART_COLORS[normalizedLoader] ?? fallbackColor
}

type PaletteRankEntry = {
	key: string
	label: string
	total: number
}

function getPaletteColorForIndex(index: number, palette: string[]): string {
	if (palette.length === 0) return ''

	return palette[index % palette.length]
}

function buildPaletteColorsByDownloadRank(
	entries: PaletteRankEntry[],
	palette: string[],
): Map<string, string> {
	const colorsByKey = new Map<string, string>()
	if (palette.length === 0) return colorsByKey

	const sortedEntries = [...entries].sort(
		(a, b) => b.total - a.total || a.label.localeCompare(b.label) || a.key.localeCompare(b.key),
	)
	sortedEntries.forEach((entry, index) => {
		colorsByKey.set(entry.key, getPaletteColorForIndex(index, palette))
	})

	return colorsByKey
}

export function getMetricValue(
	point: Labrinth.Analytics.v3.ProjectAnalytics,
	activeStat: AnalyticsDashboardStat,
): number {
	switch (activeStat) {
		case 'views':
			return point.metric_kind === 'views' ? point.views : 0
		case 'downloads':
			return point.metric_kind === 'downloads' ? point.downloads : 0
		case 'playtime':
			return point.metric_kind === 'playtime' ? point.seconds : 0
		case 'revenue': {
			if (point.metric_kind !== 'revenue') return 0
			const value = Number.parseFloat(point.revenue)
			return Number.isFinite(value) ? value : 0
		}
	}
}

function isMetricKindForStat(
	point: Labrinth.Analytics.v3.ProjectAnalytics,
	activeStat: AnalyticsDashboardStat,
): boolean {
	return point.metric_kind === activeStat
}

function isProjectAnalyticsPointInSelectedProjects(
	point: Labrinth.Analytics.v3.AnalyticsData,
	selectedProjectIds: Set<string>,
): point is Labrinth.Analytics.v3.ProjectAnalytics {
	return 'source_project' in point && selectedProjectIds.has(point.source_project)
}

export function buildChartDatasets(
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
	selectedProjects: AnalyticsDashboardProject[],
	activeStat: AnalyticsDashboardStat,
	palette: string[],
	selectedBreakdown: AnalyticsBreakdownPreset,
	selectedFilters: AnalyticsSelectedFilters,
	getVersionDisplayName: (versionId: string) => string = (versionId) => versionId,
	getVersionProjectName?: (versionId: string) => string | undefined,
	sliceCount: number = timeSlices.length,
): ChartDataset[] {
	const selectedProjectIds = new Set(selectedProjects.map((project) => project.id))
	if (selectedProjectIds.size === 0) {
		return []
	}

	const dataLength = Math.max(sliceCount, timeSlices.length)
	const normalizedFilters = normalizeAnalyticsSelectedFilters(selectedFilters)

	if (selectedBreakdown !== 'none' && selectedBreakdown !== 'project') {
		const dataByBreakdown = new Map<string, number[]>()
		const downloadTotalsByBreakdown = new Map<string, number>()

		timeSlices.forEach((slice, sliceIndex) => {
			for (const point of slice) {
				if (!isProjectAnalyticsPointInSelectedProjects(point, selectedProjectIds)) continue
				if (!doesAnalyticsPointMatchNormalizedFilters(point, normalizedFilters)) continue

				const breakdownValue = getAnalyticsBreakdownValue(point, selectedBreakdown)
				if (breakdownValue === ALL_BREAKDOWN_VALUE) continue

				if (!dataByBreakdown.has(breakdownValue)) {
					dataByBreakdown.set(breakdownValue, new Array(dataLength).fill(0))
				}

				if (point.metric_kind === 'downloads') {
					downloadTotalsByBreakdown.set(
						breakdownValue,
						(downloadTotalsByBreakdown.get(breakdownValue) ?? 0) +
							getMetricValue(point, 'downloads'),
					)
				}

				if (!isMetricKindForStat(point, activeStat)) continue

				const breakdownData = dataByBreakdown.get(breakdownValue)
				if (!breakdownData) continue
				breakdownData[sliceIndex] += getMetricValue(point, activeStat)
			}
		})

		const colorsByBreakdown = buildPaletteColorsByDownloadRank(
			Array.from(dataByBreakdown.keys()).map((breakdownValue) => ({
				key: breakdownValue,
				label: formatBreakdownLabel(breakdownValue, selectedBreakdown, getVersionDisplayName),
				total: downloadTotalsByBreakdown.get(breakdownValue) ?? 0,
			})),
			palette,
		)

		return Array.from(dataByBreakdown.entries()).map(([breakdownValue, data]) => {
			const fallbackColor = colorsByBreakdown.get(breakdownValue) ?? ''
			const color = getBreakdownColor(breakdownValue, selectedBreakdown, fallbackColor)
			return {
				projectId: `breakdown:${breakdownValue}`,
				label: formatBreakdownLabel(breakdownValue, selectedBreakdown, getVersionDisplayName),
				projectName:
					selectedBreakdown === 'version_id' ? getVersionProjectName?.(breakdownValue) : undefined,
				data,
				borderColor: color,
				backgroundColor: color,
			}
		})
	}

	if (selectedBreakdown === 'none') {
		const data = new Array(dataLength).fill(0)
		let downloadTotal = 0

		timeSlices.forEach((slice, sliceIndex) => {
			for (const point of slice) {
				if (!isProjectAnalyticsPointInSelectedProjects(point, selectedProjectIds)) continue
				if (!doesAnalyticsPointMatchNormalizedFilters(point, normalizedFilters)) continue

				if (point.metric_kind === 'downloads') {
					downloadTotal += getMetricValue(point, 'downloads')
				}

				if (!isMetricKindForStat(point, activeStat)) continue

				data[sliceIndex] += getMetricValue(point, activeStat)
			}
		})

		const color =
			buildPaletteColorsByDownloadRank(
				[
					{
						key: ALL_PROJECTS_DATASET_ID,
						label: ALL_PROJECTS_DATASET_LABEL,
						total: downloadTotal,
					},
				],
				palette,
			).get(ALL_PROJECTS_DATASET_ID) ?? ''
		const selectedProject = selectedProjects.length === 1 ? selectedProjects[0] : undefined

		return [
			{
				projectId: ALL_PROJECTS_DATASET_ID,
				label: selectedProject?.name ?? ALL_PROJECTS_DATASET_LABEL,
				data,
				borderColor: color,
				backgroundColor: color,
			},
		]
	}

	const dataByProjectId = new Map<string, number[]>()
	const downloadTotalsByProjectId = new Map<string, number>()
	for (const project of selectedProjects) {
		dataByProjectId.set(project.id, new Array(dataLength).fill(0))
		downloadTotalsByProjectId.set(project.id, 0)
	}

	timeSlices.forEach((slice, sliceIndex) => {
		for (const point of slice) {
			if (!isProjectAnalyticsPointInSelectedProjects(point, selectedProjectIds)) continue
			if (!doesAnalyticsPointMatchNormalizedFilters(point, normalizedFilters)) continue

			if (point.metric_kind === 'downloads') {
				downloadTotalsByProjectId.set(
					point.source_project,
					(downloadTotalsByProjectId.get(point.source_project) ?? 0) +
						getMetricValue(point, 'downloads'),
				)
			}

			if (!isMetricKindForStat(point, activeStat)) continue

			const projectData = dataByProjectId.get(point.source_project)
			if (!projectData) continue

			projectData[sliceIndex] += getMetricValue(point, activeStat)
		}
	})

	const colorsByProjectId = buildPaletteColorsByDownloadRank(
		selectedProjects.map((project) => ({
			key: project.id,
			label: project.name,
			total: downloadTotalsByProjectId.get(project.id) ?? 0,
		})),
		palette,
	)

	return selectedProjects.map((project) => {
		const color = colorsByProjectId.get(project.id) ?? ''
		return {
			projectId: project.id,
			label: project.name,
			data: dataByProjectId.get(project.id) ?? [],
			borderColor: color,
			backgroundColor: color,
		}
	})
}

export function getSliceCount(
	timeRange: Labrinth.Analytics.v3.TimeRange,
	fallback: number,
): number {
	if ('slices' in timeRange.resolution) {
		return Math.max(1, timeRange.resolution.slices)
	}
	if ('minutes' in timeRange.resolution) {
		const duration = new Date(timeRange.end).getTime() - new Date(timeRange.start).getTime()
		const bucketMs = timeRange.resolution.minutes * 60 * 1000
		if (bucketMs > 0 && duration > 0) {
			return Math.max(1, Math.ceil(duration / bucketMs))
		}
	}
	return Math.max(1, fallback)
}

export function getSliceBucketRange(
	timeRange: Labrinth.Analytics.v3.TimeRange,
	sliceCount: number,
	index: number,
): { start: Date; end: Date } {
	const startMs = new Date(timeRange.start).getTime()
	const endMs = new Date(timeRange.end).getTime()
	const bucketMs = sliceCount > 0 ? (endMs - startMs) / sliceCount : 0

	return {
		start: new Date(startMs + index * bucketMs),
		end: new Date(startMs + (index + 1) * bucketMs),
	}
}

const ONE_DAY_MS = 24 * 60 * 60 * 1000
const ONE_MINUTE_MS = 60 * 1000
const YEAR_LABEL_TIME_RANGE_YEARS = 2
const COMPACT_AXIS_THRESHOLD = 5
const SHORT_HOURLY_TIME_LABEL_DURATION_MS = 6 * ONE_DAY_MS
export const DEFAULT_X_AXIS_TICK_LIMIT = 12
export const SHORT_HOURLY_AXIS_TICK_LIMIT = 8

export function buildTimeAxisLabels(
	timeRange: Labrinth.Analytics.v3.TimeRange,
	sliceCount: number,
	groupBy: AnalyticsGroupByPreset,
): string[] {
	const startMs = new Date(timeRange.start).getTime()
	const endMs = new Date(timeRange.end).getTime()
	const totalMs = endMs - startMs
	const bucketMs = sliceCount > 0 ? totalMs / sliceCount : 0
	const includeTime = shouldShowTimeForHourlyAxis(timeRange, groupBy)
	const includeYear = isYearRelevantForTimeRange(timeRange) || groupBy === 'year'

	const dates: Date[] = []
	const dateKeys: string[] = []
	for (let i = 0; i < sliceCount; i++) {
		const date = new Date(startMs + (i + 1) * bucketMs)
		dates.push(date)
		dateKeys.push(`${date.getFullYear()}-${date.getMonth()}-${date.getDate()}`)
	}

	const dateFormatter = new Intl.DateTimeFormat(undefined, {
		month: 'short',
		day: 'numeric',
		...(includeYear ? { year: 'numeric' } : {}),
	})

	if (!includeTime) {
		return dates.map((date) => dateFormatter.format(date))
	}

	const timeFormatter = new Intl.DateTimeFormat(undefined, { hour: 'numeric' })
	const uniqueDateCount = new Set(dateKeys).size

	if (uniqueDateCount <= 1 || isSingleFullDayTimeRange(new Date(startMs), new Date(endMs))) {
		return dates.map((date) => timeFormatter.format(date))
	}

	if (includeTime || sliceCount <= COMPACT_AXIS_THRESHOLD) {
		const dateAndTimeFormatter = new Intl.DateTimeFormat(undefined, {
			month: 'short',
			day: 'numeric',
			hour: 'numeric',
			...(includeYear ? { year: 'numeric' } : {}),
		})
		return dates.map((date) => dateAndTimeFormatter.format(date))
	}

	return dates.map((date) => dateFormatter.format(date))
}

export function isTimeRelevantForGroupBy(groupBy: AnalyticsGroupByPreset): boolean {
	return groupBy === '1h' || groupBy === '6h'
}

export function shouldUseShortHourlyAxis(
	timeRange: Labrinth.Analytics.v3.TimeRange,
	groupBy: AnalyticsGroupByPreset,
): boolean {
	if (!isTimeRelevantForGroupBy(groupBy)) {
		return false
	}

	const durationMs = getTimeRangeDurationMs(timeRange)

	return (
		Number.isFinite(durationMs) &&
		durationMs > 0 &&
		durationMs <= DEFAULT_X_AXIS_TICK_LIMIT * ONE_DAY_MS
	)
}

export function getShortHourlyAxisTickLimit(
	timeRange: Labrinth.Analytics.v3.TimeRange,
	groupBy: AnalyticsGroupByPreset,
): number | undefined {
	if (!shouldUseShortHourlyAxis(timeRange, groupBy)) {
		return undefined
	}

	const durationMs = getTimeRangeDurationMs(timeRange)
	if (durationMs > SHORT_HOURLY_TIME_LABEL_DURATION_MS) {
		return Math.min(DEFAULT_X_AXIS_TICK_LIMIT, Math.ceil(durationMs / ONE_DAY_MS))
	}

	return SHORT_HOURLY_AXIS_TICK_LIMIT
}

function shouldShowTimeForHourlyAxis(
	timeRange: Labrinth.Analytics.v3.TimeRange,
	groupBy: AnalyticsGroupByPreset,
): boolean {
	const durationMs = getTimeRangeDurationMs(timeRange)
	return (
		isTimeRelevantForGroupBy(groupBy) &&
		Number.isFinite(durationMs) &&
		durationMs > 0 &&
		durationMs <= SHORT_HOURLY_TIME_LABEL_DURATION_MS
	)
}

function getTimeRangeDurationMs(timeRange: Labrinth.Analytics.v3.TimeRange): number {
	return new Date(timeRange.end).getTime() - new Date(timeRange.start).getTime()
}

export function isYearRelevantForTimeRange(timeRange: Labrinth.Analytics.v3.TimeRange): boolean {
	const start = new Date(timeRange.start)
	const end = new Date(timeRange.end)
	const yearLabelThreshold = new Date(start)
	yearLabelThreshold.setFullYear(start.getFullYear() + YEAR_LABEL_TIME_RANGE_YEARS)

	return (
		Number.isFinite(start.getTime()) &&
		Number.isFinite(end.getTime()) &&
		end.getTime() > yearLabelThreshold.getTime()
	)
}

export function formatBucketEndLabel(end: Date, includeTime: boolean, includeYear = false): string {
	if (includeTime) {
		return new Intl.DateTimeFormat(undefined, {
			month: 'short',
			day: 'numeric',
			...(includeYear ? { year: 'numeric' } : {}),
			hour: 'numeric',
			minute: '2-digit',
		}).format(end)
	}

	return new Intl.DateTimeFormat(undefined, {
		month: 'short',
		day: 'numeric',
		...(includeYear ? { year: 'numeric' } : {}),
	}).format(end)
}

function isStartOfDay(date: Date): boolean {
	return (
		date.getHours() === 0 &&
		date.getMinutes() === 0 &&
		date.getSeconds() === 0 &&
		date.getMilliseconds() === 0
	)
}

function isSingleFullDayTimeRange(start: Date, end: Date): boolean {
	const durationMs = end.getTime() - start.getTime()
	return (
		Math.abs(durationMs - ONE_DAY_MS) < ONE_MINUTE_MS && isStartOfDay(start) && isStartOfDay(end)
	)
}

export function formatMetricValue(
	value: number,
	activeStat: AnalyticsDashboardStat,
	formatNumber: (value: number) => string,
): string {
	switch (activeStat) {
		case 'revenue': {
			const amount = Math.round(value * 100) / 100
			return `$${formatNumber(amount)}`
		}
		case 'playtime': {
			const hours = value / 3600
			return `${hours.toFixed(1)} hrs`
		}
		case 'views':
		case 'downloads':
		default:
			return formatNumber(Math.round(value))
	}
}

function formatSmallAxisNumber(value: number): string {
	const rounded = Math.round(value)
	if (Math.abs(value - rounded) < 0.0000001) {
		return String(rounded)
	}

	const formattedValue = Math.abs(value) < 1 ? value.toFixed(2) : value.toFixed(1)
	return formattedValue.replace(/\.?0+$/, '')
}

export function formatAxisValue(
	value: number,
	activeStat: AnalyticsDashboardStat,
	formatCompact: (value: number) => string,
): string {
	switch (activeStat) {
		case 'revenue':
			return `$${formatCompact(Math.round(value * 100) / 100)}`
		case 'playtime':
			return `${(value / 3600).toFixed(1)}h`
		case 'views':
		case 'downloads':
		default:
			if (Math.abs(value) < 10) {
				return formatSmallAxisNumber(value)
			}
			return formatCompact(Math.round(value))
	}
}
