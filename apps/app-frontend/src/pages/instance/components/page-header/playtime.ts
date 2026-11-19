import { defineMessages } from '@modrinth/ui'

export const playtimeMessages = defineMessages({
	neverPlayed: {
		id: 'instance.playtime.never-played',
		defaultMessage: 'Never played',
	},
	lastPlayed: {
		id: 'instance.last-played',
		defaultMessage: 'Last played',
	},
})

export function formatPlaytime(totalSeconds: number) {
	const seconds = Math.floor(totalSeconds)
	if (seconds <= 0) return undefined

	const hours = Math.floor(seconds / 3600)
	if (hours >= 1) return `${hours} hour${hours > 1 ? 's' : ''}`

	const minutes = Math.floor(seconds / 60)
	if (minutes >= 1) return `${minutes} minute${minutes > 1 ? 's' : ''}`

	return `${seconds} second${seconds === 1 ? '' : 's'}`
}
