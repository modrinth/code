import { LRUCache } from 'lru-cache'

import { injectI18n } from '../providers/i18n'
import { LOCALES } from './i18n.ts'

const formatterCache = new LRUCache<string, Intl.RelativeTimeFormat>({ max: 5 })

export function useRelativeTime() {
	const { locale } = injectI18n()

	return (value: Date | number | string | null | undefined) => {
		if (value == null) {
			return ''
		}

		const date = value instanceof Date ? value : new Date(value)

		if (Number.isNaN(date.getTime())) {
			return ''
		}
		const now = Date.now()
		const diff = date.getTime() - now

		const seconds = Math.round(diff / 1000)
		const minutes = Math.round(diff / 60000)
		const hours = Math.round(diff / 3600000)
		const days = Math.round(diff / 86400000)
		const weeks = Math.round(diff / 604800000)
		const months = Math.round(diff / 2629746000)
		const years = Math.round(diff / 31556952000)

		const rtf = getFormatter(locale.value)

		if (Math.abs(seconds) < 60) {
			return rtf.format(seconds, 'second')
		} else if (Math.abs(minutes) < 60) {
			return rtf.format(minutes, 'minute')
		} else if (Math.abs(hours) < 24) {
			return rtf.format(hours, 'hour')
		} else if (Math.abs(days) < 7) {
			return rtf.format(days, 'day')
		} else if (Math.abs(weeks) < 4) {
			return rtf.format(weeks, 'week')
		} else if (Math.abs(months) < 12) {
			return rtf.format(months, 'month')
		} else {
			return rtf.format(years, 'year')
		}
	}
}

function getFormatter(locale: string): Intl.RelativeTimeFormat {
	let formatter = formatterCache.get(locale)
	if (!formatter) {
		const localeDefinition = LOCALES.find((loc) => loc.code === locale)
		formatter = new Intl.RelativeTimeFormat(locale, {
			numeric: localeDefinition?.numeric || 'auto',
			style: 'long',
		})
		formatterCache.set(locale, formatter)
	}
	return formatter
}
