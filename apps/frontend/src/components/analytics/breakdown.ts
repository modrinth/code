import type { Labrinth } from '@modrinth/api-client'

import type { AnalyticsBreakdownPreset } from '~/providers/analytics/analytics'

export const ALL_BREAKDOWN_VALUE = 'All'

export function getAnalyticsBreakdownValue(
	point: Labrinth.Analytics.v3.ProjectAnalytics,
	selectedBreakdown: AnalyticsBreakdownPreset,
): string {
	switch (selectedBreakdown) {
		case 'none':
			return ALL_BREAKDOWN_VALUE
		case 'country':
			return normalizeBreakdownValue('country' in point ? point.country : undefined)
		case 'monetization': {
			if ('monetized' in point && typeof point.monetized === 'boolean') {
				return point.monetized ? 'monetized' : 'unmonetized'
			}
			return ALL_BREAKDOWN_VALUE
		}
		case 'download_source':
			return normalizeBreakdownValue('domain' in point ? point.domain : undefined)
		case 'download_type':
			return normalizeBreakdownValue('version_id' in point ? point.version_id : undefined)
		case 'loader':
			return normalizeBreakdownValue('loader' in point ? point.loader : undefined)
		case 'game_version':
			return normalizeBreakdownValue('game_version' in point ? point.game_version : undefined)
		default:
			return ALL_BREAKDOWN_VALUE
	}
}

function normalizeBreakdownValue(value: string | undefined): string {
	const normalized = value?.trim()
	return normalized && normalized.length > 0 ? normalized : ALL_BREAKDOWN_VALUE
}
