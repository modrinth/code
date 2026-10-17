import { useFormatBytes } from '#ui/composables/format-bytes'
import { useVIntl } from '#ui/composables/i18n'

import type { ArchiveDetailRow } from '../../../types'
import { detailsMessages as messages } from './messages'

/**
 * Builds the size of an item as "compressed / full size", with how far it was compressed (and,
 * for an archive, how much it holds once uncompressed) on hover.
 */
export function useSizeRow() {
	const { formatMessage } = useVIntl()
	const formatBytes = useFormatBytes()

	return (
		compressed: number | null | undefined,
		size: number | null | undefined,
		contents?: number,
	): ArchiveDetailRow => {
		const label = formatMessage(messages.size)
		if (compressed == null || size == null) {
			const known = size ?? compressed
			return { id: 'size', label, value: known != null ? formatBytes(known) : null }
		}

		const ratio = size > 0 ? compressed / size : null
		const tooltip =
			ratio == null
				? undefined
				: contents != null
					? formatMessage(messages.compressedRatioWithContents, {
							ratio,
							contents: formatBytes(contents),
						})
					: formatMessage(messages.compressedRatio, { ratio })

		return {
			id: 'size',
			label,
			value: formatMessage(messages.sizeRatio, {
				compressed: formatBytes(compressed),
				size: formatBytes(size),
			}),
			tooltip,
		}
	}
}
