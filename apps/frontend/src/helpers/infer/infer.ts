import JSZip from 'jszip'

import { createLoaderParsers } from './loader-parsers'
import { createMultiFileDetectors } from './multi-file-detectors'
import { createPackParser } from './pack-parsers'
import { extractVersionDetailsFromFilename } from './version-utils'

export type GameVersion = { version: string; version_type: string }

export type Project = { title: string; actualProjectType?: string }

export type RawFile = File | (Blob & { name: string })

export interface InferredVersionInfo {
	name?: string
	version_number?: string
	version_type?: 'alpha' | 'beta' | 'release'
	loaders?: string[]
	game_versions?: string[]
}

/**
 * Fills in missing version information from the filename if not already present.
 */
function fillMissingFromFilename(
	result: InferredVersionInfo,
	filename: string,
	projectTitle: string,
): InferredVersionInfo {
	const filenameDetails = extractVersionDetailsFromFilename(filename)

	if (!result.version_number && filenameDetails.versionNumber) {
		result.version_number = filenameDetails.versionNumber
	}

	if (!result.version_type) {
		result.version_type = filenameDetails.versionType
	}

	if (!result.name && result.version_number) {
		result.name = `${projectTitle} ${result.version_number}`
	}

	return result
}

/**
 * Main function to infer version information from a file.
 * Analyzes mod loaders, packs, and other Minecraft-related file formats.
 */
export const inferVersionInfo = async function (
	rawFile: RawFile,
	project: Project,
	gameVersions: GameVersion[],
): Promise<InferredVersionInfo> {
	const simplifiedGameVersions = gameVersions
		.filter((it) => it.version_type === 'release')
		.map((it) => it.version)

	const zipReader = new JSZip()
	const zip = await zipReader.loadAsync(rawFile)

	const loaderParsers = createLoaderParsers(project, gameVersions, simplifiedGameVersions)
	const packParser = createPackParser(project, gameVersions, rawFile)
	const multiFileDetectors = createMultiFileDetectors(project, gameVersions, rawFile)

	const inferFunctions = {
		...loaderParsers,
		'pack.mcmeta': packParser,
	}

	// Multi-loader detection
	const multiLoaderFiles = [
		'META-INF/neoforge.mods.toml',
		'META-INF/mods.toml',
		'fabric.mod.json',
		'quilt.mod.json',
	]
	const detectedLoaderFiles = multiLoaderFiles.filter((fileName) => zip.file(fileName) !== null)
	if (detectedLoaderFiles.length > 1) {
		const results: InferredVersionInfo[] = []
		for (const fileName of detectedLoaderFiles) {
			const file = zip.file(fileName)
			if (file !== null) {
				const text = await file.async('text')
				const parser = inferFunctions[fileName as keyof typeof inferFunctions]
				if (parser) {
					const result = await parser(text, zip)
					if (result && Object.keys(result).length > 0) results.push(result)
				}
			}
		}
		if (results.length > 0) {
			const combinedLoaders = [...new Set(results.flatMap((r) => r.loaders || []))]
			const allGameVersions = [...new Set(results.flatMap((r) => r.game_versions || []))]
			const primaryResult = results.find((r) => r.version_number) || results[0]

			const mergedResult = {
				name: primaryResult.name,
				version_number: primaryResult.version_number,
				version_type: primaryResult.version_type,
				loaders: combinedLoaders,
				game_versions: allGameVersions,
			}
			return fillMissingFromFilename(mergedResult, rawFile.name, project.title)
		}
	}

	// Standard single-loader detection
	for (const fileName in inferFunctions) {
		const file = zip.file(fileName)

		if (file !== null) {
			const text = await file.async('text')
			const parser = inferFunctions[fileName as keyof typeof inferFunctions]
			if (parser) {
				const result = await parser(text, zip)
				return fillMissingFromFilename(result, rawFile.name, project.title)
			}
		}
	}

	// Multi-file detection functions
	for (const detector of Object.values(multiFileDetectors)) {
		const result = await detector(zip)
		if (result !== null) {
			return fillMissingFromFilename(result, rawFile.name, project.title)
		}
	}

	return fillMissingFromFilename({}, rawFile.name, project.title)
}
