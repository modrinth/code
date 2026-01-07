import JSZip from 'jszip'
import { createLoaderParsers } from './loader-parsers'
import { createMultiFileDetectors } from './multi-file-detectors'
import { createPackParser } from './pack-parsers'

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

	// Mod loader metadata files that can coexist in a single jar
	const multiLoaderFiles = [
		'META-INF/neoforge.mods.toml',
		'META-INF/mods.toml',
		'fabric.mod.json',
		'quilt.mod.json',
	]

	const zipReader = new JSZip()
	const zip = await zipReader.loadAsync(rawFile)

	const loaderParsers = createLoaderParsers(project, gameVersions, simplifiedGameVersions)
	const packParser = createPackParser(project, gameVersions, rawFile)
	const multiFileDetectors = createMultiFileDetectors(project, gameVersions, rawFile)

	const inferFunctions = {
		...loaderParsers,
		'pack.mcmeta': packParser,
	}

	// Check for multi-loader jars (mods that support multiple loaders)
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
					if (result && Object.keys(result).length > 0) {
						results.push(result)
					}
				}
			}
		}

		if (results.length > 0) {
			// Merge results: combine loaders, use first valid version info, merge game versions
			const combinedLoaders = [...new Set(results.flatMap((r) => r.loaders || []))]
			const allGameVersions = [...new Set(results.flatMap((r) => r.game_versions || []))]

			const primaryResult = results.find((r) => r.version_number) || results[0]

			return {
				name: primaryResult.name,
				version_number: primaryResult.version_number,
				version_type: primaryResult.version_type,
				loaders: combinedLoaders,
				game_versions: allGameVersions,
			}
		}
	}

	// Standard single-loader detection
	for (const fileName in inferFunctions) {
		const file = zip.file(fileName)

		if (file !== null) {
			const text = await file.async('text')
			const parser = inferFunctions[fileName as keyof typeof inferFunctions]
			if (parser) {
				return await parser(text, zip)
			}
		}
	}

	// Then, try multi-file detection functions
	for (const funcName in multiFileDetectors) {
		const detector = multiFileDetectors[funcName as keyof typeof multiFileDetectors]
		const result = await detector(zip)
		if (result !== null) {
			return result
		}
	}

	return {}
}
