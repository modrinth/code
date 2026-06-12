import type { Labrinth } from '@modrinth/api-client'

export const CREATOR_PAYOUT_SHARE = 0.75
export const MODRINTH_PAYOUT_SHARE = 0.25

export type PayoutHistoryItem = Labrinth.Payouts.Internal.HistoryItem
export type DistributionAdjustment = Labrinth.Payouts.Internal.DistributionAdjustment
export type DistributionRun = Labrinth.Payouts.Internal.DistributionRun

export function isYearMonth(value: unknown): value is Labrinth.Payouts.Internal.YearMonth {
	return typeof value === 'string' && /^\d{4}-\d{2}$/.test(value)
}

export function formatMonthYear(yearMonth: string): string {
	const date = getYearMonthDate(yearMonth)
	return new Intl.DateTimeFormat(undefined, {
		month: 'long',
		year: 'numeric',
	}).format(date)
}

export function formatShortDate(date: Date): string {
	return new Intl.DateTimeFormat(undefined, {
		weekday: 'short',
		month: 'short',
		day: 'numeric',
	}).format(date)
}

export function formatCurrency(amount: number | null | undefined, options?: { cents?: boolean }) {
	if (amount === null || amount === undefined || Number.isNaN(amount)) {
		return '—'
	}

	return new Intl.NumberFormat(undefined, {
		style: 'currency',
		currency: 'USD',
		minimumFractionDigits: options?.cents ? 2 : 0,
		maximumFractionDigits: options?.cents ? 2 : 0,
	}).format(amount)
}

export function formatSignedCurrency(amount: number | null | undefined): string {
	if (amount === null || amount === undefined || Number.isNaN(amount)) {
		return '—'
	}

	const formatted = formatCurrency(Math.abs(amount))
	return amount < 0 ? `-${formatted}` : formatted
}

export function getReviewDueDate(yearMonth: string): Date {
	return addDays(getLastDayOfMonth(yearMonth), 75)
}

export function getPendingAvailableDate(yearMonth: string): Date {
	return addDays(getLastDayOfMonth(yearMonth), 60)
}

export function getDaysRemaining(date: Date): number {
	const today = new Date()
	today.setHours(0, 0, 0, 0)
	const target = new Date(date)
	target.setHours(0, 0, 0, 0)
	return Math.ceil((target.getTime() - today.getTime()) / 86_400_000)
}

export function getNetActualRevenue(
	amountReceived: number,
	adjustments: DistributionAdjustment[],
): number {
	return roundCurrency(amountReceived + getTotalAdjustments(adjustments))
}

export function getTotalAdjustments(adjustments: DistributionAdjustment[]): number {
	return roundCurrency(adjustments.reduce((total, adjustment) => total + adjustment.amount, 0))
}

export function getCreatorShare(amount: number): number {
	return roundCurrency(amount * CREATOR_PAYOUT_SHARE)
}

export function getModrinthShare(amount: number): number {
	return roundCurrency(amount * MODRINTH_PAYOUT_SHARE)
}

export function getDistributionCreatorAmount(distribution: DistributionRun): number {
	return getCreatorShare(getNetActualRevenue(distribution.amount_received, distribution.adjustments))
}

export function roundCurrency(amount: number): number {
	return Math.round(amount * 100) / 100
}

export function getYearMonthDate(yearMonth: string): Date {
	const [year, month] = yearMonth.split('-').map(Number)
	return new Date(year, month - 1, 1, 12)
}

function getLastDayOfMonth(yearMonth: string): Date {
	const [year, month] = yearMonth.split('-').map(Number)
	return new Date(year, month, 0, 12)
}

function addDays(date: Date, days: number): Date {
	const nextDate = new Date(date)
	nextDate.setDate(nextDate.getDate() + days)
	return nextDate
}
