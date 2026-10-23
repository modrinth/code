import { computed } from 'vue'

import { useFormatDateTime } from '#ui/composables/format-date-time'
import { useVIntl } from '#ui/composables/i18n'
import type { FileTypeInfo, HashType } from '#ui/helpers/toolbox/file'
import type { FileZipTreeNode } from '#ui/helpers/toolbox/zip'
import { getEditorLanguage } from '#ui/utils/file-extensions'

import { useNodeLazyValue } from '../../../composables/node-values'
import { useArchiveDetailSettings } from '../../../settings'
import type { ArchiveDetailRow } from '../../../types'
import { isPlaceholderDate } from './entry-values'
import { detailsMessages as messages } from './messages'
import type { DetailGroups } from './types'

/** Languages that only mean the file is plain text, which isn't worth listing. */
const PLAIN_LANGUAGES = new Set(['text', 'plaintext'])

/** The language a file is highlighted as, from its detected type or else its name. */
function languageOf(name: string, type: FileTypeInfo | null | undefined) {
	const language = name.endsWith('.class') ? 'java' : getEditorLanguage(name, type ?? undefined)
	return PLAIN_LANGUAGES.has(language) ? null : language
}

/**
 * A file as stored in its archive: its detected type and language, when it changed and how it's
 * stored, its sizes (stored and full), and how to compute its hashes (by the worker), only done
 * for those shown. Its CRC-32 is the one its archive stores for it. Details show as the viewer's
 * settings say.
 */
export function useEntryDetails(file: () => FileZipTreeNode | undefined) {
	const { formatMessage } = useVIntl()
	const formatDate = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })
	const settings = useArchiveDetailSettings()

	const fileType = useNodeLazyValue(file, 'fileTypeInfo')

	const hash = computed(() => {
		const target = file()
		if (!target) return undefined
		const signature = target.metadata.signature
		return (type: HashType) =>
			type === 'crc32' && signature != null
				? Promise.resolve((signature >>> 0).toString(16).padStart(8, '0'))
				: target.hash(type)
	})

	function compressionName(method: number | undefined) {
		if (method == null) return null
		if (method === 0) return formatMessage(messages.compressionStored)
		if (method === 8) return formatMessage(messages.compressionDeflate)
		return formatMessage(messages.compressionOther, { method })
	}

	return computed<DetailGroups>(() => {
		const target = file()
		if (!target) return { rows: [], more: [], stored: null, hash: undefined }
		const { metadata } = target

		const rows: ArchiveDetailRow[] = [
			{
				id: 'detected-type',
				label: formatMessage(messages.detectedType),
				value: fileType.value.value?.mime ?? null,
				loading: fileType.loading.value,
				visibility: settings.visibility('detectedType'),
			},
		]
		const language = fileType.loading.value ? null : languageOf(target.name, fileType.value.value)
		if (language) {
			rows.push({
				id: 'detected-language',
				label: formatMessage(messages.detectedLanguage),
				value: language,
				visibility: settings.visibility('detectedLanguage'),
			})
		}
		const more: ArchiveDetailRow[] = [
			{
				id: 'compression',
				label: formatMessage(messages.compression),
				value: compressionName(metadata.compressionMethod),
			},
		]
		if (!isPlaceholderDate(metadata.lastModDate)) {
			more.push({
				id: 'modified',
				label: formatMessage(messages.modified),
				value: formatDate(metadata.lastModDate!),
			})
		}
		if (metadata.encrypted) {
			more.push({
				id: 'encrypted',
				label: formatMessage(messages.encrypted),
				value: formatMessage(messages.yes),
			})
		}
		if (metadata.comment) {
			more.push({ id: 'comment', label: formatMessage(messages.comment), value: metadata.comment })
		}

		return {
			rows,
			more,
			stored: { compressed: metadata.compressedSize, size: metadata.uncompressedSize },
			hash: hash.value,
		}
	})
}
