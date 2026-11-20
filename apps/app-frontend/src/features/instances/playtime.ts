import { defineMessages, type useVIntl } from '@modrinth/ui'

export const playtimeMessages = defineMessages({
	neverPlayed: {
		id: 'instance.playtime.never-played',
		defaultMessage: 'Never played',
	},
	lastPlayed: {
		id: 'instance.last-played',
		defaultMessage: 'Last played',
	},
	hours: {
		id: 'instance.playtime.hours',
		defaultMessage: '{count, plural, one {# hour} other {# hours}}',
	},
	minutes: {
		id: 'instance.playtime.minutes',
		defaultMessage: '{count, plural, one {# minute} other {# minutes}}',
	},
	seconds: {
		id: 'instance.playtime.seconds',
		defaultMessage: '{count, plural, one {# second} other {# seconds}}',
	},
})

export function formatPlaytime(
	totalSeconds: number,
	formatMessage: ReturnType<typeof useVIntl>['formatMessage'],
) {
	const seconds = Math.floor(totalSeconds)
	if (seconds <= 0) return undefined

	const hours = Math.floor(seconds / 3600)
	if (hours >= 1) return formatMessage(playtimeMessages.hours, { count: hours })

	const minutes = Math.floor(seconds / 60)
	if (minutes >= 1) return formatMessage(playtimeMessages.minutes, { count: minutes })

	return formatMessage(playtimeMessages.seconds, { count: seconds })
}
