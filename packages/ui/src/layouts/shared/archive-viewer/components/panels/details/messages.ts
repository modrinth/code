import { defineMessages } from '#ui/composables/i18n'

export const detailsMessages = defineMessages({
	rootArchive: { id: 'archive-viewer.details.root-archive', defaultMessage: 'Archive' },
	nestedArchive: {
		id: 'archive-viewer.details.nested-archive',
		defaultMessage: 'Nested archive · depth {depth}',
	},
	copyPath: { id: 'archive-viewer.details.copy-path', defaultMessage: 'Copy path' },
	pathCopied: { id: 'archive-viewer.details.path-copied', defaultMessage: 'Path copied' },
	showParents: {
		id: 'archive-viewer.details.show-parents',
		defaultMessage:
			'{count, plural, one {Show # more parent archive} other {Show # more parent archives}}',
	},
	hideParents: {
		id: 'archive-viewer.details.hide-parents',
		defaultMessage: 'Only show the closest parent archive',
	},
	file: { id: 'archive-viewer.details.file', defaultMessage: 'File' },
	folder: { id: 'archive-viewer.details.folder', defaultMessage: 'Folder' },
	source: { id: 'archive-viewer.details.source', defaultMessage: 'Source' },
	entries: { id: 'archive-viewer.details.entries', defaultMessage: 'Entries' },
	size: { id: 'archive-viewer.details.size', defaultMessage: 'Size' },
	sizeRatio: {
		id: 'archive-viewer.details.size-ratio',
		defaultMessage: '{compressed} / {size}',
	},
	compressedRatio: {
		id: 'archive-viewer.details.compressed-ratio',
		defaultMessage: 'Compressed to {ratio, number, percent} of its size',
	},
	compressedRatioWithContents: {
		id: 'archive-viewer.details.compressed-ratio-with-contents',
		defaultMessage:
			'Compressed to {ratio, number, percent} of its size, holding {contents} of uncompressed contents',
	},
	modified: { id: 'archive-viewer.details.modified', defaultMessage: 'Modified' },
	detectedType: { id: 'archive-viewer.details.detected-type', defaultMessage: 'Detected type' },
	compression: { id: 'archive-viewer.details.compression', defaultMessage: 'Compression' },
	compressionStored: { id: 'archive-viewer.details.compression-stored', defaultMessage: 'Stored' },
	compressionDeflate: {
		id: 'archive-viewer.details.compression-deflate',
		defaultMessage: 'Deflate',
	},
	compressionOther: {
		id: 'archive-viewer.details.compression-other',
		defaultMessage: 'Method {method}',
	},
	crc32: { id: 'archive-viewer.details.crc32', defaultMessage: 'CRC-32' },
	encrypted: { id: 'archive-viewer.details.encrypted', defaultMessage: 'Encrypted' },
	yes: { id: 'archive-viewer.details.yes', defaultMessage: 'Yes' },
	comment: { id: 'archive-viewer.details.comment', defaultMessage: 'Comment' },
	sha1: { id: 'archive-viewer.details.sha1', defaultMessage: 'SHA-1' },
	sha256: { id: 'archive-viewer.details.sha256', defaultMessage: 'SHA-256' },
	sha512: { id: 'archive-viewer.details.sha512', defaultMessage: 'SHA-512' },
	murmur2: { id: 'archive-viewer.details.murmur2', defaultMessage: 'Murmur2' },
	hashes: { id: 'archive-viewer.details.hashes', defaultMessage: 'Hashes' },
	hashPending: {
		id: 'archive-viewer.details.hash-pending',
		defaultMessage: 'Available once fully downloaded',
	},
	sourceName: {
		id: 'archive-viewer.details.source-name',
		defaultMessage: '{host, select, modrinth {Modrinth} curseforge {CurseForge} other {External}}',
	},
	contentsCounts: {
		id: 'archive-viewer.details.contents-counts',
		defaultMessage: 'Contents',
	},
	filesCount: {
		id: 'archive-viewer.details.files-count',
		defaultMessage: '{count, plural, one {# file} other {# files}}',
	},
	foldersCount: {
		id: 'archive-viewer.details.folders-count',
		defaultMessage: '{count, plural, one {# folder} other {# folders}}',
	},
	classFilesCount: {
		id: 'archive-viewer.details.class-files-count',
		defaultMessage: '{count, plural, one {# class file} other {# class files}}',
	},
	nestedArchivesCount: {
		id: 'archive-viewer.details.nested-archives-count',
		defaultMessage: '{count, plural, one {# nested archive} other {# nested archives}}',
	},
	statusReading: {
		id: 'archive-viewer.details.status-reading',
		defaultMessage: 'Reading nested archives...',
	},
	statusPending: { id: 'archive-viewer.details.status-pending', defaultMessage: 'Not read yet' },
	statusRead: { id: 'archive-viewer.details.status-read', defaultMessage: 'Fully read' },
})
