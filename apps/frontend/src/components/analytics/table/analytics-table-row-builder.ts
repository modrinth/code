import type { Labrinth } from '@modrinth/api-client'

import type {
	AnalyticsBreakdownPreset,
	AnalyticsDashboardStat,
} from '~/providers/analytics/analytics'

import type { FormatMessage } from '../analytics-messages'
import {
	ALL_BREAKDOWN_VALUE,
	COMBINED_BREAKDOWN_LABEL_SEPARATOR,
	getAnalyticsBreakdownDatasetId,
	getAnalyticsBreakdownKey,
	getAnalyticsBreakdownValues,
} from '../breakdown'
import {
	formatBreakdownLabel,
	formatBucketEndLabel,
	getSliceBucketRange,
	getSliceCount,
} from '../chart/analytics-chart-utils'
import { getAnalyticsTableBreakdownColumnKey } from './analytics-table-columns'
import type {
	AnalyticsTableBreakdownDisplayValues,
	AnalyticsTableBreakdownPreset,
	AnalyticsTableMode,
	AnalyticsTableRow,
} from './analytics-table-types'

const ALL_PROJECTS_BREAKDOWN_VALUE = 'all'

type BuildAnalyticsTableRowsOptions = {
	mode: AnalyticsTableMode
	fetchRequest: Labrinth.Analytics.v3.FetchRequest | null
	timeSlices: Labrinth.Analytics.v3.TimeSlice[]
	selectedBreakdowns: readonly AnalyticsTableBreakdownPreset[]
	selectedProjectIds: ReadonlySet<string>
	relevantStats: ReadonlySet<AnalyticsDashboardStat>
	projectNamesById: ReadonlyMap<string, string>
	getVersionDisplayName: (versionId: string) => string
	getVersionProjectName: (versionId: string) => string | undefined
	showTimeInBucketLabel: boolean
	showYearInBucketLabel: boolean
	formatMessage: FormatMessage
}

export function buildAnalyticsTableRows({
	mode,
	fetchRequest,
	timeSlices,
	selectedBreakdowns,
	selectedProjectIds,
	relevantStats,
	projectNamesById,
	getVersionDisplayName,
	getVersionProjectName,
	showTimeInBucketLabel,
	showYearInBucketLabel,
	formatMessage,
}: BuildAnalyticsTableRowsOptions): AnalyticsTableRow[] {
	if (!fetchRequest || selectedProjectIds.size === 0) {
		return []
	}

	const timeRange = fetchRequest.time_range
	const sliceCount = getSliceCount(timeRange, timeSlices.length)
	const includeDate = mode === 'date_breakdown'
	const breakdownDisplayValues = new Map<string, string>()
	const projectDisplayValues = new Map<string, string>()
	const nextRows = new Map<string, AnalyticsTableRow>()
	const bucketLabelsBySliceIndex = new Map<number, { date: string; dateMs: number }>()

	function getBreakdownDisplayValue(
		breakdownValue: string,
		breakdown: AnalyticsTableBreakdownPreset,
	) {
		const key = `${breakdown}:${breakdownValue}`
		let displayValue = breakdownDisplayValues.get(key)
		if (displayValue === undefined) {
			displayValue = formatAnalyticsTableBreakdownDisplayValue(
				breakdownValue,
				breakdown,
				projectNamesById,
				getVersionDisplayName,
				formatMessage,
			)
			breakdownDisplayValues.set(key, displayValue)
		}
		return displayValue
	}

	function getProjectDisplayValueForBreakdownValues(breakdownValues: readonly string[]) {
		const versionBreakdownIndex = selectedBreakdowns.indexOf('version_id')
		if (versionBreakdownIndex === -1 || selectedBreakdowns.includes('project')) {
			return ''
		}

		const versionId = breakdownValues[versionBreakdownIndex]
		if (!versionId) {
			return ''
		}

		let displayValue = projectDisplayValues.get(versionId)
		if (displayValue === undefined) {
			displayValue = getVersionProjectName(versionId) ?? ''
			projectDisplayValues.set(versionId, displayValue)
		}
		return displayValue
	}

	function getBreakdownDisplays(breakdownValues: readonly string[]) {
		const displays: AnalyticsTableBreakdownDisplayValues = {}

		selectedBreakdowns.forEach((breakdown, index) => {
			displays[breakdown] = getBreakdownDisplayValue(breakdownValues[index] ?? '', breakdown)
		})

		return displays
	}

	function getCombinedBreakdownDisplay(displays: AnalyticsTableBreakdownDisplayValues) {
		return selectedBreakdowns
			.map((breakdown) => displays[breakdown])
			.filter((displayValue): displayValue is string => Boolean(displayValue))
			.join(COMBINED_BREAKDOWN_LABEL_SEPARATOR)
	}

	function getBucketLabel(sliceIndex: number) {
		let bucketLabel = bucketLabelsBySliceIndex.get(sliceIndex)
		if (!bucketLabel) {
			const bucketRange = getSliceBucketRange(timeRange, sliceCount, sliceIndex)
			bucketLabel = {
				date: formatBucketEndLabel(bucketRange.end, showTimeInBucketLabel, showYearInBucketLabel),
				dateMs: bucketRange.end.getTime(),
			}
			bucketLabelsBySliceIndex.set(sliceIndex, bucketLabel)
		}
		return bucketLabel
	}

	function createRow(
		rowId: string,
		breakdownValues: readonly string[],
		bucketLabel?: { date: string; dateMs: number },
	) {
		const breakdownKey =
			breakdownValues.length === 0
				? ALL_PROJECTS_BREAKDOWN_VALUE
				: getAnalyticsBreakdownKey(breakdownValues)
		const breakdownDisplays = getBreakdownDisplays(breakdownValues)
		const row: AnalyticsTableRow = {
			id: rowId,
			date: bucketLabel?.date ?? '',
			dateMs: bucketLabel?.dateMs ?? 0,
			project: getProjectDisplayValueForBreakdownValues(breakdownValues),
			breakdown: breakdownKey,
			breakdownValues: Object.fromEntries(
				selectedBreakdowns.map((breakdown, index) => [breakdown, breakdownValues[index] ?? '']),
			) as AnalyticsTableBreakdownDisplayValues,
			breakdownDisplays,
			graphDatasetId: getAnalyticsTableGraphDatasetId(breakdownValues, selectedBreakdowns),
			breakdownDisplay: getCombinedBreakdownDisplay(breakdownDisplays),
			views: 0,
			downloads: 0,
			revenue: 0,
			playtime: 0,
		}

		for (const breakdown of selectedBreakdowns) {
			row[getAnalyticsTableBreakdownColumnKey(breakdown)] = breakdownDisplays[breakdown] ?? ''
		}

		nextRows.set(rowId, row)
		return row
	}

	if (!includeDate && selectedBreakdowns.length === 0) {
		createRow(ALL_PROJECTS_BREAKDOWN_VALUE, [])
	}

	if (!includeDate && selectedBreakdowns.length === 1 && selectedBreakdowns[0] === 'project') {
		for (const projectId of selectedProjectIds) {
			createRow(projectId, [projectId])
		}
	}

	timeSlices.forEach((slice, sliceIndex) => {
		const bucketLabel = includeDate ? getBucketLabel(sliceIndex) : undefined

		for (const point of slice) {
			if (!isProjectAnalyticsPoint(point)) {
				continue
			}

			if (!selectedProjectIds.has(point.source_project)) {
				continue
			}

			const pointStat = getAnalyticsTableStatForMetric(point.metric_kind)
			if (!pointStat || !relevantStats.has(pointStat)) {
				continue
			}

			const breakdownValues =
				selectedBreakdowns.length === 0
					? []
					: getAnalyticsBreakdownValues(point, selectedBreakdowns, formatMessage)
			if (breakdownValues.some((breakdownValue) => breakdownValue === ALL_BREAKDOWN_VALUE)) {
				continue
			}

			const nextBucketLabel = includeDate ? (bucketLabel ?? getBucketLabel(sliceIndex)) : undefined
			const breakdownKey =
				breakdownValues.length === 0
					? ALL_PROJECTS_BREAKDOWN_VALUE
					: getAnalyticsBreakdownKey(breakdownValues)
			const rowId = includeDate ? `${nextBucketLabel?.dateMs ?? 0}::${breakdownKey}` : breakdownKey
			const row = nextRows.get(rowId) ?? createRow(rowId, breakdownValues, nextBucketLabel)
			addAnalyticsMetricToTableRow(row, point)
		}
	})

	return Array.from(nextRows.values())
}

function isProjectAnalyticsPoint(
	point: Labrinth.Analytics.v3.AnalyticsData,
): point is Labrinth.Analytics.v3.ProjectAnalytics {
	return 'source_project' in point
}

function addAnalyticsMetricToTableRow(
	row: AnalyticsTableRow,
	point: Labrinth.Analytics.v3.ProjectAnalytics,
) {
	switch (point.metric_kind) {
		case 'views':
			row.views += point.views
			break
		case 'downloads':
			row.downloads += point.downloads
			break
		case 'playtime':
			row.playtime += point.seconds
			break
		case 'revenue': {
			const parsed = Number.parseFloat(point.revenue)
			row.revenue += Number.isFinite(parsed) ? parsed : 0
			break
		}
	}
}

function getAnalyticsTableStatForMetric(
	metricKind: Labrinth.Analytics.v3.ProjectAnalytics['metric_kind'],
): AnalyticsDashboardStat | null {
	switch (metricKind) {
		case 'views':
		case 'downloads':
		case 'revenue':
		case 'playtime':
			return metricKind
		default:
			return null
	}
}

function getAnalyticsTableGraphDatasetId(
	breakdownValues: readonly string[],
	selectedBreakdowns: readonly AnalyticsBreakdownPreset[],
): string {
	return getAnalyticsBreakdownDatasetId(breakdownValues, selectedBreakdowns)
}

function formatAnalyticsTableBreakdownDisplayValue(
	value: string,
	breakdown: AnalyticsTableBreakdownPreset,
	projectNamesById: ReadonlyMap<string, string>,
	getVersionDisplayName: (versionId: string) => string,
	formatMessage: FormatMessage,
): string {
	if (breakdown === 'project') {
		return projectNamesById.get(value) ?? value
	}
	return formatBreakdownLabel(value, breakdown, getVersionDisplayName, formatMessage)
}
