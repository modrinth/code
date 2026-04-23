import type { Labrinth } from '@modrinth/api-client'

import type {
	AnalyticsBreakdownPreset,
	AnalyticsDashboardProject,
	AnalyticsDashboardStat,
} from '~/providers/analytics/analytics'

import { getAnalyticsBreakdownValue } from '../breakdown'

export type ChartDataset = {
	projectId: string
	label: string
	data: number[]
	borderColor: string
	backgroundColor: string
}

const DAY_MS = 24 * 60 * 60 * 1000

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
				label: breakdownValue,
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
): string[] {
	const startMs = new Date(timeRange.start).getTime()
	const endMs = new Date(timeRange.end).getTime()
	const totalMs = endMs - startMs
	const bucketMs = sliceCount > 0 ? totalMs / sliceCount : 0
	const formatter = getTickFormatter(totalMs)

	const labels: string[] = []
	for (let i = 0; i < sliceCount; i++) {
		labels.push(formatter.format(new Date(startMs + i * bucketMs)))
	}
	return labels
}

function getTickFormatter(totalMs: number): Intl.DateTimeFormat {
	if (totalMs <= 2 * DAY_MS) {
		return new Intl.DateTimeFormat(undefined, { hour: 'numeric', minute: '2-digit' })
	}
	if (totalMs < 31 * DAY_MS) {
		return new Intl.DateTimeFormat(undefined, {
			month: 'short',
			day: 'numeric',
			hour: 'numeric',
			minute: '2-digit',
		})
	}
	return new Intl.DateTimeFormat(undefined, { month: 'short', day: 'numeric' })
}

export function formatBucketRange(start: Date, end: Date): string {
	const timeFormatter = new Intl.DateTimeFormat(undefined, {
		hour: 'numeric',
		minute: '2-digit',
	})
	const dateTimeFormatter = new Intl.DateTimeFormat(undefined, {
		month: 'short',
		day: 'numeric',
		hour: 'numeric',
		minute: '2-digit',
	})

	const sameDay =
		start.getFullYear() === end.getFullYear() &&
		start.getMonth() === end.getMonth() &&
		start.getDate() === end.getDate()

	if (sameDay) {
		return `${timeFormatter.format(start)} – ${timeFormatter.format(end)}`
	}
	return `${dateTimeFormatter.format(start)} – ${dateTimeFormatter.format(end)}`
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
