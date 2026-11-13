export function formatDate(value: string, withTime = false) {
	const date = new Date(value)
	if (!withTime) return date.toLocaleDateString('en-US', { dateStyle: 'medium', timeZone: 'UTC' })

	return (
		date.toLocaleString('en-US', {
			dateStyle: 'medium',
			timeStyle: 'medium',
			timeZone: 'UTC',
		}) + ' UTC'
	)
}

export function formatDuration(startedAt: string, completedAt: string, exact = false) {
	const milliseconds = Math.max(0, Date.parse(completedAt) - Date.parse(startedAt))
	const hours = Math.floor(milliseconds / 3_600_000)
	const minutes = Math.floor(milliseconds / 60_000) % 60
	const seconds = (milliseconds % 60_000) / 1_000

	if (!exact) {
		if (hours > 0) return `${hours} hr${minutes ? ` ${minutes} min` : ''}`
		if (minutes > 0) return `${minutes} min`
		return `${Math.floor(seconds)} sec`
	}

	return [hours ? `${hours} hr` : '', minutes ? `${minutes} min` : '', `${seconds} sec`]
		.filter(Boolean)
		.join(' ')
}
