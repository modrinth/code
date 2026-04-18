import { useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

const TABLE_UNITS = ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB'] as const

/** Localized file sizes for the shared files tab (table, upload queue, extraction progress). */
export function useFormatFileSizeI18n() {
	const { formatMessage } = useVIntl()

	/** Match FileTableRow byte formatting. */
	function formatTableRowSize(bytes: number): string {
		if (bytes === 0) {
			return formatMessage(commonMessages.fileSizeFormatted, { value: '0', unit: 'B' })
		}
		const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), TABLE_UNITS.length - 1)
		const size = (bytes / Math.pow(1024, exponent)).toFixed(2)
		return formatMessage(commonMessages.fileSizeFormatted, {
			value: size,
			unit: TABLE_UNITS[exponent],
		})
	}

	/** Match FileUploadDropdown queue item sizes. */
	function formatUploadQueueSize(bytes: number): string {
		if (bytes < 1024) {
			return formatMessage(commonMessages.fileSizeFormatted, { value: String(bytes), unit: 'B' })
		}
		if (bytes < 1024 ** 2) {
			return formatMessage(commonMessages.fileSizeFormatted, {
				value: (bytes / 1024).toFixed(1),
				unit: 'KB',
			})
		}
		if (bytes < 1024 ** 3) {
			return formatMessage(commonMessages.fileSizeFormatted, {
				value: (bytes / 1024 ** 2).toFixed(1),
				unit: 'MB',
			})
		}
		return formatMessage(commonMessages.fileSizeFormatted, {
			value: (bytes / 1024 ** 3).toFixed(1),
			unit: 'GB',
		})
	}

	/** Match @modrinth/utils formatBytes (KiB / MiB / GiB). */
	function formatBinaryIecSize(bytes: number, decimals = 2): string {
		if (bytes === 0) {
			return formatMessage(commonMessages.fileSizeFormatted, { value: '0', unit: 'Bytes' })
		}
		const k = 1024
		const dm = decimals < 0 ? 0 : decimals
		const units = ['Bytes', 'KiB', 'MiB', 'GiB'] as const
		const i = Math.min(Math.floor(Math.log(bytes) / Math.log(k)), units.length - 1)
		const value = parseFloat((bytes / Math.pow(k, i)).toFixed(dm))
		return formatMessage(commonMessages.fileSizeFormatted, {
			value: String(value),
			unit: units[i],
		})
	}

	return { formatTableRowSize, formatUploadQueueSize, formatBinaryIecSize }
}
