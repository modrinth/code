import type { AnalyticsGroupByPreset } from '~/providers/analytics/analytics'

const SECONDS_PER_MINUTE = 60
const SECONDS_PER_HOUR = 60 * SECONDS_PER_MINUTE

export function getAnalyticsTableGroupByLabel(groupBy: AnalyticsGroupByPreset): string {
	switch (groupBy) {
		case '1h':
			return '1h'
		case '6h':
			return '6h'
		case 'day':
			return 'Day'
		case 'week':
			return 'Week'
		case 'month':
			return 'Month'
		case 'year':
			return 'Year'
		default:
			return 'Date'
	}
}

export function formatAnalyticsTableInteger(
	formatNumber: (value: number) => string,
	value: number,
): string {
	return formatNumber(Math.round(value))
}

export function formatAnalyticsTableRevenue(formatter: Intl.NumberFormat, value: number): string {
	const rounded = Math.round(value * 100) / 100
	return `$${formatter.format(rounded)}`
}

export function formatAnalyticsTableCompactPlaytime(value: number): string {
	const totalSeconds = Math.max(0, Math.round(value))
	return `${(totalSeconds / SECONDS_PER_HOUR).toLocaleString(undefined, {
		minimumFractionDigits: 1,
		maximumFractionDigits: 1,
	})}h`
}

export function formatAnalyticsTableFullPlaytime(
	formatNumber: (value: number) => string,
	value: number,
): string {
	const totalMinutes = Math.max(0, Math.round(value / SECONDS_PER_MINUTE))
	const days = Math.floor(totalMinutes / (24 * 60))
	const hours = Math.floor((totalMinutes % (24 * 60)) / 60)
	const minutes = totalMinutes % 60

	return [
		formatAnalyticsTableDurationTooltipPart(formatNumber, days, 'day'),
		formatAnalyticsTableDurationTooltipPart(formatNumber, hours, 'hour'),
		formatAnalyticsTableDurationTooltipPart(formatNumber, minutes, 'minute'),
	].join(', ')
}

function formatAnalyticsTableDurationTooltipPart(
	formatNumber: (value: number) => string,
	value: number,
	unit: string,
): string {
	return `${formatNumber(value)} ${unit}${value === 1 ? '' : 's'}`
}
