import { computed } from 'vue'

import { useFormatDateTime } from '#ui/composables/format-date-time'
import { useVIntl } from '#ui/composables/i18n'
import type { FileZipTreeNode } from '#ui/helpers/toolbox/zip'

import { useNodeLazyValue } from '../../../composables/node-values'
import type { ArchiveDetailRow } from '../../../types'
import { isPlaceholderDate } from './entry-values'
import { detailsMessages as messages } from './messages'
import type { DetailGroups } from './types'

/**
 * A file as stored in its archive: when it changed and how it's stored, its sizes (stored and
 * full), and its checksums, the hashes being computed (by the worker) as they're shown.
 */
export function useEntryDetails(file: () => FileZipTreeNode | undefined) {
	const { formatMessage } = useVIntl()
	const formatDate = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

	const fileType = useNodeLazyValue(file, 'fileTypeInfo')
	const sha1 = useNodeLazyValue(file, 'sha1')
	const sha256 = useNodeLazyValue(file, 'sha256')
	const sha512 = useNodeLazyValue(file, 'sha512')
	const murmur2 = useNodeLazyValue(file, 'murmur2')

	function compressionName(method: number | undefined) {
		if (method == null) return null
		if (method === 0) return formatMessage(messages.compressionStored)
		if (method === 8) return formatMessage(messages.compressionDeflate)
		return formatMessage(messages.compressionOther, { method })
	}

	return computed<DetailGroups>(() => {
		const target = file()
		if (!target) return { rows: [], stored: null, hashes: [] }
		const { metadata } = target

		const rows: ArchiveDetailRow[] = [
			{
				id: 'detected-type',
				label: formatMessage(messages.detectedType),
				value: fileType.value.value?.mime ?? null,
				loading: fileType.loading.value,
			},
			{
				id: 'compression',
				label: formatMessage(messages.compression),
				value: compressionName(metadata.compressionMethod),
			},
		]
		if (!isPlaceholderDate(metadata.lastModDate)) {
			rows.push({
				id: 'modified',
				label: formatMessage(messages.modified),
				value: formatDate(metadata.lastModDate!),
			})
		}
		if (metadata.encrypted) {
			rows.push({
				id: 'encrypted',
				label: formatMessage(messages.encrypted),
				value: formatMessage(messages.yes),
			})
		}
		if (metadata.comment) {
			rows.push({ id: 'comment', label: formatMessage(messages.comment), value: metadata.comment })
		}

		return {
			rows,
			stored: { compressed: metadata.compressedSize, size: metadata.uncompressedSize },
			hashes: [
				{
					id: 'sha1',
					label: formatMessage(messages.sha1),
					value: sha1.value.value,
					loading: sha1.loading.value,
					hash: true,
				},
				{
					id: 'sha256',
					label: formatMessage(messages.sha256),
					value: sha256.value.value,
					loading: sha256.loading.value,
					hash: true,
				},
				{
					id: 'sha512',
					label: formatMessage(messages.sha512),
					value: sha512.value.value,
					loading: sha512.loading.value,
					hash: true,
				},
				{
					id: 'murmur2',
					label: formatMessage(messages.murmur2),
					value: murmur2.value.value,
					loading: murmur2.loading.value,
					hash: true,
				},
				...(metadata.signature != null
					? [
							{
								id: 'crc32',
								label: formatMessage(messages.crc32),
								value: metadata.signature.toString(16).padStart(8, '0'),
								hash: true,
							},
						]
					: []),
			],
		}
	})
}
