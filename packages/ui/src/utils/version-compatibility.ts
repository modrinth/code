const NON_MOD_PROJECT_TYPES = new Set(['shader', 'shaderpack', 'resourcepack', 'datapack'])

const LOADER_ALIAS_GROUPS = [
	['paper', 'purpur', 'spigot', 'bukkit'],
	['neoforge', 'neo'],
]

type VersionCompatibilityData = {
	game_versions: string[]
	loaders: string[]
}

export function normalizeLoaderAlias(loader: string) {
	return loader.toLowerCase().replaceAll('_', '').replaceAll('-', '').replaceAll(' ', '')
}

export function getCompatibleLoaderAliases(loader: string) {
	const normalizedLoader = normalizeLoaderAlias(loader)
	const aliases = new Set([normalizedLoader])
	const aliasGroup = LOADER_ALIAS_GROUPS.find((group) => group.includes(normalizedLoader))

	if (aliasGroup) {
		for (const alias of aliasGroup) {
			aliases.add(alias)
		}
	}

	return aliases
}

export function versionChangesGameVersion(
	version: VersionCompatibilityData,
	currentGameVersion: string,
) {
	return !!currentGameVersion && !version.game_versions.includes(currentGameVersion)
}

export function versionMatchesCompatibilityTarget(
	version: VersionCompatibilityData,
	target: {
		gameVersion: string
		loader: string
		projectType?: string
	},
) {
	if (!target.gameVersion || !version.game_versions.includes(target.gameVersion)) {
		return false
	}

	const normalizedVersionLoaders = version.loaders.map(normalizeLoaderAlias)

	if (target.projectType === 'datapack') {
		return normalizedVersionLoaders.includes('datapack')
	}

	if (target.projectType && NON_MOD_PROJECT_TYPES.has(target.projectType)) {
		return true
	}

	if (
		target.projectType === 'modpack' &&
		(normalizedVersionLoaders.length === 0 ||
			normalizedVersionLoaders.every((loader) => loader === 'mrpack'))
	) {
		return true
	}

	if (!target.loader) {
		return false
	}

	const loaderAliases = getCompatibleLoaderAliases(target.loader)
	return normalizedVersionLoaders.some((loader) => loaderAliases.has(loader))
}
