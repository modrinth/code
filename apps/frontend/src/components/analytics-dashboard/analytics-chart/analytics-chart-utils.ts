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

import type { FormatMessage } from '../analytics-messages'
import {
	analyticsChartMessages,
	analyticsMessages,
	analyticsStatCardMessages,
	formatAnalyticsDependentProjectFallbackLabel,
	formatAnalyticsDownloadReasonLabel,
	formatAnalyticsLoaderLabel,
	formatAnalyticsMonetizationLabel,
} from '../analytics-messages'
import {
	ALL_BREAKDOWN_VALUE,
	COMBINED_BREAKDOWN_LABEL_SEPARATOR,
	getAnalyticsBreakdownDatasetId,
	getAnalyticsBreakdownKey,
	getAnalyticsBreakdownValues,
	isUnknownAnalyticsBreakdownValue,
	UNKNOWN_BREAKDOWN_VALUE,
} from '../breakdown'
import { PREVIOUS_PERIOD_DATASET_ID_PREFIX } from './analytics-chart-constants'

export type ChartDataset = {
	projectId: string
	label: string
	projectName?: string
	tooltip?: string
	data: number[]
	borderColor: string
	backgroundColor: string
	borderDash?: number[]
}

export function getChartDatasetTotal(dataset: ChartDataset) {
	return dataset.data.reduce((sum, value) => sum + value, 0)
}

export function getPreviousPeriodDatasetId(datasetId: string) {
	return `${PREVIOUS_PERIOD_DATASET_ID_PREFIX}${datasetId}`
}

export function decodeBreakdownDatasetValue(value: string) {
	try {
		return decodeURIComponent(value)
	} catch {
		return value
	}
}

export function areStringArraysEqual(left: string[], right: string[]) {
	if (left.length !== right.length) return false
	for (let index = 0; index < left.length; index += 1) {
		if (left[index] !== right[index]) return false
	}
	return true
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
const ALL_PROJECTS_DATASET_ID = 'all'
const MONETIZATION_CHART_COLOR_INDEX: Record<string, number> = {
	monetized: 0,
	unmonetized: 1,
}
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

function formatCountryCode(countryCode: string, formatMessage: FormatMessage): string {
	const normalized = countryCode.trim().toUpperCase()
	if (normalized === OTHER_COUNTRY_CODE) {
		return formatMessage(analyticsMessages.other)
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
	getVersionDisplayName: ((versionId: string) => string) | undefined,
	formatMessage: FormatMessage,
): string {
	const normalizedValue = breakdownValue.trim()
	const normalizedLowercaseValue = normalizedValue.toLowerCase()

	if (
		normalizedValue === UNKNOWN_BREAKDOWN_VALUE ||
		normalizedLowercaseValue === 'other' ||
		normalizedLowercaseValue === 'unknown'
	) {
		if (selectedBreakdown === 'country') {
			return formatMessage(analyticsMessages.other)
		}
		return formatMessage(analyticsMessages.unknown)
	}
	if (selectedBreakdown === 'country') {
		return formatCountryCode(breakdownValue, formatMessage)
	}
	if (selectedBreakdown === 'monetization') {
		return formatAnalyticsMonetizationLabel(normalizedLowercaseValue, formatMessage)
	}
	if (selectedBreakdown === 'download_reason') {
		return formatAnalyticsDownloadReasonLabel(normalizedLowercaseValue, formatMessage)
	}
	if (selectedBreakdown === 'dependent_project_download') {
		return breakdownValue
	}
	if (selectedBreakdown === 'version_id') {
		return getVersionDisplayName?.(breakdownValue) ?? breakdownValue
	}
	if (selectedBreakdown === 'loader') {
		return formatAnalyticsLoaderLabel(normalizedValue, formatMessage)
	}

	return breakdownValue
}

export function formatBreakdownLabels(
	breakdownValues: readonly string[],
	selectedBreakdowns: readonly AnalyticsBreakdownPreset[],
	getVersionDisplayName: ((versionId: string) => string) | undefined,
	formatMessage: FormatMessage,
): string {
	const normalizedBreakdowns = selectedBreakdowns.filter((breakdown) => breakdown !== 'none')
	const downloadReasonBreakdownIndex = normalizedBreakdowns.indexOf('download_reason')

	return collapseRepeatedUnknownBreakdownLabels(
		normalizedBreakdowns.map((breakdown, index) => {
			const breakdownValue = breakdownValues[index] ?? ''
			if (
				breakdown === 'dependent_project_download' &&
				downloadReasonBreakdownIndex !== -1 &&
				isUnknownAnalyticsBreakdownValue(breakdownValue)
			) {
				return formatAnalyticsDependentProjectFallbackLabel(
					breakdownValues[downloadReasonBreakdownIndex],
					formatMessage,
				)
			}

			return formatBreakdownLabel(breakdownValue, breakdown, getVersionDisplayName, formatMessage)
		}),
		formatMessage,
	).join(COMBINED_BREAKDOWN_LABEL_SEPARATOR)
}

function collapseRepeatedUnknownBreakdownLabels(
	labels: string[],
	formatMessage: FormatMessage,
): string[] {
	let hasUnknownLabel = false
	const collapsedLabels: string[] = []
	const unknownBreakdownLabel = formatMessage(analyticsMessages.unknown)

	for (const label of labels) {
		if (label === unknownBreakdownLabel) {
			if (hasUnknownLabel) {
				continue
			}
			hasUnknownLabel = true
		}

		collapsedLabels.push(label)
	}

	return collapsedLabels
}

export function shouldCapitalizeBreakdownLabel(
	selectedBreakdown: AnalyticsBreakdownPreset | readonly AnalyticsBreakdownPreset[],
): boolean {
	const selectedBreakdowns = Array.isArray(selectedBreakdown)
		? selectedBreakdown
		: [selectedBreakdown]
	return (
		selectedBreakdowns.length > 0 &&
		selectedBreakdowns.every(
			(breakdown) =>
				breakdown === 'download_reason' ||
				breakdown === 'monetization' ||
				breakdown === 'loader' ||
				breakdown === 'country',
		)
	)
}

function getBreakdownColor(
	breakdownValue: string,
	selectedBreakdown: AnalyticsBreakdownPreset,
	fallbackColor: string,
	palette: string[],
): string {
	if (selectedBreakdown === 'monetization') {
		const colorIndex = MONETIZATION_CHART_COLOR_INDEX[breakdownValue]
		if (colorIndex !== undefined) {
			return getPaletteColorForIndex(colorIndex, palette)
		}
	}

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

function formatDatasetTooltip(projectName: string | undefined): string | undefined {
	return projectName
}

function formatDependentProjectDatasetTooltip(
	versionName: string | undefined,
	dependentProjectName: string | undefined,
	dependencyProjectNames: readonly string[],
	formatMessage: FormatMessage,
): string | undefined {
	if (dependencyProjectNames.length === 0) {
		return undefined
	}

	if (versionName && dependentProjectName) {
		return formatMessage(analyticsChartMessages.dependentProjectVersionTooltip, {
			dependentProject: dependentProjectName,
			dependencyProject: dependencyProjectNames.join(', '),
			version: versionName,
		})
	}

	return formatMessage(analyticsChartMessages.dependentOnProjectTooltip, {
		project: dependencyProjectNames.join(', '),
	})
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
	selectedBreakdowns: readonly AnalyticsBreakdownPreset[],
	selectedFilters: AnalyticsSelectedFilters,
	dependentProjectTypesById: ReadonlyMap<string, readonly string[]>,
	projectNamesById: ReadonlyMap<string, string>,
	getVersionDisplayName: ((versionId: string) => string) | undefined,
	getVersionProjectName: ((versionId: string) => string | undefined) | undefined,
	formatMessage: FormatMessage,
	sliceCount: number = timeSlices.length,
): ChartDataset[] {
	const selectedProjectIds = new Set(selectedProjects.map((project) => project.id))
	if (selectedProjectIds.size === 0) {
		return []
	}

	const dataLength = Math.max(sliceCount, timeSlices.length)
	const normalizedBreakdowns = selectedBreakdowns.filter((breakdown) => breakdown !== 'none')
	const normalizedFilters = normalizeAnalyticsSelectedFilters(selectedFilters)

	function formatChartBreakdownLabels(breakdownValues: readonly string[]): string {
		const downloadReasonBreakdownIndex = normalizedBreakdowns.indexOf('download_reason')

		return collapseRepeatedUnknownBreakdownLabels(
			normalizedBreakdowns.map((breakdown, index) => {
				const breakdownValue = breakdownValues[index] ?? ''
				if (breakdown === 'project' || breakdown === 'dependent_project_download') {
					if (
						breakdown === 'dependent_project_download' &&
						isUnknownAnalyticsBreakdownValue(breakdownValue)
					) {
						return downloadReasonBreakdownIndex === -1
							? formatMessage(analyticsMessages.noDependent)
							: formatAnalyticsDependentProjectFallbackLabel(
									breakdownValues[downloadReasonBreakdownIndex],
									formatMessage,
								)
					}

					return projectNamesById.get(breakdownValue) ?? breakdownValue
				}

				return formatBreakdownLabel(breakdownValue, breakdown, getVersionDisplayName, formatMessage)
			}),
			formatMessage,
		).join(COMBINED_BREAKDOWN_LABEL_SEPARATOR)
	}

	if (
		normalizedBreakdowns.length > 0 &&
		!(normalizedBreakdowns.length === 1 && normalizedBreakdowns[0] === 'project')
	) {
		const hasVersionBreakdown = normalizedBreakdowns.includes('version_id')
		const hasDependentProjectBreakdown = normalizedBreakdowns.includes('dependent_project_download')
		const shouldShowDependentProjectTooltip =
			hasDependentProjectBreakdown && (selectedProjects.length > 1 || hasVersionBreakdown)
		const dataByBreakdown = new Map<string, number[]>()
		const breakdownValuesByKey = new Map<string, string[]>()
		const downloadTotalsByBreakdown = new Map<string, number>()
		const dependentOnProjectIdsByBreakdown = new Map<string, Set<string>>()

		timeSlices.forEach((slice, sliceIndex) => {
			for (const point of slice) {
				if (!isProjectAnalyticsPointInSelectedProjects(point, selectedProjectIds)) continue
				if (
					!doesAnalyticsPointMatchNormalizedFilters(
						point,
						normalizedFilters,
						dependentProjectTypesById,
					)
				) {
					continue
				}

				const breakdownValues = getAnalyticsBreakdownValues(
					point,
					normalizedBreakdowns,
					formatMessage,
				)
				if (breakdownValues.some((breakdownValue) => breakdownValue === ALL_BREAKDOWN_VALUE)) {
					continue
				}
				const breakdownKey = getAnalyticsBreakdownKey(breakdownValues)

				if (!dataByBreakdown.has(breakdownKey)) {
					dataByBreakdown.set(breakdownKey, new Array(dataLength).fill(0))
					breakdownValuesByKey.set(breakdownKey, breakdownValues)
				}
				if (shouldShowDependentProjectTooltip && point.metric_kind === 'downloads') {
					const projectIds = dependentOnProjectIdsByBreakdown.get(breakdownKey) ?? new Set<string>()
					projectIds.add(point.source_project)
					dependentOnProjectIdsByBreakdown.set(breakdownKey, projectIds)
				}

				if (point.metric_kind === 'downloads') {
					downloadTotalsByBreakdown.set(
						breakdownKey,
						(downloadTotalsByBreakdown.get(breakdownKey) ?? 0) + getMetricValue(point, 'downloads'),
					)
				}

				if (!isMetricKindForStat(point, activeStat)) continue

				const breakdownData = dataByBreakdown.get(breakdownKey)
				if (!breakdownData) continue
				breakdownData[sliceIndex] += getMetricValue(point, activeStat)
			}
		})

		const colorsByBreakdown = buildPaletteColorsByDownloadRank(
			Array.from(dataByBreakdown.keys()).map((breakdownKey) => ({
				key: breakdownKey,
				label: formatChartBreakdownLabels(breakdownValuesByKey.get(breakdownKey) ?? []),
				total: downloadTotalsByBreakdown.get(breakdownKey) ?? 0,
			})),
			palette,
		)

		return Array.from(dataByBreakdown.entries()).map(([breakdownKey, data]) => {
			const breakdownValues = breakdownValuesByKey.get(breakdownKey) ?? []
			const fallbackColor = colorsByBreakdown.get(breakdownKey) ?? ''
			const versionBreakdownIndex = normalizedBreakdowns.indexOf('version_id')
			const dependentProjectBreakdownIndex = normalizedBreakdowns.indexOf(
				'dependent_project_download',
			)
			const versionName =
				hasVersionBreakdown && versionBreakdownIndex !== -1
					? getVersionDisplayName?.(breakdownValues[versionBreakdownIndex] ?? '')
					: undefined
			const dependentProjectId =
				dependentProjectBreakdownIndex !== -1
					? breakdownValues[dependentProjectBreakdownIndex]
					: undefined
			const dependentProjectName = dependentProjectId
				? isUnknownAnalyticsBreakdownValue(dependentProjectId)
					? undefined
					: (projectNamesById.get(dependentProjectId) ?? dependentProjectId)
				: undefined
			const versionProjectName =
				normalizedBreakdowns.length === 1 && normalizedBreakdowns[0] === 'version_id'
					? getVersionProjectName?.(breakdownValues[0] ?? '')
					: undefined
			const dependencyProjectNames = [...(dependentOnProjectIdsByBreakdown.get(breakdownKey) ?? [])]
				.map((projectId) => projectNamesById.get(projectId) ?? projectId)
				.sort((left, right) => left.localeCompare(right))
			const dependentProjectTooltip = dependentProjectId
				? isUnknownAnalyticsBreakdownValue(dependentProjectId)
					? formatMessage(analyticsMessages.noDependentTooltip)
					: formatDependentProjectDatasetTooltip(
							versionName,
							dependentProjectName,
							dependencyProjectNames,
							formatMessage,
						)
				: undefined
			const color =
				normalizedBreakdowns.length === 1
					? getBreakdownColor(
							breakdownValues[0] ?? '',
							normalizedBreakdowns[0],
							fallbackColor,
							palette,
						)
					: fallbackColor
			return {
				projectId: getAnalyticsBreakdownDatasetId(breakdownValues, normalizedBreakdowns),
				label: formatChartBreakdownLabels(breakdownValues),
				projectName: versionProjectName,
				tooltip: dependentProjectTooltip ?? formatDatasetTooltip(versionProjectName),
				data,
				borderColor: color,
				backgroundColor: color,
			}
		})
	}

	if (normalizedBreakdowns.length === 0) {
		const data = new Array(dataLength).fill(0)
		let downloadTotal = 0

		timeSlices.forEach((slice, sliceIndex) => {
			for (const point of slice) {
				if (!isProjectAnalyticsPointInSelectedProjects(point, selectedProjectIds)) continue
				if (
					!doesAnalyticsPointMatchNormalizedFilters(
						point,
						normalizedFilters,
						dependentProjectTypesById,
					)
				) {
					continue
				}

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
						label: formatMessage(analyticsMessages.allProjects),
						total: downloadTotal,
					},
				],
				palette,
			).get(ALL_PROJECTS_DATASET_ID) ?? ''
		const selectedProject = selectedProjects.length === 1 ? selectedProjects[0] : undefined

		return [
			{
				projectId: ALL_PROJECTS_DATASET_ID,
				label: selectedProject?.name ?? formatMessage(analyticsMessages.allProjects),
				data,
				borderColor: color,
				backgroundColor: color,
			},
		]
	}

	const dataByProjectBreakdown = new Map<string, number[]>()
	const breakdownValuesByKey = new Map<string, string[]>()
	const downloadTotalsByProjectBreakdown = new Map<string, number>()
	for (const project of selectedProjects) {
		const breakdownValues = [project.id]
		const breakdownKey = getAnalyticsBreakdownKey(breakdownValues)
		dataByProjectBreakdown.set(breakdownKey, new Array(dataLength).fill(0))
		breakdownValuesByKey.set(breakdownKey, breakdownValues)
		downloadTotalsByProjectBreakdown.set(breakdownKey, 0)
	}

	timeSlices.forEach((slice, sliceIndex) => {
		for (const point of slice) {
			if (!isProjectAnalyticsPointInSelectedProjects(point, selectedProjectIds)) continue
			if (
				!doesAnalyticsPointMatchNormalizedFilters(
					point,
					normalizedFilters,
					dependentProjectTypesById,
				)
			) {
				continue
			}

			const breakdownValues = getAnalyticsBreakdownValues(
				point,
				normalizedBreakdowns,
				formatMessage,
			)
			if (breakdownValues.some((breakdownValue) => breakdownValue === ALL_BREAKDOWN_VALUE)) {
				continue
			}
			const breakdownKey = getAnalyticsBreakdownKey(breakdownValues)
			if (!dataByProjectBreakdown.has(breakdownKey)) {
				dataByProjectBreakdown.set(breakdownKey, new Array(dataLength).fill(0))
				breakdownValuesByKey.set(breakdownKey, breakdownValues)
				downloadTotalsByProjectBreakdown.set(breakdownKey, 0)
			}

			if (point.metric_kind === 'downloads') {
				downloadTotalsByProjectBreakdown.set(
					breakdownKey,
					(downloadTotalsByProjectBreakdown.get(breakdownKey) ?? 0) +
						getMetricValue(point, 'downloads'),
				)
			}

			if (!isMetricKindForStat(point, activeStat)) continue

			const projectData = dataByProjectBreakdown.get(breakdownKey)
			if (!projectData) continue

			projectData[sliceIndex] += getMetricValue(point, activeStat)
		}
	})

	const colorsByBreakdown = buildPaletteColorsByDownloadRank(
		Array.from(dataByProjectBreakdown.keys()).map((breakdownKey) => ({
			key: breakdownKey,
			label: formatChartBreakdownLabels(breakdownValuesByKey.get(breakdownKey) ?? []),
			total: downloadTotalsByProjectBreakdown.get(breakdownKey) ?? 0,
		})),
		palette,
	)

	return Array.from(dataByProjectBreakdown.entries()).map(([breakdownKey, data]) => {
		const breakdownValues = breakdownValuesByKey.get(breakdownKey) ?? []
		const fallbackColor = colorsByBreakdown.get(breakdownKey) ?? ''
		const versionProjectName =
			normalizedBreakdowns.length === 1 && normalizedBreakdowns[0] === 'version_id'
				? getVersionProjectName?.(breakdownValues[0] ?? '')
				: undefined
		const color =
			normalizedBreakdowns.length === 1
				? getBreakdownColor(
						breakdownValues[0] ?? '',
						normalizedBreakdowns[0],
						fallbackColor,
						palette,
					)
				: fallbackColor
		return {
			projectId: getAnalyticsBreakdownDatasetId(breakdownValues, normalizedBreakdowns),
			label: formatChartBreakdownLabels(breakdownValues),
			projectName: versionProjectName,
			tooltip: formatDatasetTooltip(versionProjectName),
			data,
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
	formatMessage: FormatMessage,
): string {
	switch (activeStat) {
		case 'revenue': {
			const amount = Math.round(value * 100) / 100
			return formatMessage(analyticsStatCardMessages.revenueValue, {
				value: formatNumber(amount),
			})
		}
		case 'playtime': {
			const hours = value / 3600
			return formatMessage(analyticsStatCardMessages.playtimeHours, {
				hours: Math.abs(hours) < 1 ? hours.toFixed(2) : hours.toFixed(1),
			})
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
	return trimTrailingFractionZeros(formattedValue)
}

function trimTrailingFractionZeros(value: string): string {
	return value.replace(/(\.\d*?)0+$/, '$1').replace(/\.$/, '')
}

const COMPACT_AXIS_UNITS = [
	{ threshold: 1_000_000, divisor: 1_000_000, suffix: 'M' },
	{ threshold: 1_000, divisor: 1_000, suffix: 'K' },
] as const
const MAX_COMPACT_AXIS_DIGITS = 3

function getCompactAxisUnit(values: readonly number[]) {
	let maxAbsoluteValue = 0
	for (const value of values) {
		if (Number.isFinite(value)) {
			maxAbsoluteValue = Math.max(maxAbsoluteValue, Math.abs(value))
		}
	}

	return COMPACT_AXIS_UNITS.find((unit) => maxAbsoluteValue >= unit.threshold) ?? null
}

function formatCompactAxisNumber(value: number, axisValues: readonly number[]): string | null {
	if (Math.abs(value) === 0) return '0'

	const unit = getCompactAxisUnit(axisValues)
	if (!unit) return null

	return `${formatCompactAxisValue(value / unit.divisor)}${unit.suffix}`
}

function formatCompactAxisValue(value: number): string {
	const absoluteValue = Math.abs(value)
	if (absoluteValue === 0) return '0'

	const integerDigitCount = absoluteValue < 1 ? 1 : Math.floor(absoluteValue).toString().length
	const fractionDigitCount = Math.max(0, MAX_COMPACT_AXIS_DIGITS - integerDigitCount)
	const roundedValue = Number(value.toFixed(fractionDigitCount))
	const roundedIntegerDigitCount =
		Math.abs(roundedValue) < 1 ? 1 : Math.floor(Math.abs(roundedValue)).toString().length

	if (roundedIntegerDigitCount > MAX_COMPACT_AXIS_DIGITS) {
		const truncatedValue = Math.sign(value) * (10 ** MAX_COMPACT_AXIS_DIGITS - 1)
		return String(truncatedValue)
	}

	return trimTrailingFractionZeros(roundedValue.toFixed(fractionDigitCount))
}

export function formatAxisValue(
	value: number,
	activeStat: AnalyticsDashboardStat,
	formatCompact: (value: number) => string,
	formatMessage: FormatMessage,
	axisValues: readonly number[] = [value],
): string {
	switch (activeStat) {
		case 'revenue': {
			const amount = Math.round(value * 100) / 100
			const axisAmounts = axisValues.map((axisValue) => Math.round(axisValue * 100) / 100)
			return formatMessage(analyticsStatCardMessages.revenueValue, {
				value: formatCompactAxisNumber(amount, axisAmounts) ?? formatCompact(amount),
			})
		}
		case 'playtime': {
			const formattedHours = formatCompactAxisNumber(value, axisValues)
			if (formattedHours) {
				return formatMessage(analyticsChartMessages.playtimeAxisHours, { hours: formattedHours })
			}
			if (Math.abs(value) < 10) {
				return formatMessage(analyticsChartMessages.playtimeAxisHours, {
					hours: formatSmallAxisNumber(value),
				})
			}
			return formatMessage(analyticsChartMessages.playtimeAxisHours, {
				hours: formatCompact(Math.round(value)),
			})
		}
		case 'views':
		case 'downloads':
		default: {
			const roundedValue = Math.round(value)
			const roundedAxisValues = axisValues.map((axisValue) => Math.round(axisValue))
			const formattedValue = formatCompactAxisNumber(roundedValue, roundedAxisValues)
			if (formattedValue) return formattedValue
			if (Math.abs(value) < 10) {
				return formatSmallAxisNumber(value)
			}
			return formatCompact(roundedValue)
		}
	}
}
