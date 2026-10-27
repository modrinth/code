import { defineMessages, useVIntl } from '#ui/composables/i18n'

const HOURS_PER_DAY = 24
const HOURS_PER_WEEK = HOURS_PER_DAY * 7

export const BACKUP_INTERVAL_PRESETS = [6, 12, 24, 48, HOURS_PER_WEEK]

const messages = defineMessages({
	hours: {
		id: 'servers.backups.interval.hours',
		defaultMessage: '{hours}h',
	},
	days: {
		id: 'servers.backups.interval.days',
		defaultMessage: '{days, plural, one {# day} other {# days}}',
	},
	weekly: {
		id: 'servers.backups.interval.weekly',
		defaultMessage: 'Weekly',
	},
	everyHours: {
		id: 'servers.backups.interval.every-hours',
		defaultMessage: 'Every {hours}h',
	},
	everyDays: {
		id: 'servers.backups.interval.every-days',
		defaultMessage: 'Every {days, plural, one {day} other {# days}}',
	},
})

export function useBackupIntervalFormat() {
	const { formatMessage } = useVIntl()

	function isWholeDays(hours: number) {
		return hours > HOURS_PER_DAY && hours % HOURS_PER_DAY === 0
	}

	function formatInterval(hours: number) {
		if (hours === HOURS_PER_WEEK) return formatMessage(messages.weekly)
		if (isWholeDays(hours)) return formatMessage(messages.days, { days: hours / HOURS_PER_DAY })
		return formatMessage(messages.hours, { hours })
	}

	function formatFrequency(hours: number) {
		if (hours === HOURS_PER_WEEK) return formatMessage(messages.weekly)
		if (isWholeDays(hours)) {
			return formatMessage(messages.everyDays, { days: hours / HOURS_PER_DAY })
		}
		return formatMessage(messages.everyHours, { hours })
	}

	return { formatInterval, formatFrequency }
}
