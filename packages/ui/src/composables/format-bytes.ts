import { defineMessage, useVIntl } from '#ui/composables/i18n.ts'

const messages = [
	defineMessage({
		id: 'format.bytes.0',
		defaultMessage: '{count, plural, one {# byte} other {# bytes}}',
	}),
	defineMessage({
		id: 'format.bytes.1',
		defaultMessage: '{count, number} KiB',
	}),
	defineMessage({
		id: 'format.bytes.2',
		defaultMessage: '{count, number} MiB',
	}),
	defineMessage({
		id: 'format.bytes.3',
		defaultMessage: '{count, number} GiB',
	}),
	defineMessage({
		id: 'format.bytes.4',
		defaultMessage: '{count, number} TiB',
	}),
]

export function useFormatBytes() {
	const { formatMessage } = useVIntl()

	function format(bytes: number, decimals = 2): string {
		if (bytes === 0) return formatMessage(messages[0], { count: 0 })

		const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), messages.length - 1)
		return formatMessage(messages[exponent], {
			count: (bytes / Math.pow(1024, exponent)).toFixed(decimals),
		})
	}

	return format
}
