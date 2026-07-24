import type {
	AnalyticsBreakdownGroup,
	AnalyticsBreakdownPreset,
} from '~/providers/analytics/analytics'

export const ANALYTICS_BREAKDOWN_GROUP_OTHER_SERIES_ID = '__other__'
const GROUPED_BREAKDOWN_VALUE_PREFIX = '__breakdown_group__:'

export function createAnalyticsBreakdownGroupId(): string {
	if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
		return crypto.randomUUID()
	}
	return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`
}

export function getAnalyticsBreakdownGroupValue(groupId: string, seriesId: string): string {
	return `${GROUPED_BREAKDOWN_VALUE_PREFIX}${encodeURIComponent(groupId)}:${encodeURIComponent(seriesId)}`
}

export function getAnalyticsBreakdownGroupSeriesId(value: string): string | null {
	if (!value.startsWith(GROUPED_BREAKDOWN_VALUE_PREFIX)) return null
	const encodedParts = value.slice(GROUPED_BREAKDOWN_VALUE_PREFIX.length).split(':')
	if (encodedParts.length !== 2) return null
	try {
		return decodeURIComponent(encodedParts[1] ?? '') || null
	} catch {
		return null
	}
}

export function getAnalyticsBreakdownGroupSeriesName(
	value: string,
	group: AnalyticsBreakdownGroup | null | undefined,
	otherLabel: string,
): string | undefined {
	if (!group) return undefined
	const seriesId = getAnalyticsBreakdownGroupSeriesId(value)
	if (!seriesId) return undefined
	if (seriesId === ANALYTICS_BREAKDOWN_GROUP_OTHER_SERIES_ID) return otherLabel
	return group.series.find((series) => series.id === seriesId)?.name
}

export function applyAnalyticsBreakdownGroup(
	values: readonly string[],
	breakdowns: readonly AnalyticsBreakdownPreset[],
	group: AnalyticsBreakdownGroup | null | undefined,
): string[] {
	const nextValues = [...values]
	if (!group) return nextValues
	const breakdownIndex = breakdowns.findIndex((breakdown) => breakdown === group.breakdown)
	if (breakdownIndex === -1) return nextValues
	const rawValue = nextValues[breakdownIndex]
	if (!rawValue) return nextValues
	const series = group.series.find((candidate) => candidate.values.includes(rawValue))
	nextValues[breakdownIndex] = getAnalyticsBreakdownGroupValue(
		group.id,
		series?.id ?? ANALYTICS_BREAKDOWN_GROUP_OTHER_SERIES_ID,
	)
	return nextValues
}
