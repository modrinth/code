import { defineMessages } from '#ui/composables/i18n'

export const detailsMessages = defineMessages({
	rootArchive: { id: 'archive-viewer.details.root-archive', defaultMessage: 'Archive' },
	nestedArchive: {
		id: 'archive-viewer.details.nested-archive',
		defaultMessage: 'Nested archive · depth {depth}',
	},
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
	onModrinth: { id: 'archive-viewer.details.on-modrinth', defaultMessage: 'On Modrinth' },
	onModrinthMatch: {
		id: 'archive-viewer.details.on-modrinth-match',
		defaultMessage: '{project} {version}',
	},
	onModrinthNone: {
		id: 'archive-viewer.details.on-modrinth-none',
		defaultMessage: 'Not found',
	},
	detectedLanguage: {
		id: 'archive-viewer.details.detected-language',
		defaultMessage: 'Language',
	},
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
	encrypted: { id: 'archive-viewer.details.encrypted', defaultMessage: 'Encrypted' },
	yes: { id: 'archive-viewer.details.yes', defaultMessage: 'Yes' },
	comment: { id: 'archive-viewer.details.comment', defaultMessage: 'Comment' },
	listedHash: { id: 'archive-viewer.details.listed-hash', defaultMessage: 'Listed hash' },
	listedHashMatch: { id: 'archive-viewer.details.listed-hash-match', defaultMessage: 'Matches' },
	listedHashMismatch: {
		id: 'archive-viewer.details.listed-hash-mismatch',
		defaultMessage: "Doesn't match",
	},
	listedHashPending: {
		id: 'archive-viewer.details.listed-hash-pending',
		defaultMessage: 'Checked once fully downloaded',
	},
	listedHashValue: {
		id: 'archive-viewer.details.listed-hash-value',
		defaultMessage: 'Listed SHA-512: {hash}',
	},
	sourceName: {
		id: 'archive-viewer.details.source-name',
		defaultMessage: '{host, select, modrinth {Modrinth} curseforge {CurseForge} other {External}}',
	},
})
