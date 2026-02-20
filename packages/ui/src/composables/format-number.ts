import { LRUCache } from 'lru-cache'

import { injectI18n } from '../providers/i18n'

const formatterCache = new LRUCache<string, Intl.NumberFormat>({ max: 15 })

// `formatNumber(1234567)` → `1,234,567`
export function useFormatNumber() {
	const { locale } = injectI18n()

	function format(value: number | bigint): string {
		const formatter = getStandardFormatter(locale.value)
		return formatter!.format(value)
	}

	return format
}

// `formatCompactNumber(1234567)` → `1.23M`
//
// Use `formatCompactNumberPlural` over `{(here!), plural, one {...} other {...}}`
export function useCompactNumber() {
	const { locale } = injectI18n()

	function formatCompactNumber(value: number | bigint): string {
		if (value < 10_000) {
			const standardFormatter = getStandardFormatter(locale.value)
			return standardFormatter.format(value)
		}
		if (value < 1_000_000) {
			const oneDigitCompactFormatter = getCompactFormatter(locale.value, 1)
			return oneDigitCompactFormatter.format(value)
		}
		const twoDigitsCompactFormatter = getCompactFormatter(locale.value, 2)
		return twoDigitsCompactFormatter.format(value)
	}

	function formatCompactNumberPlural(value: number | bigint): string {
		if (value < 10_000) {
			return value.toString()
		}
		if (value < 1_000_000) {
			const oneDigitCompactFormatter = getCompactFormatter(locale.value, 1)
			return oneDigitCompactFormatter.format(value)
		}
		const twoDigitsCompactFormatter = getCompactFormatter(locale.value, 2)
		return twoDigitsCompactFormatter.format(value)
	}

	return { formatCompactNumber, formatCompactNumberPlural }
}

function getStandardFormatter(locale: string): Intl.NumberFormat {
	const cacheKey = `${locale}:standard`
	let formatter = formatterCache.get(cacheKey)
	if (!formatter) {
		formatter = new Intl.NumberFormat(locale)
		formatterCache.set(cacheKey, formatter)
	}
	return formatter
}

function getCompactFormatter(locale: string, maximumFractionDigits: number): Intl.NumberFormat {
	const cacheKey = `${locale}:compact:${maximumFractionDigits}`
	let formatter = formatterCache.get(cacheKey)
	if (!formatter) {
		formatter = new Intl.NumberFormat(locale, {
			notation: 'compact',
			maximumFractionDigits,
		})
		formatterCache.set(cacheKey, formatter)
	}
	return formatter
}
