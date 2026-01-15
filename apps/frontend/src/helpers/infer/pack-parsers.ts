import type JSZip from 'jszip'

import { DATA_PACK_FORMATS, RESOURCE_PACK_FORMATS } from './constants'
import type { GameVersion, InferredVersionInfo, Project, RawFile } from './infer'
import { extractVersionFromFilename, versionType } from './version-utils'

type PackFormat = number | [number] | [number, number]

/**
 * Normalizes a pack format to [major, minor] tuple. See https://minecraft.wiki/w/Pack.mcmeta
 * - Single integer: [major, 0] for min, [major, Infinity] for max
 * - Array [major]: [major, 0] for min, [major, Infinity] for max
 * - Array [major, minor]: returns as-is
 */
function normalizePackFormat(format: PackFormat, isMax: boolean): [number, number] {
	if (Array.isArray(format)) {
		if (format.length === 1) {
			return isMax ? [format[0], Infinity] : [format[0], 0]
		}
		return [format[0], format[1]]
	}
	return isMax ? [format, Infinity] : [format, 0]
}

/**
 * Compares two pack formats [major, minor].
 * Returns: -1 if a < b, 0 if equal, 1 if a > b
 */
function comparePackFormats(a: [number, number], b: [number, number]): number {
	if (a[0] !== b[0]) return a[0] - b[0]
	return a[1] - b[1]
}

/**
 * Checks if a format number falls within the min/max range.
 */
function isFormatInRange(
	format: number,
	minFormat: [number, number],
	maxFormat: [number, number],
): boolean {
	// Check if the major version matches
	if (format < minFormat[0] || format > maxFormat[0]) {
		return false
	}

	// If major version is exactly min or max, we need to check minor version
	// For entries in our map, we treat them as [major, 0]
	const formatTuple: [number, number] = [format, 0]

	// If the format has a decimal (like 69.0, 88.0), extract it
	const formatStr = format.toString()
	if (formatStr.includes('.')) {
		const [maj, min] = formatStr.split('.').map(Number)
		formatTuple[0] = maj
		formatTuple[1] = min
	}

	return (
		comparePackFormats(formatTuple, minFormat) >= 0 &&
		comparePackFormats(formatTuple, maxFormat) <= 0
	)
}

/**
 * Helper function to get a range of game versions between two versions.
 */
function getRange(versionA: string, versionB: string, gameVersions: GameVersion[]): string[] {
	const startingIndex = gameVersions.findIndex((x) => x.version === versionA)
	const endingIndex = gameVersions.findIndex((x) => x.version === versionB)

	if (startingIndex === -1 || endingIndex === -1) {
		return []
	}

	const final = []
	const filterOnlyRelease = gameVersions[startingIndex]?.version_type === 'release'

	for (let i = startingIndex; i >= endingIndex; i--) {
		if (gameVersions[i].version_type === 'release' || !filterOnlyRelease) {
			final.push(gameVersions[i].version)
		}
	}

	return final
}

/**
 * Gets game versions from a single pack format number.
 */
function getVersionsFromPackFormat(
	packFormat: number,
	formatMap: Record<number, { min: string; max: string }>,
	gameVersions: GameVersion[],
): string[] {
	const mapping = formatMap[packFormat]
	if (!mapping) {
		return []
	}
	return getRange(mapping.min, mapping.max, gameVersions)
}

/**
 * Gets game versions from a pack format range (min to max inclusive).
 * Supports both integer and [major, minor] format specifications.
 */
function getVersionsFromFormatRange(
	minFormat: PackFormat,
	maxFormat: PackFormat,
	formatMap: Record<number, { min: string; max: string }>,
	gameVersions: GameVersion[],
): string[] {
	const normalizedMin = normalizePackFormat(minFormat, false)
	const normalizedMax = normalizePackFormat(maxFormat, true)

	// Get all format numbers from the map that fall within the range
	const allVersions: string[] = []
	const formatNumbers = Object.keys(formatMap)
		.map(Number)
		.sort((a, b) => a - b)

	for (const format of formatNumbers) {
		if (isFormatInRange(format, normalizedMin, normalizedMax)) {
			const versions = getVersionsFromPackFormat(format, formatMap, gameVersions)
			for (const version of versions) {
				if (!allVersions.includes(version)) {
					allVersions.push(version)
				}
			}
		}
	}

	return allVersions
}

/**
 * Gets game versions from pack.mcmeta metadata.
 * Supports multiple format specifications:
 * - min_format + max_format: Can be integers or [major, minor] arrays (since 25w31a)
 * - supported_formats: Single int, array of ints, or { min_inclusive, max_inclusive }
 * - pack_format: Single format number (legacy)
 */
function getGameVersionsFromPackMeta(
	packMeta: any,
	formatMap: Record<number, { min: string; max: string }>,
	gameVersions: GameVersion[],
): string[] {
	const pack = packMeta.pack
	if (!pack) return []

	// Check for min_format and max_format (25w31a+ format)
	// These can be: int (e.g., 82), [int] (e.g., [82]), or [major, minor] (e.g., [88, 0])
	if (pack.min_format !== undefined && pack.max_format !== undefined) {
		return getVersionsFromFormatRange(pack.min_format, pack.max_format, formatMap, gameVersions)
	}

	// Check for supported_formats
	if (pack.supported_formats !== undefined) {
		const formats = pack.supported_formats

		// Single integer: major version
		if (typeof formats === 'number') {
			return getVersionsFromPackFormat(formats, formatMap, gameVersions)
		}

		// Array of integers or [min, max] range
		if (Array.isArray(formats)) {
			if (
				formats.length === 2 &&
				typeof formats[0] === 'number' &&
				typeof formats[1] === 'number'
			) {
				// Could be [major, minor] or [minMajor, maxMajor]
				// Based on context, if both are close (within ~50), treat as major version range
				// Otherwise, treat as [major, minor]
				if (Math.abs(formats[1] - formats[0]) < 50) {
					// Likely a major version range like [42, 45]
					return getVersionsFromFormatRange(formats[0], formats[1], formatMap, gameVersions)
				}
			}

			// Array of major versions
			const allVersions: string[] = []
			for (const format of formats) {
				if (typeof format === 'number') {
					const versions = getVersionsFromPackFormat(format, formatMap, gameVersions)
					for (const version of versions) {
						if (!allVersions.includes(version)) {
							allVersions.push(version)
						}
					}
				}
			}
			return allVersions
		}

		// Object format: { min_inclusive, max_inclusive }
		if (
			typeof formats === 'object' &&
			formats.min_inclusive !== undefined &&
			formats.max_inclusive !== undefined
		) {
			return getVersionsFromFormatRange(
				formats.min_inclusive,
				formats.max_inclusive,
				formatMap,
				gameVersions,
			)
		}
	}

	// Fall back to pack_format (legacy single format)
	if (pack.pack_format !== undefined) {
		return getVersionsFromPackFormat(pack.pack_format, formatMap, gameVersions)
	}

	return []
}

/**
 * Creates the pack.mcmeta parser function.
 */
export function createPackParser(project: Project, gameVersions: GameVersion[], rawFile: RawFile) {
	return async (file: string, zip: JSZip): Promise<InferredVersionInfo> => {
		const metadata = JSON.parse(file) as any

		// Check for assets/ directory (resource pack) or data/ directory (data pack)
		const hasAssetsDir = zip.file(/^assets\//)?.[0] !== undefined
		const hasDataDir = zip.file(/^data\//)?.[0] !== undefined
		const hasZipExtension = rawFile.name.toLowerCase().endsWith('.zip')

		const loaders: string[] = []
		let newGameVersions: string[] = []

		// Data pack detection: has data/ directory
		if (hasDataDir && hasZipExtension) {
			loaders.push('datapack')
			newGameVersions = getGameVersionsFromPackMeta(metadata, DATA_PACK_FORMATS, gameVersions)
		}
		// Resource pack detection: has assets/ directory
		else if (hasAssetsDir && hasZipExtension) {
			loaders.push('minecraft')
			newGameVersions = getGameVersionsFromPackMeta(metadata, RESOURCE_PACK_FORMATS, gameVersions)
		}

		// Fallback to old behavior based on project type
		else if (project.actualProjectType === 'mod') {
			loaders.push('datapack')
			newGameVersions = getGameVersionsFromPackMeta(metadata, DATA_PACK_FORMATS, gameVersions)
		} else if (project.actualProjectType === 'resourcepack') {
			loaders.push('minecraft')
			newGameVersions = getGameVersionsFromPackMeta(metadata, RESOURCE_PACK_FORMATS, gameVersions)
		}

		// Try to extract version from filename
		const versionNum = extractVersionFromFilename(rawFile.name)

		return {
			name: versionNum ? `${project.title} ${versionNum}` : undefined,
			version_number: versionNum || undefined,
			version_type: versionType(versionNum),
			loaders,
			game_versions: newGameVersions,
		}
	}
}
