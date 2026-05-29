import type { AnalyticsGroupByPreset } from '~/providers/analytics/analytics'

import {
	analyticsStatCardMessages,
	analyticsTableMessages,
	formatAnalyticsGroupByLabel,
	type FormatMessage,
} from '../analytics-messages'

const SECONDS_PER_MINUTE = 60
const SECONDS_PER_HOUR = 60 * SECONDS_PER_MINUTE

export function getAnalyticsTableGroupByLabel(
	groupBy: AnalyticsGroupByPreset,
	formatMessage: FormatMessage,
): string {
	return formatAnalyticsGroupByLabel(groupBy, formatMessage)
}

export function formatAnalyticsTableInteger(
	formatNumber: (value: number) => string,
	value: number,
): string {
	return formatNumber(Math.round(value))
}

export function formatAnalyticsTableRevenue(
	formatter: Intl.NumberFormat,
	value: number,
	formatMessage: FormatMessage,
): string {
	const rounded = Math.round(value * 100) / 100
	return formatMessage(analyticsStatCardMessages.revenueValue, {
		value: formatter.format(rounded),
	})
}

export function formatAnalyticsTableCompactPlaytime(
	value: number,
	formatMessage: FormatMessage,
): string {
	const totalSeconds = Math.max(0, Math.round(value))
	return formatMessage(analyticsStatCardMessages.playtimeHours, {
		hours: (totalSeconds / SECONDS_PER_HOUR).toLocaleString(undefined, {
			minimumFractionDigits: 1,
			maximumFractionDigits: 1,
		}),
	})
}

export function formatAnalyticsTableFullPlaytime(
	value: number,
	formatMessage: FormatMessage,
): string {
	const totalMinutes = Math.max(0, Math.round(value / SECONDS_PER_MINUTE))
	const days = Math.floor(totalMinutes / (24 * 60))
	const hours = Math.floor((totalMinutes % (24 * 60)) / 60)
	const minutes = totalMinutes % 60

	return [
		formatMessage(analyticsTableMessages.durationDays, { count: days }),
		formatMessage(analyticsTableMessages.durationHours, { count: hours }),
		formatMessage(analyticsTableMessages.durationMinutes, { count: minutes }),
	].join(', ')
}
