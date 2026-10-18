import { defineMessages } from '#ui/composables/i18n'
import type { HashType } from '#ui/helpers/toolbox/file'

export const hashTypeMessages = defineMessages({
	sha1: { id: 'archive-viewer.hash-type.sha1', defaultMessage: 'SHA-1' },
	sha256: { id: 'archive-viewer.hash-type.sha256', defaultMessage: 'SHA-256' },
	sha384: { id: 'archive-viewer.hash-type.sha384', defaultMessage: 'SHA-384' },
	sha512: { id: 'archive-viewer.hash-type.sha512', defaultMessage: 'SHA-512' },
	murmur2: { id: 'archive-viewer.hash-type.murmur2', defaultMessage: 'Murmur2' },
	crc32: { id: 'archive-viewer.hash-type.crc32', defaultMessage: 'CRC-32' },
} satisfies Record<HashType, unknown>)
