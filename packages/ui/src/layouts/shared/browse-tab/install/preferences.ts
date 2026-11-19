import type { Labrinth } from '@modrinth/api-client'

import {
	newestVersionForChannel,
	type UpdateChannelPolicy,
} from '#ui/layouts/shared/content-tab/utils/update-channels'
import type { FilterValue } from '#ui/utils/search'

export type BrowseInstallContentType =
	| 'modpack'
	| 'mod'
	| 'plugin'
	| 'datapack'
	| 'resourcepack'
	| 'shader'
export type BrowseInstallAddonContentType = Exclude<BrowseInstallContentType, 'modpack'>

export type BrowseInstallPlanSource = 'filtered' | 'target'

export interface BrowseInstallPreferences {
	gameVersions?: string[]
	loaders?: string[]
}

export interface BrowseInstallTarget {
	gameVersion?: string | null
	loader?: string | null
}

export interface SelectedInstallPreferencesOptions {
	contentType: string
	selectedFilters?: readonly FilterValue[]
	providedFilters?: readonly FilterValue[]
	overriddenProvidedFilterTypes?: readonly string[]
}

interface InstallCandidate {
	preferences: BrowseInstallPreferences
	source: BrowseInstallPlanSource
}

export function getLoaderFilterTypes(contentType: string) {
	if (contentType === 'mod') return ['mod_loader']
	if (contentType === 'plugin') return ['plugin_loader', 'plugin_platform']
	if (contentType === 'modpack') return ['modpack_loader']
	if (contentType === 'shader') return ['shader_loader']
	if (contentType === 'datapack') return ['datapack_loader']
	return []
}

const SERVER_RUNTIME_INSTALL_FILTER_TYPES = new Set([
	'game_version',
	'mod_loader',
	'plugin_loader',
	'plugin_platform',
	'datapack_loader',
])

export function stripServerRuntimeInstallFilters(filters: readonly FilterValue[]) {
	return filters.filter((filter) => !SERVER_RUNTIME_INSTALL_FILTER_TYPES.has(filter.type))
}

export function stripServerRuntimeInstallOverrides(filterTypes: readonly string[]) {
	return filterTypes.filter((type) => !SERVER_RUNTIME_INSTALL_FILTER_TYPES.has(type))
}

export function getEffectiveInstallFilters({
	selectedFilters = [],
	providedFilters = [],
	overriddenProvidedFilterTypes = [],
}: Omit<SelectedInstallPreferencesOptions, 'contentType'>) {
	const effectiveProvidedFilters = providedFilters.filter(
		(providedFilter) => !overriddenProvidedFilterTypes.includes(providedFilter.type),
	)
	const userFilters = selectedFilters.filter(
		(userFilter) =>
			!effectiveProvidedFilters.some((providedFilter) => providedFilter.type === userFilter.type),
	)

	return [...userFilters, ...effectiveProvidedFilters].filter((filter) => !filter.negative)
}

export function getInstallPreferencesFromFilters(
	contentType: string,
	filters: readonly FilterValue[],
): BrowseInstallPreferences {
	const loaderFilterTypes = getLoaderFilterTypes(contentType)
	const gameVersions = uniqueDefined(
		filters.filter((filter) => filter.type === 'game_version').map((filter) => filter.option),
	)
	const loaders = uniqueDefined(
		filters
			.filter((filter) => loaderFilterTypes.includes(filter.type))
			.map((filter) => filter.option),
	)

	return normalizeInstallPreferences({
		gameVersions: gameVersions.length > 0 ? gameVersions : undefined,
		loaders: loaders.length > 0 ? loaders : undefined,
	})
}

export function getSelectedInstallPreferences(
	options: SelectedInstallPreferencesOptions,
): BrowseInstallPreferences {
	return getInstallPreferencesFromFilters(options.contentType, getEffectiveInstallFilters(options))
}

export function getProjectVersionFilterQuery(options: SelectedInstallPreferencesOptions) {
	const preferences = getSelectedInstallPreferences(options)
	return {
		g: preferences.gameVersions ?? [],
		l: preferences.loaders ?? [],
		e: [],
		page: undefined,
	}
}

export function getTargetInstallPreferences(
	target: BrowseInstallTarget,
	contentType?: string,
): BrowseInstallPreferences {
	const gameVersion = target.gameVersion?.trim()
	const loader = target.loader?.trim()
	const shouldUseTargetRuntime = contentType !== 'modpack'

	return normalizeInstallPreferences({
		gameVersions: gameVersion && shouldUseTargetRuntime ? [gameVersion] : undefined,
		loaders:
			contentType === 'datapack'
				? ['datapack']
				: loader &&
					  shouldUseTargetRuntime &&
					  contentType !== 'resourcepack' &&
					  contentType !== 'shader'
					? [loader]
					: undefined,
	})
}

export function normalizeLoaderAlias(loader: string) {
	return loader.toLowerCase().replaceAll('_', '').replaceAll('-', '').replaceAll(' ', '')
}

export function getCompatibleLoaderAliases(loader: string) {
	const normalized = normalizeLoaderAlias(loader)
	if (!normalized) return new Set<string>()
	if (['paper', 'purpur', 'spigot', 'bukkit'].includes(normalized)) {
		return new Set(['paper', 'purpur', 'spigot', 'bukkit'])
	}
	if (normalized === 'neoforge' || normalized === 'neo') {
		return new Set(['neoforge', 'neo'])
	}
	return new Set([normalized])
}

export function preferencesDiffer(
	selected: BrowseInstallPreferences,
	target: BrowseInstallPreferences,
) {
	return (
		preferencesConflict(selected.gameVersions, target.gameVersions) ||
		loaderPreferencesConflict(selected.loaders, target.loaders)
	)
}

export function mergeInstallPreferences(
	selected: BrowseInstallPreferences,
	target: BrowseInstallPreferences,
): BrowseInstallPreferences {
	return normalizeInstallPreferences({
		gameVersions: selected.gameVersions?.length ? selected.gameVersions : target.gameVersions,
		loaders: selected.loaders?.length ? selected.loaders : target.loaders,
	})
}

export function getLatestMatchingInstallVersion(
	versions: readonly Labrinth.Versions.v2.Version[],
	preferences: BrowseInstallPreferences,
	updateChannel: UpdateChannelPolicy = 'alpha',
) {
	return newestVersionForChannel(
		versions.filter((version) => versionMatchesPreferences(version, preferences)),
		updateChannel,
	)
}

export function getInstallCandidates(
	selectedPreferences: BrowseInstallPreferences,
	targetPreferences: BrowseInstallPreferences,
): InstallCandidate[] {
	const filteredPreferences = mergeInstallPreferences(selectedPreferences, targetPreferences)
	const candidates: InstallCandidate[] = []

	if (hasPreferences(filteredPreferences)) {
		candidates.push({
			preferences: filteredPreferences,
			source: preferencesEquivalent(selectedPreferences, targetPreferences) ? 'target' : 'filtered',
		})
	} else {
		candidates.push({ preferences: {}, source: 'filtered' })
	}

	if (
		hasPreferences(targetPreferences) &&
		preferencesDiffer(filteredPreferences, targetPreferences)
	) {
		candidates.push({ preferences: targetPreferences, source: 'target' })
	}

	return candidates
}

function hasPreferences(preferences: BrowseInstallPreferences) {
	return !!preferences.gameVersions?.length || !!preferences.loaders?.length
}

function versionMatchesPreferences(
	version: Labrinth.Versions.v2.Version,
	preferences: BrowseInstallPreferences,
) {
	const gameVersionMatches =
		!preferences.gameVersions?.length ||
		version.game_versions.some((gameVersion) => preferences.gameVersions?.includes(gameVersion))
	if (!gameVersionMatches) return false
	if (!preferences.loaders?.length) return true

	const compatibleLoaders = getCompatibleLoaderAliasSet(preferences.loaders)
	return version.loaders.some((loader) => compatibleLoaders.has(normalizeLoaderAlias(loader)))
}

function preferencesConflict(
	selected: readonly string[] | undefined,
	target: readonly string[] | undefined,
) {
	if (!selected?.length || !target?.length) return false
	return !selected.some((value) => target.includes(value))
}

function loaderPreferencesConflict(
	selected: readonly string[] | undefined,
	target: readonly string[] | undefined,
) {
	if (!selected?.length || !target?.length) return false
	const selectedLoaders = getCompatibleLoaderAliasSet(selected)
	const targetLoaders = getCompatibleLoaderAliasSet(target)
	return !Array.from(selectedLoaders).some((loader) => targetLoaders.has(loader))
}

function preferencesEquivalent(
	selected: BrowseInstallPreferences,
	target: BrowseInstallPreferences,
) {
	return (
		valueSetsEquivalent(selected.gameVersions, target.gameVersions) &&
		loaderSetsEquivalent(selected.loaders, target.loaders)
	)
}

function valueSetsEquivalent(
	selected: readonly string[] | undefined,
	target: readonly string[] | undefined,
) {
	return setsEquivalent(new Set(selected ?? []), new Set(target ?? []))
}

function loaderSetsEquivalent(
	selected: readonly string[] | undefined,
	target: readonly string[] | undefined,
) {
	return setsEquivalent(
		getCompatibleLoaderAliasSet(selected ?? []),
		getCompatibleLoaderAliasSet(target ?? []),
	)
}

function getCompatibleLoaderAliasSet(loaders: readonly string[]) {
	const aliases = new Set<string>()
	for (const loader of loaders) {
		for (const alias of getCompatibleLoaderAliases(loader)) {
			aliases.add(alias)
		}
	}
	return aliases
}

function setsEquivalent(a: Set<string>, b: Set<string>) {
	if (a.size !== b.size) return false
	return Array.from(a).every((value) => b.has(value))
}

export function normalizeInstallPreferences(
	preferences?: BrowseInstallPreferences,
): BrowseInstallPreferences {
	return {
		gameVersions: uniqueDefined(preferences?.gameVersions),
		loaders: uniqueDefined(preferences?.loaders),
	}
}

function uniqueDefined(values: readonly (string | null | undefined)[] = []) {
	return Array.from(
		new Set(values.map((value) => value?.trim()).filter((value): value is string => !!value)),
	)
}
