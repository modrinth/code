import { defineMessages } from '#ui/composables/i18n'
import type { DecompilerTypes } from '#ui/helpers/toolbox/decompile'
import type { HashType } from '#ui/helpers/toolbox/file'

export const hashTypeMessages = defineMessages({
	sha1: { id: 'archive-viewer.hash-type.sha1', defaultMessage: 'SHA-1' },
	sha256: { id: 'archive-viewer.hash-type.sha256', defaultMessage: 'SHA-256' },
	sha384: { id: 'archive-viewer.hash-type.sha384', defaultMessage: 'SHA-384' },
	sha512: { id: 'archive-viewer.hash-type.sha512', defaultMessage: 'SHA-512' },
	murmur2: { id: 'archive-viewer.hash-type.murmur2', defaultMessage: 'Murmur2' },
	crc32: { id: 'archive-viewer.hash-type.crc32', defaultMessage: 'CRC-32' },
} satisfies Record<HashType, unknown>)

export const decompilerMessages = defineMessages({
	cfr: { id: 'archive-viewer.decompiler.cfr', defaultMessage: 'CFR' },
	vineflower: { id: 'archive-viewer.decompiler.vineflower', defaultMessage: 'Vineflower' },
	procyon: { id: 'archive-viewer.decompiler.procyon', defaultMessage: 'Procyon' },
	krakatau: { id: 'archive-viewer.decompiler.krakatau', defaultMessage: 'Krakatau' },
} satisfies Record<DecompilerTypes, unknown>)
