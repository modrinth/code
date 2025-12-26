import { computed, type ComputedRef } from 'vue'
import { useI18n } from 'vue-i18n'

export type Formatter = (value: Date | number, options?: FormatOptions) => string

export interface FormatOptions {
	roundingMode?: 'halfExpand' | 'floor' | 'ceil'
}

const formatters = new Map<string, ComputedRef<Intl.RelativeTimeFormat>>()

export function useRelativeTime(): Formatter {
	const { locale } = useI18n()

	const formatterRef = computed(
		() =>
			new Intl.RelativeTimeFormat(locale.value, {
				numeric: 'auto',
				style: 'long',
			}),
	)

	if (!formatters.has(locale.value)) {
		formatters.set(locale.value, formatterRef)
	}

	return (value: Date | number) => {
		const date = value instanceof Date ? value : new Date(value)
		const now = Date.now()
		const diff = date.getTime() - now

		const seconds = Math.round(diff / 1000)
		const minutes = Math.round(diff / 60000)
		const hours = Math.round(diff / 3600000)
		const days = Math.round(diff / 86400000)
		const weeks = Math.round(diff / 604800000)
		const months = Math.round(diff / 2629746000)
		const years = Math.round(diff / 31556952000)

		const rtf = formatterRef.value

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
