import {
	type AnalyticsGroupByPreset,
	type AnalyticsLastTimeframeUnit,
	type AnalyticsTimeframeMode,
	type AnalyticsTimeframePreset,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

const MIN_RANGE_MS = 60 * 60 * 1000
const TIME_RANGE_ROUNDING_MS = 60 * 1000
export const MAX_ANALYTICS_TIME_SLICES = 256

const GROUP_BY_PRESET_MINUTES: Record<AnalyticsGroupByPreset, number> = {
	'1h': 60,
	'6h': 360,
	day: 24 * 60,
	week: 7 * 24 * 60,
	month: 30 * 24 * 60,
	year: 365 * 24 * 60,
}

export type AnalyticsTimeRange = {
	start: Date
	end: Date
}

export function startOfDay(date: Date): Date {
	const nextDate = new Date(date)
	nextDate.setHours(0, 0, 0, 0)
	return nextDate
}

export function getRoundedNow(timestamp: number): Date {
	const roundedTimestamp = Math.floor(timestamp / TIME_RANGE_ROUNDING_MS) * TIME_RANGE_ROUNDING_MS
	return new Date(roundedTimestamp)
}

export function getDateInputValue(date: Date): string {
	const year = date.getFullYear()
	const month = String(date.getMonth() + 1).padStart(2, '0')
	const day = String(date.getDate()).padStart(2, '0')
	return `${year}-${month}-${day}`
}

export function parseDateInputValue(value: string): Date {
	const parsedDate = new Date(`${value}T00:00:00`)
	return Number.isNaN(parsedDate.getTime()) ? startOfDay(new Date()) : parsedDate
}

export function parseDateTimeInputValue(value: string): Date {
	const parsedDate = new Date(value)
	return Number.isNaN(parsedDate.getTime()) ? getRoundedNow(Date.now()) : parsedDate
}

export function addDays(date: Date, days: number): Date {
	const nextDate = new Date(date)
	nextDate.setDate(nextDate.getDate() + days)
	return nextDate
}

function isStartOfDay(date: Date): boolean {
	return (
		date.getHours() === 0 &&
		date.getMinutes() === 0 &&
		date.getSeconds() === 0 &&
		date.getMilliseconds() === 0
	)
}

export function getInclusiveEndDateInputValue(end: Date): string {
	return getDateInputValue(isStartOfDay(end) ? addDays(end, -1) : end)
}

function subtractCalendarMonths(date: Date, months: number): Date {
	const nextDate = new Date(date)
	const day = nextDate.getDate()
	nextDate.setDate(1)
	nextDate.setMonth(nextDate.getMonth() - months)
	const daysInMonth = new Date(nextDate.getFullYear(), nextDate.getMonth() + 1, 0).getDate()
	nextDate.setDate(Math.min(day, daysInMonth))
	return nextDate
}

export function getTimeRangeForPreset(
	preset: AnalyticsTimeframePreset,
	nowTimestamp: number,
	allTimeStartDate: Date = new Date(Date.UTC(2023, 0, 1, 0, 0, 0, 0)),
): AnalyticsTimeRange {
	const now = getRoundedNow(nowTimestamp)
	const end = new Date(now)

	switch (preset) {
		case 'today':
			return { start: startOfDay(now), end }
		case 'yesterday': {
			const todayStart = startOfDay(now)
			return {
				start: new Date(todayStart.getTime() - 24 * 60 * 60 * 1000),
				end: todayStart,
			}
		}
		case 'last_7_days':
			return {
				start: new Date(end.getTime() - 7 * 24 * 60 * 60 * 1000),
				end,
			}
		case 'last_14_days':
			return {
				start: new Date(end.getTime() - 14 * 24 * 60 * 60 * 1000),
				end,
			}
		case 'last_30_days':
			return {
				start: new Date(end.getTime() - 30 * 24 * 60 * 60 * 1000),
				end,
			}
		case 'last_90_days':
			return {
				start: new Date(end.getTime() - 90 * 24 * 60 * 60 * 1000),
				end,
			}
		case 'last_180_days':
			return {
				start: new Date(end.getTime() - 180 * 24 * 60 * 60 * 1000),
				end,
			}
		case 'year_to_date': {
			const yearStart = new Date(now.getFullYear(), 0, 1)
			yearStart.setHours(0, 0, 0, 0)
			return { start: yearStart, end }
		}
		case 'all_time':
			return {
				start: new Date(allTimeStartDate),
				end,
			}
		default:
			return {
				start: new Date(end.getTime() - 24 * 60 * 60 * 1000),
				end,
			}
	}
}

export function getTimeRangeForLastTimeframe(
	amountValue: number,
	unit: AnalyticsLastTimeframeUnit,
	nowTimestamp: number,
): AnalyticsTimeRange {
	const end = getRoundedNow(nowTimestamp)
	const amount = Math.max(1, Math.floor(amountValue))

	switch (unit) {
		case 'hours':
			return { start: new Date(end.getTime() - amount * 60 * 60 * 1000), end }
		case 'days':
			return { start: new Date(end.getTime() - amount * 24 * 60 * 60 * 1000), end }
		case 'weeks':
			return { start: new Date(end.getTime() - amount * 7 * 24 * 60 * 60 * 1000), end }
		case 'months':
			return { start: subtractCalendarMonths(end, amount), end }
		default:
			return { start: new Date(end.getTime() - 24 * 60 * 60 * 1000), end }
	}
}

export function getTimeRangeForCustomDateRange(
	startDate: string,
	endDate: string,
): AnalyticsTimeRange {
	const start = parseDateInputValue(startDate)
	const inclusiveEnd = parseDateInputValue(endDate)
	return {
		start,
		end: addDays(inclusiveEnd, 1),
	}
}

export function getTimeRangeForCustomDateTimeRange(
	startDateTime: string,
	endDateTime: string,
): AnalyticsTimeRange {
	return {
		start: parseDateTimeInputValue(startDateTime),
		end: parseDateTimeInputValue(endDateTime),
	}
}

export function getAnalyticsTimeRange({
	mode,
	preset,
	lastAmount,
	lastUnit,
	customStartDate,
	customEndDate,
	nowTimestamp,
	allTimeStartDate,
}: {
	mode: AnalyticsTimeframeMode
	preset: AnalyticsTimeframePreset
	lastAmount: number
	lastUnit: AnalyticsLastTimeframeUnit
	customStartDate: string
	customEndDate: string
	nowTimestamp: number
	allTimeStartDate?: Date
}): AnalyticsTimeRange {
	switch (mode) {
		case 'last':
			return getTimeRangeForLastTimeframe(lastAmount, lastUnit, nowTimestamp)
		case 'custom_range':
			return getTimeRangeForCustomDateRange(customStartDate, customEndDate)
		case 'custom_datetime_range':
			return getTimeRangeForCustomDateTimeRange(customStartDate, customEndDate)
		case 'preset':
		default:
			return getTimeRangeForPreset(preset, nowTimestamp, allTimeStartDate)
	}
}

export function getDefaultAnalyticsGroupByForDurationMinutes(
	durationMinutes: number,
): AnalyticsGroupByPreset {
	const days = durationMinutes / (24 * 60)
	if (days <= 2) return '1h'
	if (days <= 7) return '6h'
	if (days <= 90) return 'day'
	if (days <= 365) return 'week'
	if (days <= 365 * 3) return 'month'
	return 'year'
}

export function getAnalyticsGroupByPresetMinutes(preset: AnalyticsGroupByPreset): number {
	return GROUP_BY_PRESET_MINUTES[preset]
}

export function isAnalyticsGroupByAvailableForDurationMinutes(
	preset: AnalyticsGroupByPreset,
	durationMinutes: number,
): boolean {
	const groupByMinutes = getAnalyticsGroupByPresetMinutes(preset)
	const isTooCoarse = groupByMinutes >= durationMinutes
	const isTooFine = durationMinutes / groupByMinutes > MAX_ANALYTICS_TIME_SLICES

	return !isTooCoarse && !isTooFine
}

export function ensureMinimumTimeRange(start: Date, end: Date): AnalyticsTimeRange {
	if (end.getTime() <= start.getTime()) {
		return {
			start: new Date(end.getTime() - MIN_RANGE_MS),
			end,
		}
	}

	if (end.getTime() - start.getTime() < MIN_RANGE_MS) {
		return {
			start: new Date(end.getTime() - MIN_RANGE_MS),
			end,
		}
	}

	return { start, end }
}

export function useSelectedAnalyticsTimeRange() {
	const {
		selectedTimeframeMode,
		selectedTimeframe,
		selectedLastTimeframeAmount,
		selectedLastTimeframeUnit,
		selectedCustomTimeframeStartDate,
		selectedCustomTimeframeEndDate,
		queryRefreshTimestamp,
		analyticsAllTimeStartDate,
	} = injectAnalyticsDashboardContext()

	const selectedTimeRange = computed(() =>
		getAnalyticsTimeRange({
			mode: selectedTimeframeMode.value,
			preset: selectedTimeframe.value,
			lastAmount: selectedLastTimeframeAmount.value,
			lastUnit: selectedLastTimeframeUnit.value,
			customStartDate: selectedCustomTimeframeStartDate.value,
			customEndDate: selectedCustomTimeframeEndDate.value,
			nowTimestamp: queryRefreshTimestamp.value,
			allTimeStartDate: analyticsAllTimeStartDate.value,
		}),
	)

	const selectedTimeframeDurationMinutes = computed(() => {
		const { start, end } = ensureMinimumTimeRange(
			selectedTimeRange.value.start,
			selectedTimeRange.value.end,
		)
		const durationMs = end.getTime() - start.getTime()
		return Math.max(1, Math.floor(durationMs / (60 * 1000)))
	})

	return {
		selectedTimeRange,
		selectedTimeframeDurationMinutes,
	}
}
