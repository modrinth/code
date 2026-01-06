import { parse as parseTOML } from '@ltd/j-toml'
import yaml from 'js-yaml'
import JSZip from 'jszip'
import { satisfies } from 'semver'

// Pack format to Minecraft version mappings
// See: https://minecraft.wiki/w/Pack_format

// Resource pack format history (full table including development versions)
const RESOURCE_PACK_FORMATS = {
	1: { min: '1.6.1', max: '1.8.9' },
	2: { min: '1.9', max: '1.10.2' },
	3: { min: '1.11', max: '1.12.2' },
	4: { min: '1.13', max: '1.14.4' },
	5: { min: '1.15', max: '1.16.1' },
	6: { min: '1.16.2', max: '1.16.5' },
	7: { min: '1.17', max: '1.17.1' },
	8: { min: '1.18', max: '1.18.2' },
	9: { min: '1.19', max: '1.19.2' },
	11: { min: '22w42a', max: '22w44a' },
	12: { min: '1.19.3', max: '1.19.3' },
	13: { min: '1.19.4', max: '1.19.4' },
	14: { min: '23w14a', max: '23w16a' },
	15: { min: '1.20', max: '1.20.1' },
	16: { min: '23w31a', max: '23w31a' },
	17: { min: '23w32a', max: '1.20.2-pre1' },
	18: { min: '1.20.2', max: '1.20.2' },
	19: { min: '23w42a', max: '23w42a' },
	20: { min: '23w43a', max: '23w44a' },
	21: { min: '23w45a', max: '23w46a' },
	22: { min: '1.20.3', max: '1.20.4' },
	24: { min: '24w03a', max: '24w04a' },
	25: { min: '24w05a', max: '24w05b' },
	26: { min: '24w06a', max: '24w07a' },
	28: { min: '24w09a', max: '24w10a' },
	29: { min: '24w11a', max: '24w11a' },
	30: { min: '24w12a', max: '24w12a' },
	31: { min: '24w13a', max: '1.20.5-pre3' },
	32: { min: '1.20.5', max: '1.20.6' },
	33: { min: '24w18a', max: '24w20a' },
	34: { min: '1.21', max: '1.21.1' },
	35: { min: '24w33a', max: '24w33a' },
	36: { min: '24w34a', max: '24w35a' },
	37: { min: '24w36a', max: '24w36a' },
	38: { min: '24w37a', max: '24w37a' },
	39: { min: '24w38a', max: '24w39a' },
	40: { min: '24w40a', max: '24w40a' },
	41: { min: '1.21.2-pre1', max: '1.21.2-pre2' },
	42: { min: '1.21.2', max: '1.21.3' },
	43: { min: '24w44a', max: '24w44a' },
	44: { min: '24w45a', max: '24w45a' },
	45: { min: '24w46a', max: '24w46a' },
	46: { min: '1.21.4', max: '1.21.4' },
}

// Data pack format history (full table including development versions)
const DATA_PACK_FORMATS = {
	4: { min: '1.13', max: '1.14.4' },
	5: { min: '1.15', max: '1.16.1' },
	6: { min: '1.16.2', max: '1.16.5' },
	7: { min: '1.17', max: '1.17.1' },
	8: { min: '1.18', max: '1.18.1' },
	9: { min: '1.18.2', max: '1.18.2' },
	10: { min: '1.19', max: '1.19.3' },
	11: { min: '23w03a', max: '23w05a' },
	12: { min: '1.19.4', max: '1.19.4' },
	13: { min: '23w12a', max: '23w14a' },
	14: { min: '23w16a', max: '23w17a' },
	15: { min: '1.20', max: '1.20.1' },
	16: { min: '23w31a', max: '23w31a' },
	17: { min: '23w32a', max: '1.20.2-pre1' },
	18: { min: '1.20.2', max: '1.20.2' },
	19: { min: '23w40a', max: '23w40a' },
	20: { min: '23w41a', max: '23w41a' },
	21: { min: '23w42a', max: '23w42a' },
	22: { min: '23w43a', max: '23w44a' },
	23: { min: '23w45a', max: '23w46a' },
	24: { min: '1.20.3-pre1', max: '1.20.3-pre1' },
	25: { min: '1.20.3-pre2', max: '1.20.3-pre4' },
	26: { min: '1.20.3', max: '1.20.4' },
	27: { min: '23w51a', max: '23w51b' },
	28: { min: '24w03a', max: '24w04a' },
	29: { min: '24w05a', max: '24w05b' },
	30: { min: '24w06a', max: '24w06a' },
	31: { min: '24w07a', max: '24w07a' },
	32: { min: '24w09a', max: '24w10a' },
	33: { min: '24w11a', max: '24w11a' },
	34: { min: '24w12a', max: '24w12a' },
	35: { min: '24w13a', max: '24w13a' },
	36: { min: '24w14a', max: '24w14a' },
	37: { min: '1.20.5-pre1', max: '1.20.5-pre1' },
	38: { min: '1.20.5-pre2', max: '1.20.5-pre3' },
	39: { min: '1.20.5-pre4', max: '1.20.5-rc3' },
	40: { min: '1.20.5-rc4', max: '1.20.5-rc4' },
	41: { min: '1.20.5', max: '1.20.6' },
	42: { min: '24w18a', max: '24w19b' },
	43: { min: '24w20a', max: '24w20a' },
	44: { min: '24w21a', max: '24w21b' },
	45: { min: '1.21-pre1', max: '1.21-pre1' },
	46: { min: '1.21-pre2', max: '1.21-pre4' },
	47: { min: '1.21-rc1', max: '1.21-rc1' },
	48: { min: '1.21', max: '1.21.1' },
	49: { min: '24w33a', max: '24w33a' },
	50: { min: '24w34a', max: '24w35a' },
	51: { min: '24w36a', max: '24w36a' },
	52: { min: '24w37a', max: '24w37a' },
	53: { min: '24w38a', max: '24w38a' },
	54: { min: '24w39a', max: '24w39a' },
	55: { min: '24w40a', max: '24w40a' },
	56: { min: '1.21.2-pre1', max: '1.21.2-pre2' },
	57: { min: '1.21.2', max: '1.21.3' },
	58: { min: '24w44a', max: '24w44a' },
	59: { min: '24w45a', max: '24w45a' },
	60: { min: '24w46a', max: '24w46a' },
	61: { min: '1.21.4', max: '1.21.4' },
}

export const inferVersionInfo = async function (rawFile, project, gameVersions) {
	function versionType(number) {
		if (!number) return 'release'
		if (number.includes('alpha')) {
			return 'alpha'
		} else if (
			number.includes('beta') ||
			number.match(/[^A-z](rc)[^A-z]/) || // includes `rc`
			number.match(/[^A-z](pre)[^A-z]/) // includes `pre`
		) {
			return 'beta'
		} else {
			return 'release'
		}
	}

	/**
	 * Extracts version number from a filename.
	 * Handles patterns like:
	 * - "Bare Bones 1.21.11.zip" -> "1.21.11"
	 * - "FA+All_Extensions-v1.7.zip" -> "1.7"
	 * - "FreshAnimations_v1.10.3.zip" -> "1.10.3"
	 * - "LowOnFire v1.21.1158.zip" -> "1.21.1158"
	 * - "Dramatic Skys Demo 1.5.3.36.2.zip" -> "1.5.3.36.2"
	 */
	function extractVersionFromFilename(filename) {
		if (!filename) return null

		// Remove file extension
		const baseName = filename.replace(/\.(zip|jar)$/i, '')

		// Try to match version patterns:
		// 1. "v" followed by version number (v1.2.3)
		// 2. Version number at the end after separator (space, hyphen, underscore)
		// Pattern matches version-like strings: digits separated by dots, possibly with additional segments
		const versionPatterns = [
			// Match "v1.2.3" or "V1.2.3" style versions
			/[_\-\s]v(\d+(?:\.\d+)*(?:\.\d+)?)$/i,
			// Match version at end after space/separator: "Name 1.2.3"
			/[_\-\s](\d+(?:\.\d+)+)$/,
			// Match version with 'v' anywhere: "Name-v1.2.3-extra" (less strict)
			/[_\-\s]v(\d+(?:\.\d+)*)/i,
		]

		for (const pattern of versionPatterns) {
			const match = baseName.match(pattern)
			if (match && match[1]) {
				return match[1]
			}
		}

		return null
	}

	function getGameVersionsMatchingSemverRange(range, gameVersions) {
		if (!range) {
			return []
		}
		const ranges = Array.isArray(range) ? range : [range]
		return gameVersions.filter((version) => {
			const semverVersion = version.split('.').length === 2 ? `${version}.0` : version // add patch version if missing (e.g. 1.16 -> 1.16.0)
			return ranges.some((v) => satisfies(semverVersion, v))
		})
	}

	function getGameVersionsMatchingMavenRange(range, gameVersions) {
		if (!range) {
			return []
		}
		const ranges = []

		while (range.startsWith('[') || range.startsWith('(')) {
			let index = range.indexOf(')')
			const index2 = range.indexOf(']')
			if (index === -1 || (index2 !== -1 && index2 < index)) {
				index = index2
			}
			if (index === -1) break
			ranges.push(range.substring(0, index + 1))
			range = range.substring(index + 1).trim()
			if (range.startsWith(',')) {
				range = range.substring(1).trim()
			}
		}

		if (range) {
			ranges.push(range)
		}

		const LESS_THAN_EQUAL = /^\(,(.*)]$/
		const LESS_THAN = /^\(,(.*)\)$/
		const EQUAL = /^\[(.*)]$/
		const GREATER_THAN_EQUAL = /^\[(.*),\)$/
		const GREATER_THAN = /^\((.*),\)$/
		const BETWEEN = /^\((.*),(.*)\)$/
		const BETWEEN_EQUAL = /^\[(.*),(.*)]$/
		const BETWEEN_LESS_THAN_EQUAL = /^\((.*),(.*)]$/
		const BETWEEN_GREATER_THAN_EQUAL = /^\[(.*),(.*)\)$/

		const semverRanges = []

		for (const range of ranges) {
			let result
			if ((result = range.match(LESS_THAN_EQUAL))) {
				semverRanges.push(`<=${result[1]}`)
			} else if ((result = range.match(LESS_THAN))) {
				semverRanges.push(`<${result[1]}`)
			} else if ((result = range.match(EQUAL))) {
				semverRanges.push(`${result[1]}`)
			} else if ((result = range.match(GREATER_THAN_EQUAL))) {
				semverRanges.push(`>=${result[1]}`)
			} else if ((result = range.match(GREATER_THAN))) {
				semverRanges.push(`>${result[1]}`)
			} else if ((result = range.match(BETWEEN))) {
				semverRanges.push(`>${result[1]} <${result[2]}`)
			} else if ((result = range.match(BETWEEN_EQUAL))) {
				semverRanges.push(`>=${result[1]} <=${result[2]}`)
			} else if ((result = range.match(BETWEEN_LESS_THAN_EQUAL))) {
				semverRanges.push(`>${result[1]} <=${result[2]}`)
			} else if ((result = range.match(BETWEEN_GREATER_THAN_EQUAL))) {
				semverRanges.push(`>=${result[1]} <${result[2]}`)
			}
		}
		return getGameVersionsMatchingSemverRange(semverRanges, gameVersions)
	}

	const simplifiedGameVersions = gameVersions
		.filter((it) => it.version_type === 'release')
		.map((it) => it.version)

	const inferFunctions = {
		// NeoForge
		'META-INF/neoforge.mods.toml': (file) => {
			const metadata = parseTOML(file, { joiner: '\n' })
			if (!metadata.mods || metadata.mods.length === 0) {
				return {}
			}

			const neoForgeDependency = Object.values(metadata.dependencies)
				.flat()
				.find((dependency) => dependency.modId === 'neoforge')
			if (!neoForgeDependency) {
				return {}
			}

			// https://docs.neoforged.net/docs/gettingstarted/versioning/#neoforge
			const mcVersionRange = neoForgeDependency.versionRange
				.replace('-beta', '')
				.replace(/(\d+)(?:\.(\d+))?(?:\.(\d+)?)?/g, (_match, major, minor) => {
					return `1.${major}${minor ? '.' + minor : ''}`
				})
			const gameVersions = getGameVersionsMatchingMavenRange(mcVersionRange, simplifiedGameVersions)

			const versionNum = metadata.mods[0].version
			return {
				name: `${project.title} ${versionNum}`,
				version_number: versionNum,
				loaders: ['neoforge'],
				version_type: versionType(versionNum),
				game_versions: gameVersions,
			}
		},
		// Forge 1.13+
		'META-INF/mods.toml': async (file, zip) => {
			const metadata = parseTOML(file, { joiner: '\n' })

			if (metadata.mods && metadata.mods.length > 0) {
				let versionNum = metadata.mods[0].version

				// ${file.jarVersion} -> Implementation-Version from manifest
				const manifestFile = zip.file('META-INF/MANIFEST.MF')
				if (metadata.mods[0].version.includes('${file.jarVersion}') && manifestFile !== null) {
					const manifestText = await manifestFile.async('text')
					const regex = /Implementation-Version: (.*)$/m
					const match = manifestText.match(regex)
					if (match) {
						versionNum = versionNum.replace('${file.jarVersion}', match[1])
					}
				}

				let gameVersions = []
				const mcDependencies = Object.values(metadata.dependencies)
					.flat()
					.filter((dependency) => dependency.modId === 'minecraft')

				if (mcDependencies.length > 0) {
					gameVersions = getGameVersionsMatchingMavenRange(
						mcDependencies[0].versionRange,
						simplifiedGameVersions,
					)
				}

				return {
					name: `${project.title} ${versionNum}`,
					version_number: versionNum,
					version_type: versionType(versionNum),
					loaders: ['forge'],
					game_versions: gameVersions,
				}
			} else {
				return {}
			}
		},
		// Old Forge
		'mcmod.info': (file) => {
			const metadata = JSON.parse(file)

			return {
				name: metadata.version ? `${project.title} ${metadata.version}` : '',
				version_number: metadata.version,
				version_type: versionType(metadata.version),
				loaders: ['forge'],
				game_versions: simplifiedGameVersions.filter((version) =>
					version.startsWith(metadata.mcversion),
				),
			}
		},
		// Fabric
		'fabric.mod.json': (file) => {
			const metadata = JSON.parse(file)

			return {
				name: `${project.title} ${metadata.version}`,
				version_number: metadata.version,
				loaders: ['fabric'],
				version_type: versionType(metadata.version),
				game_versions: metadata.depends
					? getGameVersionsMatchingSemverRange(metadata.depends.minecraft, simplifiedGameVersions)
					: [],
			}
		},
		// Quilt
		'quilt.mod.json': (file) => {
			const metadata = JSON.parse(file)

			return {
				name: `${project.title} ${metadata.quilt_loader.version}`,
				version_number: metadata.quilt_loader.version,
				loaders: ['quilt'],
				version_type: versionType(metadata.quilt_loader.version),
				game_versions: metadata.quilt_loader.depends
					? getGameVersionsMatchingSemverRange(
							metadata.quilt_loader.depends.find((x) => x.id === 'minecraft')
								? metadata.quilt_loader.depends.find((x) => x.id === 'minecraft').versions
								: [],
							simplifiedGameVersions,
						)
					: [],
			}
		},
		// Bukkit + Other Forks
		'plugin.yml': (file) => {
			const metadata = yaml.load(file)

			// Check for Folia support
			const loaders = []
			if (metadata['folia-supported'] === true) {
				loaders.push('folia')
			}
			// We don't know which fork of Bukkit users are using otherwise

			return {
				name: `${project.title} ${metadata.version}`,
				version_number: metadata.version,
				version_type: versionType(metadata.version),
				loaders,
				game_versions: gameVersions
					.filter(
						(x) => x.version.startsWith(metadata['api-version']) && x.version_type === 'release',
					)
					.map((x) => x.version),
			}
		},
		// Paper 1.19.3+
		'paper-plugin.yml': (file) => {
			const metadata = yaml.load(file)

			return {
				name: `${project.title} ${metadata.version}`,
				version_number: metadata.version,
				version_type: versionType(metadata.version),
				loaders: ['paper'],
				game_versions: gameVersions
					.filter(
						(x) => x.version.startsWith(metadata['api-version']) && x.version_type === 'release',
					)
					.map((x) => x.version),
			}
		},
		// Bungeecord + Waterfall
		'bungee.yml': (file) => {
			const metadata = yaml.load(file)

			return {
				name: `${project.title} ${metadata.version}`,
				version_number: metadata.version,
				version_type: versionType(metadata.version),
				loaders: ['bungeecord'],
			}
		},
		// Velocity
		'velocity-plugin.json': (file) => {
			const metadata = JSON.parse(file)

			return {
				name: `${project.title} ${metadata.version}`,
				version_number: metadata.version,
				version_type: versionType(metadata.version),
				loaders: ['velocity'],
			}
		},
		// Sponge plugin (8+)
		'META-INF/sponge_plugins.json': (file) => {
			const metadata = JSON.parse(file)
			const plugin = metadata.plugins?.[0]

			if (!plugin) {
				return {}
			}

			return {
				name: plugin.version ? `${project.title} ${plugin.version}` : '',
				version_number: plugin.version,
				version_type: versionType(plugin.version),
				loaders: ['sponge'],
			}
		},
		// Modpacks
		'modrinth.index.json': (file) => {
			const metadata = JSON.parse(file)

			const loaders = []
			if ('forge' in metadata.dependencies) {
				loaders.push('forge')
			}
			if ('neoforge' in metadata.dependencies) {
				loaders.push('neoforge')
			}
			if ('fabric-loader' in metadata.dependencies) {
				loaders.push('fabric')
			}
			if ('quilt-loader' in metadata.dependencies) {
				loaders.push('quilt')
			}

			return {
				name: `${project.title} ${metadata.versionId}`,
				version_number: metadata.versionId,
				version_type: versionType(metadata.versionId),
				loaders,
				game_versions: gameVersions
					.filter((x) => x.version === metadata.dependencies.minecraft)
					.map((x) => x.version),
			}
		},
		// Resource Packs + Data Packs
		'pack.mcmeta': async (file, zip) => {
			const metadata = JSON.parse(file)

			function getRange(versionA, versionB) {
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

			function getVersionsFromPackFormat(packFormat, formatMap) {
				const mapping = formatMap[packFormat]
				if (!mapping) {
					return []
				}
				return getRange(mapping.min, mapping.max)
			}

			function getVersionsFromFormatRange(minFormat, maxFormat, formatMap) {
				// Find all versions between minFormat and maxFormat
				const allVersions = []
				for (let format = minFormat; format <= maxFormat; format++) {
					const versions = getVersionsFromPackFormat(format, formatMap)
					for (const version of versions) {
						if (!allVersions.includes(version)) {
							allVersions.push(version)
						}
					}
				}
				return allVersions
			}

			function getGameVersionsFromPackMeta(packMeta, formatMap) {
				const pack = packMeta.pack
				if (!pack) return []

				// Check for supported_formats (array of [min, max] or object with min_inclusive/max_inclusive)
				if (pack.supported_formats) {
					if (Array.isArray(pack.supported_formats)) {
						// Array format: [minFormat, maxFormat]
						const [minFormat, maxFormat] = pack.supported_formats
						return getVersionsFromFormatRange(minFormat, maxFormat, formatMap)
					} else if (typeof pack.supported_formats === 'object') {
						// Object format: { min_inclusive: X, max_inclusive: Y }
						const minFormat = pack.supported_formats.min_inclusive
						const maxFormat = pack.supported_formats.max_inclusive
						if (minFormat !== undefined && maxFormat !== undefined) {
							return getVersionsFromFormatRange(minFormat, maxFormat, formatMap)
						}
					}
				}

				// Check for min_format and max_format (older format range spec)
				if (pack.min_format !== undefined && pack.max_format !== undefined) {
					return getVersionsFromFormatRange(pack.min_format, pack.max_format, formatMap)
				}

				// Fall back to pack_format
				if (pack.pack_format !== undefined) {
					return getVersionsFromPackFormat(pack.pack_format, formatMap)
				}

				return []
			}

			// Check for assets/ directory (resource pack) or data/ directory (data pack)
			const hasAssetsDir = zip.file(/^assets\//)?.[0] !== undefined
			const hasDataDir = zip.file(/^data\//)?.[0] !== undefined

			// Detect vanilla shaders
			const hasVanillaShaders = await (async () => {
				// Check for assets/*/shaders directory
				const shaderDirs = zip.file(/^assets\/[^/]+\/shaders\//)
				if (shaderDirs && shaderDirs.length > 0) {
					return true
				}
				// Check for shader files (.fsh, .vsh, .glsl)
				const shaderFiles = zip.file(/\.(fsh|vsh|glsl)$/)
				return shaderFiles && shaderFiles.length > 0
			})()

			const loaders = []
			let newGameVersions = []

			// Data pack detection: has data/ directory
			if (
				hasDataDir &&
				(project.actualProjectType === 'mod' || project.actualProjectType === 'datapack')
			) {
				loaders.push('datapack')
				newGameVersions = getGameVersionsFromPackMeta(metadata, DATA_PACK_FORMATS)
			}
			// Resource pack detection: has assets/ directory
			else if (
				hasAssetsDir &&
				(project.actualProjectType === 'resourcepack' || project.actualProjectType === 'shader')
			) {
				if (hasVanillaShaders && project.actualProjectType === 'shader') {
					loaders.push('vanilla')
				} else {
					loaders.push('minecraft')
				}
				newGameVersions = getGameVersionsFromPackMeta(metadata, RESOURCE_PACK_FORMATS)
			}
			// Fallback to old behavior based on project type
			else if (project.actualProjectType === 'mod') {
				loaders.push('datapack')
				newGameVersions = getGameVersionsFromPackMeta(metadata, DATA_PACK_FORMATS)
			} else if (project.actualProjectType === 'resourcepack') {
				loaders.push('minecraft')
				newGameVersions = getGameVersionsFromPackMeta(metadata, RESOURCE_PACK_FORMATS)
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
		},
	}

	// Additional detection functions that need to scan multiple files
	const multiFileInferFunctions = {
		// Legacy texture pack (pre-1.6.1)
		legacyTexturePack: async (zip) => {
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

			// Try to extract version from filename
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
		shaderPack: async (zip) => {
			const shadersDir = zip.file(/^shaders\//)
			if (!shadersDir || shadersDir.length === 0) return null

			const loaders = []

			// Check for Iris-specific features in shaders.properties
			const shaderProps = zip.file('shaders/shaders.properties')
			if (shaderProps) {
				const propsText = await shaderProps.async('text')
				if (
					propsText.includes('iris.features.required') ||
					propsText.includes('iris.features.optional')
				) {
					loaders.push('iris')
				}
			}

			// If no specific loader detected, it could be OptiFine or Iris
			if (loaders.length === 0) {
				loaders.push('optifine', 'iris')
			}

			// Try to extract version from filename
			const versionNum = extractVersionFromFilename(rawFile.name)

			return {
				name: versionNum ? `${project.title} ${versionNum}` : undefined,
				version_number: versionNum || undefined,
				version_type: versionType(versionNum),
				loaders,
				// No reliable way to detect MC versions for shader packs
				game_versions: [],
			}
		},

		// NilLoader mod
		nilLoaderMod: async (zip) => {
			const nilModFiles = zip.file(/\.nilmod\.css$/)
			if (!nilModFiles || nilModFiles.length === 0) return null

			return {
				loaders: ['nilloader'],
				game_versions: [],
			}
		},

		// Java Agent
		javaAgent: async (zip) => {
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

	const zipReader = new JSZip()

	const zip = await zipReader.loadAsync(rawFile)

	// First, try the standard file-based detection
	for (const fileName in inferFunctions) {
		const file = zip.file(fileName)

		if (file !== null) {
			const text = await file.async('text')
			return inferFunctions[fileName](text, zip)
		}
	}

	// Then, try multi-file detection functions
	for (const funcName in multiFileInferFunctions) {
		const result = await multiFileInferFunctions[funcName](zip)
		if (result !== null) {
			return result
		}
	}

	return {}
}
