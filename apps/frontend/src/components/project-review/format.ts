const numberFormatter = new Intl.NumberFormat('en-US')
const compactFormatter = new Intl.NumberFormat('en-US', {
	notation: 'compact',
	maximumFractionDigits: 2,
})
const shortCompactFormatter = new Intl.NumberFormat('en-US', {
	notation: 'compact',
	maximumFractionDigits: 1,
})

export const formatNumber = (value: number | bigint) => numberFormatter.format(value)
export const formatCompactNumber = (value: number | bigint) => {
	if (value < 10_000) return formatNumber(value)
	return (value < 1_000_000 ? shortCompactFormatter : compactFormatter).format(value)
}

export function formatBytes(bytes: number, decimals = 2) {
	if (bytes === 0) return '0 bytes'
	const units = ['bytes', 'KiB', 'MiB', 'GiB', 'TiB']
	const exponent = Math.max(
		0,
		Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1),
	)
	const value = Number((bytes / 1024 ** exponent).toFixed(decimals))
	return `${formatNumber(value)} ${exponent === 0 && value === 1 ? 'byte' : units[exponent]}`
}

export function createDateFormatter(options?: Intl.DateTimeFormatOptions) {
	const formatter = new Intl.DateTimeFormat('en-US', options)
	return (value: Date | number | string) => formatter.format(new Date(value))
}

export function createRelativeTimeFormatter(options?: Intl.RelativeTimeFormatOptions) {
	const formatter = new Intl.RelativeTimeFormat('en-US', {
		numeric: 'auto',
		...options,
	})
	return (value: Date | number | string | null | undefined) => {
		if (value == null) return ''
		const timestamp = new Date(value).getTime()
		if (Number.isNaN(timestamp)) return ''
		const difference = timestamp - Date.now()
		const units: [Intl.RelativeTimeFormatUnit, number, number][] = [
			['second', 1000, 60],
			['minute', 60_000, 60],
			['hour', 3_600_000, 24],
			['day', 86_400_000, 7],
			['week', 604_800_000, 4],
			['month', 2_629_746_000, 12],
			['year', 31_556_952_000, Infinity],
		]
		for (const [unit, duration, limit] of units) {
			const amount = Math.round(difference / duration)
			if (Math.abs(amount) < limit) return formatter.format(amount, unit)
		}
		return ''
	}
}
