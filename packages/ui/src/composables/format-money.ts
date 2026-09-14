import { LRUCache } from 'lru-cache'

import { injectI18n } from '../providers/i18n'

const formatterCache = new LRUCache<string, Intl.NumberFormat>({ max: 10 })
// Billing amounts use Stripe's minor units, which can differ from Intl's display defaults
// (for example, COP displays without decimals in some runtimes but stores centavos).
// ISK and UGX also use two-decimal amounts for Stripe compatibility.
// https://docs.stripe.com/currencies#zero-decimal
const zeroDecimalCurrencies = new Set([
	'BIF',
	'CLP',
	'DJF',
	'GNF',
	'JPY',
	'KMF',
	'KRW',
	'MGA',
	'PYG',
	'RWF',
	'VND',
	'VUV',
	'XAF',
	'XOF',
	'XPF',
])

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
		const maxDigits = zeroDecimalCurrencies.has(currency.toUpperCase()) ? 0 : 2
		const convertedPrice = price / Math.pow(10, maxDigits)

		const minimumFractionDigits = trimZeros && Number.isInteger(convertedPrice) ? 0 : maxDigits

		try {
			const formatter = getFormatter(locale.value, currency, minimumFractionDigits, maxDigits)
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
	maximumFractionDigits?: number,
): Intl.NumberFormat {
	const cacheKey = `${locale}:${currency}:${minimumFractionDigits}:${maximumFractionDigits}`
	let formatter = formatterCache.get(cacheKey)
	if (!formatter) {
		formatter = new Intl.NumberFormat(locale, {
			style: 'currency',
			currency,
			minimumFractionDigits,
			maximumFractionDigits,
		})
		formatterCache.set(cacheKey, formatter)
	}
	return formatter
}
