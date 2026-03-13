import { LRUCache } from 'lru-cache'

import { injectI18n } from '../providers/i18n'

const formatterCache = new LRUCache<string, Intl.NumberFormat>({ max: 10 })
const maxDigitsCache = new LRUCache<string, number>({ max: 10 })

// `formatMoney(1234.56, 'USD')` → `$1,234.56`
export function useFormatMoney() {
	const { locale } = injectI18n()

	function format(number: number, currency = 'USD'): string {
		try {
			const formatter = getFormatter(locale.value, currency)
			return formatter!.format(number)
		} catch {
			return `${currency} ${number.toFixed(2)}`
		}
	}

	return format
}

// `formatPrice(123456, 'USD')` → `$1,234.56`
export function useFormatPrice() {
	const { locale } = injectI18n()

	function format(price: number, currency: string, trimZeros = false): string {
		const maxDigits = getMaxDigits(currency)
		const convertedPrice = price / Math.pow(10, maxDigits)

		const minimumFractionDigits = trimZeros && Number.isInteger(convertedPrice) ? 0 : undefined

		try {
			const formatter = getFormatter(locale.value, currency, minimumFractionDigits)
			return formatter.format(convertedPrice)
		} catch {
			return `${currency} ${convertedPrice}`
		}
	}

	return format
}

function getFormatter(
	locale: string,
	currency: string,
	minimumFractionDigits?: number,
): Intl.NumberFormat {
	const cacheKey = `${locale}:${currency}:${minimumFractionDigits}`
	let formatter = formatterCache.get(cacheKey)
	if (!formatter) {
		formatter = new Intl.NumberFormat(locale, {
			style: 'currency',
			currency,
			minimumFractionDigits,
		})
		formatterCache.set(cacheKey, formatter)
	}
	return formatter
}

function getMaxDigits(currency: string): number {
	let maxDigits = maxDigitsCache.get(currency)
	if (!maxDigits) {
		try {
			const formatter = new Intl.NumberFormat(undefined, {
				style: 'currency',
				currency,
			})
			maxDigits = formatter.resolvedOptions().maximumFractionDigits || 2
		} catch {
			maxDigits = 2
		}
		maxDigitsCache.set(currency, maxDigits)
	}
	return maxDigits
}
