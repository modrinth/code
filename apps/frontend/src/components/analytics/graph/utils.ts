import type { Labrinth } from '@modrinth/api-client'

import {
	type AnalyticsBreakdownPreset,
	type AnalyticsDashboardProject,
	type AnalyticsDashboardStat,
	type AnalyticsGroupByPreset,
	type AnalyticsSelectedFilters,
	doesAnalyticsPointMatchFilters,
} from '~/providers/analytics/analytics'

import { getAnalyticsBreakdownValue } from '../breakdown'

export type ChartDataset = {
	projectId: string
	label: string
	data: number[]
	borderColor: string
	backgroundColor: string
}

const REGION_CODE_PATTERN = /^[a-z]{2}$/i
const OTHER_COUNTRY_CODE = 'XX'
const OTHER_COUNTRY_LABEL = 'Other'
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
		return OTHER_COUNTRY_LABEL
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

function formatLoaderLabel(loader: string): string {
	const normalized = loader.trim()
	if (normalized.length === 0) {
		return loader
	}

	return `${normalized[0].toUpperCase()}${normalized.slice(1)}`
}

export function formatBreakdownLabel(
	breakdownValue: string,
	selectedBreakdown: AnalyticsBreakdownPreset,
	getVersionDisplayName: (versionId: string) => string = (versionId) => versionId,
): string {
	if (selectedBreakdown === 'country') {
		return formatCountryCode(breakdownValue)
	}
	if (selectedBreakdown === 'version_id') {
		return getVersionDisplayName(breakdownValue)
	}
	if (selectedBreakdown === 'loader') {
		return formatLoaderLabel(breakdownValue)
	}

	return breakdownValue
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

export function buildChartDatasets(
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
	selectedProjects: AnalyticsDashboardProject[],
	activeStat: AnalyticsDashboardStat,
	palette: string[],
	selectedBreakdown: AnalyticsBreakdownPreset,
	selectedFilters: AnalyticsSelectedFilters,
	getVersionDisplayName: (versionId: string) => string = (versionId) => versionId,
): ChartDataset[] {
	const selectedProjectIds = new Set(selectedProjects.map((project) => project.id))
	if (selectedProjectIds.size === 0) {
		return []
	}

	if (selectedBreakdown !== 'none') {
		const dataByBreakdown = new Map<string, number[]>()

		timeSlices.forEach((slice, sliceIndex) => {
			for (const point of slice) {
				if (!('source_project' in point)) continue
				if (!selectedProjectIds.has(point.source_project)) continue
				if (!doesAnalyticsPointMatchFilters(point, selectedFilters)) continue

				const value = getMetricValue(point, activeStat)
				if (value === 0) continue

				const breakdownValue = getAnalyticsBreakdownValue(point, selectedBreakdown)

				let breakdownData = dataByBreakdown.get(breakdownValue)
				if (!breakdownData) {
					breakdownData = new Array(timeSlices.length).fill(0)
					dataByBreakdown.set(breakdownValue, breakdownData)
				}

				breakdownData[sliceIndex] += value
			}
		})

		return Array.from(dataByBreakdown.entries()).map(([breakdownValue, data], index) => {
			const color = palette[index % palette.length]
			return {
				projectId: `breakdown:${breakdownValue}`,
				label: formatBreakdownLabel(breakdownValue, selectedBreakdown, getVersionDisplayName),
				data,
				borderColor: color,
				backgroundColor: color,
			}
		})
	}

	const dataByProjectId = new Map<string, number[]>()
	for (const project of selectedProjects) {
		dataByProjectId.set(project.id, new Array(timeSlices.length).fill(0))
	}

	timeSlices.forEach((slice, sliceIndex) => {
		for (const point of slice) {
			if (!('source_project' in point)) continue
			if (!selectedProjectIds.has(point.source_project)) continue
			if (!doesAnalyticsPointMatchFilters(point, selectedFilters)) continue

			const projectData = dataByProjectId.get(point.source_project)
			if (!projectData) continue

			projectData[sliceIndex] += getMetricValue(point, activeStat)
		}
	})

	return selectedProjects.map((project, index) => {
		const color = palette[index % palette.length]
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

export function buildTimeAxisLabels(
	timeRange: Labrinth.Analytics.v3.TimeRange,
	sliceCount: number,
	includeTime: boolean,
): string[] {
	const startMs = new Date(timeRange.start).getTime()
	const endMs = new Date(timeRange.end).getTime()
	const totalMs = endMs - startMs
	const bucketMs = sliceCount > 0 ? totalMs / sliceCount : 0
	const formatter = getBucketEndFormatter(includeTime)

	const labels: string[] = []
	for (let i = 0; i < sliceCount; i++) {
		labels.push(formatter.format(new Date(startMs + (i + 1) * bucketMs)))
	}
	return labels
}

function getBucketEndFormatter(includeTime: boolean): Intl.DateTimeFormat {
	if (includeTime) {
		return new Intl.DateTimeFormat(undefined, {
			month: 'short',
			day: 'numeric',
			hour: 'numeric',
			minute: '2-digit',
		})
	}
	return new Intl.DateTimeFormat(undefined, { month: 'short', day: 'numeric' })
}

export function isTimeRelevantForGroupBy(groupBy: AnalyticsGroupByPreset): boolean {
	return groupBy === '1h' || groupBy === '6h'
}

export function formatBucketEndLabel(end: Date, includeTime: boolean): string {
	if (includeTime) {
		return new Intl.DateTimeFormat(undefined, {
			month: 'short',
			day: 'numeric',
			hour: 'numeric',
			minute: '2-digit',
		}).format(end)
	}

	return new Intl.DateTimeFormat(undefined, {
		month: 'short',
		day: 'numeric',
	}).format(end)
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
			return formatCompact(Math.round(value))
	}
}
