import type { Labrinth } from '@modrinth/api-client'

import type { AnalyticsBreakdownPreset } from '~/providers/analytics/analytics'

export const ALL_BREAKDOWN_VALUE = 'All'
export const UNKNOWN_BREAKDOWN_VALUE = 'Unknown'

export function getAnalyticsBreakdownValue(
	point: Labrinth.Analytics.v3.ProjectAnalytics,
	selectedBreakdown: AnalyticsBreakdownPreset,
): string {
	switch (selectedBreakdown) {
		case 'none':
			return ALL_BREAKDOWN_VALUE
		case 'country':
			return normalizeBreakdownValue('country' in point ? point.country?.toUpperCase() : undefined)
		case 'monetization': {
			if ('monetized' in point && typeof point.monetized === 'boolean') {
				return point.monetized ? 'monetized' : 'unmonetized'
			}
			return ALL_BREAKDOWN_VALUE
		}
		case 'user_agent':
			return getDownloadSourceLabel(
				normalizeBreakdownValue('user_agent' in point ? point.user_agent : undefined),
			)
		case 'download_reason':
			return normalizeBreakdownValue(
				'reason' in point ? point.reason : undefined,
				UNKNOWN_BREAKDOWN_VALUE,
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

export function getDownloadSourceLabel(value: string): string {
	const normalized = value.trim()
	const normalizedLowercase = normalized.toLowerCase()
	if (normalizedLowercase === 'website') {
		return 'Modrinth Website'
	}
	if (normalizedLowercase === 'modrinth_app') {
		return 'Modrinth App'
	}
	if (!normalized.includes('_')) {
		return normalized
	}

	return normalizedLowercase
		.split('_')
		.filter((part) => part.length > 0)
		.map((part) => `${part.charAt(0).toUpperCase()}${part.slice(1)}`)
		.join(' ')
}

function normalizeBreakdownValue(
	value: string | undefined,
	fallback = ALL_BREAKDOWN_VALUE,
): string {
	const normalized = value?.trim()
	return normalized && normalized.length > 0 ? normalized : fallback
}
