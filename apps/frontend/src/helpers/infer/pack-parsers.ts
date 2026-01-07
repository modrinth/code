import type JSZip from 'jszip'

import { DATA_PACK_FORMATS, RESOURCE_PACK_FORMATS } from './constants'
import type { GameVersion, InferredVersionInfo, Project, RawFile } from './infer'
import { extractVersionFromFilename, versionType } from './version-utils'

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
 */
function getVersionsFromFormatRange(
	minFormat: number,
	maxFormat: number,
	formatMap: Record<number, { min: string; max: string }>,
	gameVersions: GameVersion[],
): string[] {
	// Find all versions between minFormat and maxFormat
	const allVersions: string[] = []
	for (let format = minFormat; format <= maxFormat; format++) {
		const versions = getVersionsFromPackFormat(format, formatMap, gameVersions)
		for (const version of versions) {
			if (!allVersions.includes(version)) {
				allVersions.push(version)
			}
		}
	}
	return allVersions
}

/**
 * Gets game versions from pack.mcmeta metadata.
 */
function getGameVersionsFromPackMeta(
	packMeta: any,
	formatMap: Record<number, { min: string; max: string }>,
	gameVersions: GameVersion[],
): string[] {
	const pack = packMeta.pack
	if (!pack) return []

	// Check for supported_formats (array of [min, max] or object with min_inclusive/max_inclusive)
	if (pack.supported_formats) {
		if (Array.isArray(pack.supported_formats)) {
			// Array format: [minFormat, maxFormat]
			const [minFormat, maxFormat] = pack.supported_formats
			return getVersionsFromFormatRange(minFormat, maxFormat, formatMap, gameVersions)
		} else if (typeof pack.supported_formats === 'object') {
			// Object format: { min_inclusive: X, max_inclusive: Y }
			const minFormat = pack.supported_formats.min_inclusive
			const maxFormat = pack.supported_formats.max_inclusive
			if (minFormat !== undefined && maxFormat !== undefined) {
				return getVersionsFromFormatRange(minFormat, maxFormat, formatMap, gameVersions)
			}
		}
	}

	// Check for min_format and max_format (older format range spec)
	if (pack.min_format !== undefined && pack.max_format !== undefined) {
		return getVersionsFromFormatRange(pack.min_format, pack.max_format, formatMap, gameVersions)
	}

	// Fall back to pack_format
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
