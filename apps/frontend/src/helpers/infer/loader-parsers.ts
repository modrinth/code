import { parse as parseTOML } from '@ltd/j-toml'
import yaml from 'js-yaml'
import type JSZip from 'jszip'

import type { GameVersion, InferredVersionInfo, Project } from './infer'
import {
	getGameVersionsMatchingMavenRange,
	getGameVersionsMatchingSemverRange,
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
					.find((dependency: any) => dependency.modId === 'neoforge')

				if (neoForgeDependency) {
					try {
						// https://docs.neoforged.net/docs/gettingstarted/versioning/#neoforge
						const mcVersionRange = (neoForgeDependency as any).versionRange
							.replace('-beta', '')
							.replace(
								/(\d+)(?:\.(\d+))?(?:\.(\d+)?)?/g,
								(_match: string, major: string, minor: string) => {
									return `1.${major}${minor ? '.' + minor : ''}`
								},
							)
						newGameVersions = getGameVersionsMatchingMavenRange(
							mcVersionRange,
							simplifiedGameVersions,
						)
					} catch {
						// Ignore parsing errors, just leave game_versions empty
					}
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
		// Fabric
		'fabric.mod.json': (file: string): InferredVersionInfo => {
			const metadata = JSON.parse(file) as any

			const detectedGameVersions = metadata.depends
				? getGameVersionsMatchingSemverRange(metadata.depends.minecraft, simplifiedGameVersions)
				: []
			const loaders: string[] = []

			// Detect 1.3-1.13 -> legacy-fabric
			const hasLegacyVersions = detectedGameVersions.some((version) => {
				const match = version.match(/^1\.(\d+)/)
				return match && parseInt(match[1]) >= 3 && parseInt(match[1]) <= 13
			})

			if (hasLegacyVersions) loaders.push('legacy-fabric')
			else loaders.push('fabric')

			return {
				name: `${project.title} ${metadata.version}`,
				version_number: metadata.version,
				loaders,
				version_type: versionType(metadata.version),
				game_versions: detectedGameVersions,
			}
		},
		// Quilt
		'quilt.mod.json': (file: string): InferredVersionInfo => {
			const metadata = JSON.parse(file) as any

			return {
				name: `${project.title} ${metadata.quilt_loader.version}`,
				version_number: metadata.quilt_loader.version,
				loaders: ['quilt'],
				version_type: versionType(metadata.quilt_loader.version),
				game_versions: metadata.quilt_loader.depends
					? getGameVersionsMatchingSemverRange(
							metadata.quilt_loader.depends.find((x: any) => x.id === 'minecraft')
								? metadata.quilt_loader.depends.find((x: any) => x.id === 'minecraft').versions
								: [],
							simplifiedGameVersions,
						)
					: [],
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
