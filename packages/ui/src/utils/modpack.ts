import JSZip from 'jszip'

const EOCD_SIGNATURE = 0x06054b50
const CENTRAL_DIRECTORY_FILE_HEADER_SIGNATURE = 0x02014b50
const LOCAL_FILE_HEADER_SIGNATURE = 0x04034b50
const ZIP64_VALUE_16 = 0xffff
const ZIP64_VALUE_32 = 0xffffffff
const EOCD_MIN_SIZE = 22
const EOCD_MAX_COMMENT_SIZE = 0xffff
const EOCD_SEARCH_SIZE = EOCD_MIN_SIZE + EOCD_MAX_COMMENT_SIZE
const LOCAL_FILE_HEADER_SIZE = 30
const CENTRAL_DIRECTORY_FILE_HEADER_SIZE = 46
const MODRINTH_INDEX_PATH = 'modrinth.index.json'

export type ModrinthPackIndex = Record<string, unknown> & {
	formatVersion?: number
	game?: string
	versionId?: string
	name?: string
	summary?: string
	files?: Array<Record<string, unknown>>
	dependencies?: Record<string, string>
}

export async function getModrinthIndexFromModpackUrl(url: string): Promise<ModrinthPackIndex> {
	const endRange = await fetchRange(url, `bytes=-${EOCD_SEARCH_SIZE}`)
	const eocd = locateEndOfCentralDirectory(endRange.bytes)
	const centralDirectoryRange = await fetchRange(
		url,
		`bytes=${eocd.centralDirectoryOffset}-${
			eocd.centralDirectoryOffset + eocd.centralDirectorySize - 1
		}`,
	)
	const entry = findCentralDirectoryEntry(centralDirectoryRange.bytes)
	const localHeaderRange = await fetchRange(
		url,
		`bytes=${entry.localHeaderOffset}-${entry.localHeaderOffset + LOCAL_FILE_HEADER_SIZE - 1}`,
	)
	const localHeaderLength = getLocalHeaderLength(localHeaderRange.bytes)
	const fileEnd = entry.localHeaderOffset + localHeaderLength + entry.compressedSize - 1
	const localFileRange = await fetchRange(url, `bytes=${entry.localHeaderOffset}-${fileEnd}`)
	const zip = await JSZip.loadAsync(createSingleFileZip(localFileRange.bytes, entry.record))
	const indexFile = zip.file(MODRINTH_INDEX_PATH)

	if (!indexFile) {
		throw new Error(`${MODRINTH_INDEX_PATH} was not found in the reconstructed modpack slice`)
	}

	const parsed = JSON.parse(await indexFile.async('text')) as unknown
	if (!isRecord(parsed)) {
		throw new Error(`${MODRINTH_INDEX_PATH} did not contain a JSON object`)
	}

	return parsed as ModrinthPackIndex
}

async function fetchRange(url: string, range: string) {
	const response = await fetch(url, {
		headers: {
			Range: range,
		},
	})

	if (response.status !== 206) {
		throw new Error(`Expected HTTP 206 for range request, got ${response.status}`)
	}

	const contentRange = parseContentRange(response.headers.get('content-range'))
	const bytes = new Uint8Array(await response.arrayBuffer())
	if (bytes.length !== contentRange.end - contentRange.start + 1) {
		throw new Error('Range response length did not match Content-Range')
	}

	return {
		bytes,
		...contentRange,
	}
}

function parseContentRange(contentRange: string | null) {
	const match = contentRange?.match(/^bytes (\d+)-(\d+)\/(\d+)$/)
	if (!match) {
		throw new Error('Range response did not include a valid Content-Range header')
	}

	const start = Number(match[1])
	const end = Number(match[2])
	const total = Number(match[3])
	if (!Number.isSafeInteger(start) || !Number.isSafeInteger(end) || !Number.isSafeInteger(total)) {
		throw new Error('Content-Range contains an unsafe integer value')
	}

	return { start, end, total }
}

function locateEndOfCentralDirectory(bytes: Uint8Array) {
	const view = viewFor(bytes)

	for (let index = bytes.length - EOCD_MIN_SIZE; index >= 0; index--) {
		if (view.getUint32(index, true) !== EOCD_SIGNATURE) {
			continue
		}

		const commentLength = view.getUint16(index + 20, true)
		if (index + EOCD_MIN_SIZE + commentLength !== bytes.length) {
			continue
		}

		const diskNumber = view.getUint16(index + 4, true)
		const centralDirectoryDisk = view.getUint16(index + 6, true)
		const diskEntryCount = view.getUint16(index + 8, true)
		const totalEntryCount = view.getUint16(index + 10, true)
		const centralDirectorySize = view.getUint32(index + 12, true)
		const centralDirectoryOffset = view.getUint32(index + 16, true)

		if (diskNumber !== 0 || centralDirectoryDisk !== 0) {
			throw new Error('Multi-disk zip archives are not supported')
		}
		if (
			diskEntryCount === ZIP64_VALUE_16 ||
			totalEntryCount === ZIP64_VALUE_16 ||
			centralDirectorySize === ZIP64_VALUE_32 ||
			centralDirectoryOffset === ZIP64_VALUE_32
		) {
			throw new Error('Zip64 modpack indexes are not supported')
		}

		return {
			centralDirectorySize,
			centralDirectoryOffset,
		}
	}

	throw new Error('Could not locate zip central directory')
}

function findCentralDirectoryEntry(centralDirectory: Uint8Array) {
	const decoder = new TextDecoder()
	let offset = 0

	while (offset < centralDirectory.length) {
		if (offset + CENTRAL_DIRECTORY_FILE_HEADER_SIZE > centralDirectory.length) {
			throw new Error('Central directory ended in the middle of a file header')
		}

		const recordStart = offset
		const view = viewFor(centralDirectory, offset)
		if (view.getUint32(0, true) !== CENTRAL_DIRECTORY_FILE_HEADER_SIGNATURE) {
			throw new Error('Central directory contained an invalid file header')
		}

		const compressedSize = view.getUint32(20, true)
		const localHeaderOffset = view.getUint32(42, true)
		const fileNameLength = view.getUint16(28, true)
		const extraFieldLength = view.getUint16(30, true)
		const fileCommentLength = view.getUint16(32, true)
		const recordLength =
			CENTRAL_DIRECTORY_FILE_HEADER_SIZE + fileNameLength + extraFieldLength + fileCommentLength

		if (offset + recordLength > centralDirectory.length) {
			throw new Error('Central directory file header exceeded the directory length')
		}

		const fileName = decoder.decode(
			centralDirectory.subarray(
				offset + CENTRAL_DIRECTORY_FILE_HEADER_SIZE,
				offset + CENTRAL_DIRECTORY_FILE_HEADER_SIZE + fileNameLength,
			),
		)

		if (fileName === MODRINTH_INDEX_PATH) {
			if (compressedSize === ZIP64_VALUE_32 || localHeaderOffset === ZIP64_VALUE_32) {
				throw new Error('Zip64 modpack indexes are not supported')
			}

			return {
				compressedSize,
				localHeaderOffset,
				record: centralDirectory.slice(recordStart, recordStart + recordLength),
			}
		}

		offset += recordLength
	}

	throw new Error(`${MODRINTH_INDEX_PATH} was not found in the modpack root`)
}

function getLocalHeaderLength(localHeader: Uint8Array): number {
	if (localHeader.length !== LOCAL_FILE_HEADER_SIZE) {
		throw new Error('Local file header range did not return the expected number of bytes')
	}

	const view = viewFor(localHeader)
	if (view.getUint32(0, true) !== LOCAL_FILE_HEADER_SIGNATURE) {
		throw new Error('Local file header had an invalid signature')
	}

	return LOCAL_FILE_HEADER_SIZE + view.getUint16(26, true) + view.getUint16(28, true)
}

function createSingleFileZip(
	localFile: Uint8Array,
	centralDirectoryRecord: Uint8Array,
): Uint8Array {
	const centralDirectory = centralDirectoryRecord.slice()
	viewFor(centralDirectory).setUint32(42, 0, true)

	const eocd = new Uint8Array(EOCD_MIN_SIZE)
	const eocdView = viewFor(eocd)
	eocdView.setUint32(0, EOCD_SIGNATURE, true)
	eocdView.setUint16(8, 1, true)
	eocdView.setUint16(10, 1, true)
	eocdView.setUint32(12, centralDirectory.length, true)
	eocdView.setUint32(16, localFile.length, true)

	const zip = new Uint8Array(localFile.length + centralDirectory.length + eocd.length)
	zip.set(localFile, 0)
	zip.set(centralDirectory, localFile.length)
	zip.set(eocd, localFile.length + centralDirectory.length)

	return zip
}

function viewFor(bytes: Uint8Array, byteOffset = 0): DataView {
	return new DataView(bytes.buffer, bytes.byteOffset + byteOffset, bytes.byteLength - byteOffset)
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}
