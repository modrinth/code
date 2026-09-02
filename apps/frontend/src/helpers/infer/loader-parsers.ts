import { parse as parseTOML } from '@ltd/j-toml'
import yaml from 'js-yaml'
import type JSZip from 'jszip'

import type { GameVersion, InferredVersionInfo, Project } from './infer'
import {
	getGameVersionsMatchingMavenRange,
	getGameVersionsMatchingSemverRange,
	semverRangeIntersects,
} from './version-ranges'
import { versionType } from './version-utils'

/**
 * Creates the inferFunctions object containing all mod loader parsers.
 */
export function createLoaderParsers(
	project: Project,
	gameVersions: GameVersion[],
	simplifiedGameVersions: string[],
) {
	return {
		// NeoForge
		'META-INF/neoforge.mods.toml': (file: string): InferredVersionInfo => {
			const metadata = parseTOML(file, { joiner: '\n' }) as any

			const versionNum = metadata.mods?.[0]?.version || ''
			let newGameVersions: string[] = []

			if (metadata.dependencies) {
				const neoForgeDependency = Object.values(metadata.dependencies)
					.flat()
					.filter((dependency: any) => dependency.modId === 'neoforge')
					.map((dependency: any) => dependency.versionRange)
					.find((range) => range)
				const minecraftDependency = Object.values(metadata.dependencies)
					.flat()
					.filter((dependency: any) => dependency.modId === 'minecraft')
					.map((dependency: any) => dependency.versionRange)
					.find((range) => range)

				if (minecraftDependency) {
					newGameVersions = getGameVersionsMatchingMavenRange(
						minecraftDependency,
						simplifiedGameVersions,
					)
				} else if (neoForgeDependency) {
					// https://docs.neoforged.net/docs/gettingstarted/versioning/#neoforge
					// NeoForge's versioning changed after Mojang changed from 1.<major>.<minor> to <year>.<drop>.<patch>, so both cases need to be handled
					const neoPre26Regex = /^(?<mc_major>\d+)(?:\.(?<mc_minor>\d+))?(?:\.(?<neo>\d+)?)?$/
					const neoPost26Regex =
						/^(?<mc_year>\d+)(?:\.(?<mc_drop>\d+))?(?:\.(?<mc_patch>\d+)?)?(?:\.(\d+))?$/

					newGameVersions = getGameVersionsMatchingMavenRange(
						neoForgeDependency.replace('-beta', ''),
						simplifiedGameVersions,
						(version) => {
							const matchPre26 = version.match(neoPre26Regex)
							if (matchPre26 && matchPre26.groups) {
								const { mc_major, mc_minor } = matchPre26.groups
								return mc_minor ? `1.${mc_major}.${mc_minor}` : `1.${mc_major}`
							}
							const matchPost26 = version.match(neoPost26Regex)
							if (matchPost26 && matchPost26.groups) {
								const { mc_year, mc_drop, mc_patch } = matchPost26.groups
								return `${mc_year}${mc_drop ? `.${mc_drop}` : ''}${mc_patch ? `.${mc_patch}` : ''}`
							}
							return version
						},
					)
				}
			}

			return {
				name: versionNum ? `${project.title} ${versionNum}` : '',
				version_number: versionNum,
				loaders: ['neoforge'],
				version_type: versionType(versionNum),
				game_versions: newGameVersions,
			}
		},
		// Forge 1.13+
		'META-INF/mods.toml': async (file: string, zip: JSZip): Promise<InferredVersionInfo> => {
			const metadata = parseTOML(file, { joiner: '\n' }) as any

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

				let newGameVersions: string[] = []
				const mcDependencies = Object.values(metadata.dependencies)
					.flat()
					.filter((dependency: any) => dependency.modId === 'minecraft')

				if (mcDependencies.length > 0) {
					newGameVersions = getGameVersionsMatchingMavenRange(
						(mcDependencies[0] as any).versionRange,
						simplifiedGameVersions,
					)
				}

				return {
					name: `${project.title} ${versionNum}`,
					version_number: versionNum,
					version_type: versionType(versionNum),
					loaders: ['forge'],
					game_versions: newGameVersions,
				}
			} else {
				return {}
			}
		},
		// Old Forge
		'mcmod.info': (file: string): InferredVersionInfo => {
			const metadata = JSON.parse(file) as any

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
		// Fabric (or its derivatives: Babric, Legacy Fabric, Ornithe)
		'fabric.mod.json': async (file: string, zip: JSZip): Promise<InferredVersionInfo> => {
			const metadata = JSON.parse(file) as any

			const mcDependency = metadata.depends?.minecraft
			const mcDependencies = Array.isArray(mcDependency) ? mcDependency : [mcDependency]

			let detectedGameVersions = metadata.depends
				? getGameVersionsMatchingSemverRange(mcDependency, simplifiedGameVersions)
				: []
			const loaders: string[] = []

			// Both Legacy Fabric and Ornithe add details about their intermediary to the jar manifest
			const manifestFile = zip.file('META-INF/MANIFEST.MF')
			if (manifestFile !== null) {
				const manifestText = await manifestFile.async('text')
				if (manifestText.match(/^Legacy-Fabric-Intermediary-Version: (.*)$/)) {
					loaders.push('legacy-fabric')
				}
				if (manifestText.match(/^Calamus-Generation: (.*)$/)) {
					loaders.push('ornithe')
				}
			}

			// fall back to version comparison if nothing found in jar manifest
			if (loaders.length == 0) {
				// Fabric and its derivates all support different version ranges
				// - Fabric supports 18w43b and later
				// - Legacy Fabric supports (almost) only release versions 1.3-1.13.2
				// - Ornithe supports versions up to 1.14.4
				// - Babric supports only b1.7.3
				// These ranges overlap but given the jar manifest logic above the only likely overlap scenarios are
				// - a b1.7.3 mod that could be Ornithe but is most likely Babric
				// - a Fabric mod that is dependent on "*" (i.e. all versions)
				// In other cases, resolve as follows:
				// 1. if the dependency falls entirely in >=1.3 <=1.13.2, assume Legacy Fabric
				// 2. if the dependency falls entirely in <=18w43a, assume Ornithe

				// dependency version strings are normalized to be Semver 2.0.0 compliant through
				// https://github.com/FabricMC/fabric-loader/blob/master/minecraft/src/main/java/net/fabricmc/loader/impl/game/minecraft/McVersionLookup.java

				// assume Babric only if dependent on b1.7.3 exactly
				const hasBabricVersion = mcDependencies.every(
					(version: string | undefined) => version == '1.0.0-beta.7.3',
				)

				// Legacy Fabric supports (almost) only release versions which is easy to check
				const hasLegacyFabricVersions = detectedGameVersions.every((version) => {
					const match = version.match(/^1\.(\d+)/)
					return match && parseInt(match[1]) >= 3 && parseInt(match[1]) <= 13
				})

				// Assume Ornithe only if the dependency range falls entirely into <18w43b
				const hasOrnitheVersions = !semverRangeIntersects(mcDependencies, '>=1.14.0-alpha.18.43.b')

				if (hasBabricVersion) {
					loaders.push('babric')
					detectedGameVersions = gameVersions
						.filter((version) => version.version === 'b1.7.3')
						.map((version) => version.version)
				} else if (hasLegacyFabricVersions) {
					loaders.push('legacy-fabric')
				} else if (hasOrnitheVersions) {
					loaders.push('ornithe')
				} else {
					loaders.push('fabric')
				}
			}

			return {
				name: `${project.title} ${metadata.version}`,
				version_number: metadata.version,
				loaders,
				version_type: versionType(metadata.version),
				game_versions: detectedGameVersions,
			}
		},
		// Quilt (or its derivatives: Ornithe)
		'quilt.mod.json': async (file: string, zip: JSZip): Promise<InferredVersionInfo> => {
			const metadata = JSON.parse(file) as any

			const mcDependency = metadata.quilt_loader.depends?.find(
				(x: any) => x.id === 'minecraft',
			)?.versions
			const mcDependencies = Array.isArray(mcDependency) ? mcDependency : [mcDependency]

			const detectedGameVersions = metadata.quilt_loader.depends
				? getGameVersionsMatchingSemverRange(mcDependency, simplifiedGameVersions)
				: []
			const loaders: string[] = []

			// Ornithe add details about their intermediary to the jar manifest
			const manifestFile = zip.file('META-INF/MANIFEST.MF')
			if (manifestFile !== null) {
				const manifestText = await manifestFile.async('text')
				if (manifestText.match(/^Calamus-Generation: (.*)$/)) {
					loaders.push('ornithe')
				}
			}

			// fall back to version comparison if nothing found in jar manifest
			if (loaders.length == 0) {
				// Quilt and its derivates all support different version ranges
				// - Quilt supports 1.14.4 and later
				// - Ornithe supports versions up to 1.14.4
				// These ranges overlap but given the jar manifest logic above
				// we can simply prioritize Quilt for the small overlap range

				// dependency version strings are normalized to be Semver 2.0.0 compliant through
				// https://github.com/QuiltMC/quilt-loader/blob/develop/minecraft/src/main/java/org/quiltmc/loader/impl/game/minecraft/McVersionLookup.java

				// Assume Ornithe only if the dependency range falls entirely into <1.14.4`
				const hasOrnitheVersions = !semverRangeIntersects(mcDependencies, '>=1.14.4')

				if (hasOrnitheVersions) {
					loaders.push('ornithe')
				} else {
					loaders.push('quilt')
				}
			}

			return {
				name: `${project.title} ${metadata.quilt_loader.version}`,
				version_number: metadata.quilt_loader.version,
				loaders,
				version_type: versionType(metadata.quilt_loader.version),
				game_versions: detectedGameVersions,
			}
		},
		// Bukkit + Other Forks
		'plugin.yml': (file: string): InferredVersionInfo => {
			const metadata = yaml.load(file) as any

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
		'paper-plugin.yml': (file: string): InferredVersionInfo => {
			const metadata = yaml.load(file) as any

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
		'bungee.yml': (file: string): InferredVersionInfo => {
			const metadata = yaml.load(file) as any

			return {
				name: `${project.title} ${metadata.version}`,
				version_number: metadata.version,
				version_type: versionType(metadata.version),
				loaders: ['bungeecord'],
			}
		},
		// Velocity
		'velocity-plugin.json': (file: string): InferredVersionInfo => {
			const metadata = JSON.parse(file) as any

			return {
				name: `${project.title} ${metadata.version}`,
				version_number: metadata.version,
				version_type: versionType(metadata.version),
				loaders: ['velocity'],
			}
		},
		// Sponge plugin (8+)
		'META-INF/sponge_plugins.json': (file: string): InferredVersionInfo => {
			const metadata = JSON.parse(file) as any
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
		// Geyser Extensions
		'extension.yml': (file: string): InferredVersionInfo => {
			const metadata = yaml.load(file) as any

			return {
				name: metadata.version ? `${project.title} ${metadata.version}` : '',
				version_number: metadata.version,
				version_type: versionType(metadata.version),
				loaders: ['geyser'],
			}
		},
		// Modpacks
		'modrinth.index.json': (file: string): InferredVersionInfo => {
			const metadata = JSON.parse(file) as any

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
	}
}
