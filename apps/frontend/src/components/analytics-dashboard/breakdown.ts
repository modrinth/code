import type { Labrinth } from '@modrinth/api-client'

import type { AnalyticsBreakdownPreset } from '~/providers/analytics/analytics'

import { formatAnalyticsDownloadSourceLabel, type FormatMessage } from './analytics-messages'

export const ALL_BREAKDOWN_VALUE = '__all__'
export const UNKNOWN_BREAKDOWN_VALUE = '__unknown__'
export const NO_DEPENDENT_BREAKDOWN_VALUE = '__no_dependent__'
export const COMBINED_BREAKDOWN_LABEL_SEPARATOR = ' + '
export const COMBINED_BREAKDOWN_DATASET_ID_PREFIX = 'breakdowns:'

export function getAnalyticsBreakdownValue(
	point: Labrinth.Analytics.v3.ProjectAnalytics,
	selectedBreakdown: AnalyticsBreakdownPreset,
	_formatMessage: FormatMessage,
): string {
	switch (selectedBreakdown) {
		case 'none':
			return ALL_BREAKDOWN_VALUE
		case 'project':
			return normalizeBreakdownValue('source_project' in point ? point.source_project : undefined)
		case 'country':
			return normalizeBreakdownValue('country' in point ? point.country?.toUpperCase() : undefined)
		case 'monetization': {
			if ('monetized' in point && typeof point.monetized === 'boolean') {
				return point.monetized ? 'monetized' : 'unmonetized'
			}
			return ALL_BREAKDOWN_VALUE
		}
		case 'user_agent': {
			return normalizeBreakdownValue(
				'user_agent' in point ? point.user_agent : undefined,
				UNKNOWN_BREAKDOWN_VALUE,
			)
		}
		case 'download_reason':
			return normalizeBreakdownValue(
				'reason' in point ? point.reason : undefined,
				UNKNOWN_BREAKDOWN_VALUE,
			)
		case 'user_id':
			return normalizeBreakdownValue('user_id' in point ? point.user_id : undefined)
		case 'dependent_project_download': {
			const dependentProjectId = normalizeBreakdownValue(
				'dependent_project_id' in point ? point.dependent_project_id : undefined,
				UNKNOWN_BREAKDOWN_VALUE,
			)
			const downloadReason = 'reason' in point ? point.reason?.trim().toLowerCase() : undefined
			return dependentProjectId === UNKNOWN_BREAKDOWN_VALUE &&
				(downloadReason === 'standalone' || downloadReason === 'update')
				? NO_DEPENDENT_BREAKDOWN_VALUE
				: dependentProjectId
		}
		case 'version_id':
			return normalizeBreakdownValue('version_id' in point ? point.version_id : undefined)
		case 'loader':
			return normalizeBreakdownValue(
				'loader' in point ? point.loader : undefined,
				UNKNOWN_BREAKDOWN_VALUE,
			)
		case 'game_version':
			return normalizeBreakdownValue(
				'game_version' in point ? point.game_version : undefined,
				UNKNOWN_BREAKDOWN_VALUE,
			)
		default:
			return ALL_BREAKDOWN_VALUE
	}
}

export function getAnalyticsBreakdownValues(
	point: Labrinth.Analytics.v3.ProjectAnalytics,
	selectedBreakdowns: readonly AnalyticsBreakdownPreset[],
	formatMessage: FormatMessage,
): string[] {
	return selectedBreakdowns
		.filter((breakdown) => breakdown !== 'none')
		.map((breakdown) => getAnalyticsBreakdownValue(point, breakdown, formatMessage))
}

export function getAnalyticsBreakdownKey(values: readonly string[]): string {
	return values.map((value) => encodeURIComponent(value)).join('+')
}

export function getAnalyticsBreakdownDatasetId(
	values: readonly string[],
	selectedBreakdowns: readonly AnalyticsBreakdownPreset[],
): string {
	const normalizedBreakdowns = selectedBreakdowns.filter((breakdown) => breakdown !== 'none')
	if (normalizedBreakdowns.length === 0) {
		return 'all'
	}
	if (normalizedBreakdowns.length === 1) {
		if (normalizedBreakdowns[0] === 'project') {
			return values[0] ?? 'all'
		}
		return `breakdown:${values[0] ?? 'all'}`
	}

	return `${COMBINED_BREAKDOWN_DATASET_ID_PREFIX}${getAnalyticsBreakdownKey(values)}`
}

export function getDownloadSourceLabel(value: string, formatMessage: FormatMessage): string {
	return formatAnalyticsDownloadSourceLabel(value, formatMessage)
}

export function isUnknownAnalyticsBreakdownValue(value: string | null | undefined): boolean {
	const normalized = value?.trim()
	if (!normalized) {
		return false
	}

	const normalizedLowercase = normalized.toLowerCase()
	return (
		normalized === UNKNOWN_BREAKDOWN_VALUE ||
		normalizedLowercase === 'unknown' ||
		normalizedLowercase === 'other'
	)
}

export function isNoDependentAnalyticsBreakdownValue(value: string | null | undefined): boolean {
	return value?.trim() === NO_DEPENDENT_BREAKDOWN_VALUE
}

function normalizeBreakdownValue(
	value: string | undefined,
	fallback = ALL_BREAKDOWN_VALUE,
): string {
	const normalized = value?.trim()
	if (fallback === UNKNOWN_BREAKDOWN_VALUE && isUnknownAnalyticsBreakdownValue(normalized)) {
		return fallback
	}
	return normalized && normalized.length > 0 ? normalized : fallback
}
