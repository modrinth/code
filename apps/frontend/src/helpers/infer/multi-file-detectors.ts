import type JSZip from 'jszip'

import type { GameVersion, InferredVersionInfo, Project, RawFile } from './infer'
import { extractVersionFromFilename, versionType } from './version-utils'

/**
 * Creates multi-file detection functions that scan multiple files in a zip.
 */
export function createMultiFileDetectors(
	project: Project,
	gameVersions: GameVersion[],
	rawFile: RawFile,
) {
	return {
		// Legacy texture pack (pre-1.6.1)
		legacyTexturePack: async (zip: JSZip): Promise<InferredVersionInfo | null> => {
			const packTxt = zip.file('pack.txt')
			if (!packTxt) return null

			// Check for legacy texture pack files/directories
			const legacyIndicators = [
				'font.txt',
				'particles.png',
				'achievement/',
				'armor/',
				'art/',
				'environment/',
				'font/',
				'gui/',
				'item/',
				'lang/',
				'misc/',
				'mob/',
				'textures/',
				'title/',
			]

			const hasLegacyContent = legacyIndicators.some((indicator) => {
				if (indicator.endsWith('/')) {
					return zip.file(new RegExp(`^${indicator}`))?.length > 0
				}
				return zip.file(indicator) !== null
			})

			if (!hasLegacyContent) return null

			// Legacy texture packs are compatible with a1.2.2 to 1.5.2
			// We'll return versions from 1.0 to 1.5.2 (as older alpha/beta versions may not be in gameVersions)
			const legacyVersions = gameVersions
				.filter((v) => {
					const version = v.version
					// Match 1.0 through 1.5.2
					if (version.match(/^1\.[0-4](\.\d+)?$/) || version.match(/^1\.5(\.[0-2])?$/)) {
						return true
					}
					return false
				})
				.map((v) => v.version)

			const versionNum = extractVersionFromFilename(rawFile.name)

			return {
				name: versionNum ? `${project.title} ${versionNum}` : undefined,
				version_number: versionNum || undefined,
				version_type: versionType(versionNum),
				loaders: ['minecraft'],
				game_versions: legacyVersions,
			}
		},

		// Shader pack (OptiFine/Iris)
		shaderPack: async (zip: JSZip): Promise<InferredVersionInfo | null> => {
			const shadersDir = zip.file(/^shaders\//)
			if (!shadersDir || shadersDir.length === 0) return null

			const loaders: string[] = []

			// Check for Iris-specific features in shaders.properties
			const shaderProps = zip.file('shaders/shaders.properties')
			if (shaderProps) {
				const propsText = await shaderProps.async('text')
				if (
					propsText.includes('iris.features.required') ||
					propsText.includes('iris.features.optional')
				) {
					loaders.push('iris', 'optifine')
				}
			}

			// If no specific loader detected, it could be OptiFine or Iris
			if (loaders.length === 0) {
				loaders.push('optifine', 'iris')
			}

			const versionNum = extractVersionFromFilename(rawFile.name)

			return {
				name: versionNum ? `${project.title} ${versionNum}` : undefined,
				version_number: versionNum || undefined,
				version_type: versionType(versionNum),
				loaders,
				game_versions: [],
			}
		},

		// NilLoader mod
		nilLoaderMod: async (zip: JSZip): Promise<InferredVersionInfo | null> => {
			const nilModFiles = zip.file(/\.nilmod\.css$/)
			if (!nilModFiles || nilModFiles.length === 0) return null

			return {
				loaders: ['nilloader'],
				game_versions: [],
			}
		},

		// Java Agent
		javaAgent: async (zip: JSZip): Promise<InferredVersionInfo | null> => {
			const manifest = zip.file('META-INF/MANIFEST.MF')
			if (!manifest) return null

			const manifestText = await manifest.async('text')
			if (!manifestText.includes('Premain-Class:')) return null

			return {
				loaders: ['java-agent'],
				game_versions: [],
			}
		},
	}
}
