import { LRUCache } from 'lru-cache'

import { injectI18n } from '../providers/i18n'

const formatterCache = new LRUCache<string, Intl.DateTimeFormat>({ max: 40 })

export function useFormatDateTime(options?: Intl.DateTimeFormatOptions) {
	const { locale } = injectI18n()

	const formatter = getFormatter(locale.value, options)

	function format(date?: Date | number | string): string {
		if (typeof date === 'number' || typeof date === 'string') {
			date = new Date(date)
		}
		return formatter!.format(date)
	}

	return format
}

function getFormatter(locale: string, options?: Intl.DateTimeFormatOptions): Intl.DateTimeFormat {
	let cacheKey = locale
	if (options) {
		const entries = Object.entries(options)
			.filter(([, value]) => value !== undefined)
			.sort()
			.map(([key, value]) => `${key}=${value}`)
		cacheKey = [locale, ...entries].join(':')
	}

	let formatter = formatterCache.get(cacheKey)
	if (!formatter) {
		formatter = new Intl.DateTimeFormat(locale, options)
		formatterCache.set(cacheKey, formatter)
	}
	return formatter
}
