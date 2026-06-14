import type { Labrinth } from '@modrinth/api-client'

import type { AnalyticsBreakdownPreset } from '~/providers/analytics/analytics'

import { formatAnalyticsDownloadSourceLabel, type FormatMessage } from './analytics-messages'

export const ALL_BREAKDOWN_VALUE = '__all__'
export const UNKNOWN_BREAKDOWN_VALUE = '__unknown__'
export const COMBINED_BREAKDOWN_LABEL_SEPARATOR = ' + '
export const COMBINED_BREAKDOWN_DATASET_ID_PREFIX = 'breakdowns:'

export function getAnalyticsBreakdownValue(
	point: Labrinth.Analytics.v3.ProjectAnalytics,
	selectedBreakdown: AnalyticsBreakdownPreset,
	formatMessage: FormatMessage,
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
			const downloadSource = normalizeBreakdownValue(
				'user_agent' in point ? point.user_agent : undefined,
				UNKNOWN_BREAKDOWN_VALUE,
			)
			return downloadSource === UNKNOWN_BREAKDOWN_VALUE
				? UNKNOWN_BREAKDOWN_VALUE
				: getDownloadSourceLabel(downloadSource, formatMessage)
		}
		case 'download_reason':
			return normalizeBreakdownValue(
				'reason' in point ? point.reason : undefined,
				UNKNOWN_BREAKDOWN_VALUE,
			)
		case 'dependent_project_download':
			return normalizeBreakdownValue(
				'dependent_project_id' in point ? point.dependent_project_id : undefined,
			)
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

function normalizeBreakdownValue(
	value: string | undefined,
	fallback = ALL_BREAKDOWN_VALUE,
): string {
	const normalized = value?.trim()
	const normalizedLowercase = normalized?.toLowerCase()
	if (
		fallback === UNKNOWN_BREAKDOWN_VALUE &&
		(normalizedLowercase === 'unknown' || normalizedLowercase === 'other')
	) {
		return fallback
	}
	return normalized && normalized.length > 0 ? normalized : fallback
}
