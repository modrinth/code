import type { Labrinth } from '@modrinth/api-client'

import type { FilterValue } from '#ui/utils/search'

export type BrowseInstallContentType = 'modpack' | 'mod' | 'plugin' | 'datapack'
export type BrowseInstallAddonContentType = Exclude<BrowseInstallContentType, 'modpack'>

/**
 * Indicates why a concrete version was selected.
 *
 * `filtered` means the current browse filters resolved the version.
 * `target` means filter resolution failed or matched the target exactly, so the server/instance target won.
 */
export type BrowseInstallPlanSource = 'filtered' | 'target'

/**
 * Version constraints used during install resolution.
 *
 * Empty arrays and blank values are normalized away, so missing properties mean "do not constrain".
 */
export interface BrowseInstallPreferences {
	gameVersions?: string[]
	loaders?: string[]
}

/**
 * Server or instance metadata that should be used as the fallback compatibility target.
 */
export interface BrowseInstallTarget {
	gameVersion?: string | null
	loader?: string | null
}

/**
 * Minimal project shape needed by shared install resolution.
 */
export interface BrowseInstallProject {
	project_id: string
	latest_version?: string | null
	version_id?: string | null
	title?: string
	name?: string
	icon_url?: string | null
}

/**
 * Fully resolved install work item.
 *
 * This is intentionally concrete so queued installs can be flushed later without re-resolving
 * against filters that may have changed since the user clicked install.
 */
export interface BrowseInstallPlan<TProject extends BrowseInstallProject = BrowseInstallProject> {
	project: TProject
	projectId: string
	versionId: string
	versionName?: string
	versionNumber?: string
	fileName?: string
	contentType: BrowseInstallContentType
	preferences: BrowseInstallPreferences
	source: BrowseInstallPlanSource
}

/**
 * Small adapter around caller-owned queue state.
 *
 * Callers keep their own reactive storage; shared logic only replaces the whole map.
 */
export interface BrowseInstallQueue<TProject extends BrowseInstallProject = BrowseInstallProject> {
	get: () => Map<string, BrowseInstallPlan<TProject>>
	set: (plans: Map<string, BrowseInstallPlan<TProject>>) => void
}

/**
 * Filter inputs for deriving selected install preferences.
 *
 * Provided filters come from a target context, and overridden filter types are ignored so user
 * choices can replace the target-provided constraints.
 */
export interface SelectedInstallPreferencesOptions {
	contentType: string
	selectedFilters?: readonly FilterValue[]
	providedFilters?: readonly FilterValue[]
	overriddenProvidedFilterTypes?: readonly string[]
}

/**
 * Inputs for resolving one concrete install plan.
 *
 * Version fetching is injected so this module stays platform-agnostic and can be used by both web
 * and app frontends.
 */
export interface ResolveInstallPlanOptions<
	TProject extends BrowseInstallProject,
> extends SelectedInstallPreferencesOptions {
	project: TProject
	contentType: BrowseInstallContentType
	targetPreferences?: BrowseInstallPreferences
	getProjectVersions: (projectId: string) => Promise<Labrinth.Versions.v2.Version[]>
}

/**
 * Install request wrapper around plan resolution.
 *
 * Queue mode stores the resolved plan; immediate mode passes it to the caller's install handler.
 */
export interface RequestInstallOptions<
	TProject extends BrowseInstallProject,
> extends ResolveInstallPlanOptions<TProject> {
	mode: 'queue' | 'immediate'
	queue?: BrowseInstallQueue<TProject>
	install?: (plan: BrowseInstallPlan<TProject>) => void | Promise<void>
}

/**
 * Inputs for committing queued plans without re-running version matching.
 */
export interface FlushInstallQueueOptions<TProject extends BrowseInstallProject> {
	queue: BrowseInstallQueue<TProject>
	install: (plan: BrowseInstallPlan<TProject>) => void | Promise<void>
	onError?: (error: unknown, plan: BrowseInstallPlan<TProject>) => void
	onProgress?: (
		completed: number,
		total: number,
		plan: BrowseInstallPlan<TProject>,
	) => void | Promise<void>
}

/**
 * Result of a queue flush. Failed plans are also written back to the queue.
 */
export interface FlushInstallQueueResult<TProject extends BrowseInstallProject> {
	ok: boolean
	successfulPlans: BrowseInstallPlan<TProject>[]
	failedPlans: Map<string, BrowseInstallPlan<TProject>>
}

interface InstallCandidate {
	preferences: BrowseInstallPreferences
	source: BrowseInstallPlanSource
}

/**
 * Maps a project/content type to the browse filter keys that represent its loader.
 */
export function getLoaderFilterTypes(contentType: string) {
	if (contentType === 'mod') return ['mod_loader']
	if (contentType === 'plugin') return ['plugin_loader', 'plugin_platform']
	if (contentType === 'modpack') return ['modpack_loader']
	if (contentType === 'shader') return ['shader_loader']
	return []
}

/**
 * Merges user-selected filters with target-provided filters for install decisions.
 *
 * User filters win per filter type, provided filters are dropped when overridden, and negative
 * filters are excluded because they are browse-only constraints.
 */
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

/**
 * Converts effective browse filters into install preferences for a specific content type.
 */
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

/**
 * Derives the preferences represented by the current browse selection plus active provided filters.
 */
export function getSelectedInstallPreferences(
	options: SelectedInstallPreferencesOptions,
): BrowseInstallPreferences {
	return getInstallPreferencesFromFilters(options.contentType, getEffectiveInstallFilters(options))
}

/**
 * Converts server/instance metadata into fallback install preferences.
 */
export function getTargetInstallPreferences(
	target: BrowseInstallTarget,
	contentType?: string,
): BrowseInstallPreferences {
	const gameVersion = target.gameVersion?.trim()
	const loader = target.loader?.trim()
	const shouldUseTargetRuntime = contentType !== 'modpack'

	return normalizeInstallPreferences({
		gameVersions: gameVersion && shouldUseTargetRuntime ? [gameVersion] : undefined,
		loaders: loader && shouldUseTargetRuntime ? [loader] : undefined,
	})
}

/**
 * Normalizes loader identifiers so API and UI aliases compare consistently.
 */
export function normalizeLoaderAlias(loader: string) {
	return loader.toLowerCase().replaceAll('_', '').replaceAll('-', '').replaceAll(' ', '')
}

/**
 * Returns aliases that should be considered mutually compatible for install matching.
 */
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

/**
 * Checks whether selected filters conflict with the target constraints.
 */
export function preferencesDiffer(
	selected: BrowseInstallPreferences,
	target: BrowseInstallPreferences,
) {
	return (
		preferencesConflict(selected.gameVersions, target.gameVersions) ||
		loaderPreferencesConflict(selected.loaders, target.loaders)
	)
}

/**
 * Fills missing selected preferences from the target.
 *
 * This preserves the user's explicit filter choices while still constraining unconstrained axes to
 * the server/instance target.
 */
export function mergeInstallPreferences(
	selected: BrowseInstallPreferences,
	target: BrowseInstallPreferences,
): BrowseInstallPreferences {
	return normalizeInstallPreferences({
		gameVersions: selected.gameVersions?.length ? selected.gameVersions : target.gameVersions,
		loaders: selected.loaders?.length ? selected.loaders : target.loaders,
	})
}

/**
 * Finds the newest version matching the given preferences.
 */
export function getLatestMatchingInstallVersion(
	versions: readonly Labrinth.Versions.v2.Version[],
	preferences: BrowseInstallPreferences,
	contentType: string,
) {
	return [...versions]
		.filter((version) => versionMatchesPreferences(version, preferences, contentType))
		.sort((a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime())[0]
}

/**
 * Resolves the concrete version to install.
 *
 * The resolver tries the filtered plan first, with target values filling any missing axes. If that
 * cannot resolve and differs from the target, it falls back to the target-only plan.
 */
export async function resolveInstallPlan<TProject extends BrowseInstallProject>(
	options: ResolveInstallPlanOptions<TProject>,
): Promise<BrowseInstallPlan<TProject>> {
	const projectId = options.project.project_id
	if (!projectId) {
		throw new Error('No project is available for install.')
	}

	const selectedPreferences = getSelectedInstallPreferences(options)
	const targetPreferences = normalizeInstallPreferences(options.targetPreferences)
	const candidates = getInstallCandidates(selectedPreferences, targetPreferences)
	const versions = await options.getProjectVersions(projectId)
	let lastError: Error | null = null

	for (const candidate of candidates) {
		const version = getLatestMatchingInstallVersion(
			versions,
			candidate.preferences,
			options.contentType,
		)

		if (version) {
			const fileName =
				version.files.find((file) => file.primary)?.filename ?? version.files[0]?.filename
			return {
				project: options.project,
				projectId,
				versionId: version.id,
				versionName: version.name,
				versionNumber: version.version_number,
				fileName,
				contentType: options.contentType,
				preferences: candidate.preferences,
				source: candidate.source,
			}
		}

		lastError = createNoCompatibleVersionError(options.contentType, candidate.preferences)
	}

	throw lastError ?? new Error('No version found for this project.')
}

/**
 * Resolves and either queues or immediately installs a project.
 *
 * Queue replacement is keyed by project ID, so clicking install again after changing filters
 * replaces the previously resolved plan.
 */
export async function requestInstall<TProject extends BrowseInstallProject>(
	options: RequestInstallOptions<TProject>,
) {
	const plan = await resolveInstallPlan(options)

	if (options.mode === 'queue') {
		if (!options.queue) {
			throw new Error('No install queue is available.')
		}

		const nextPlans = new Map(options.queue.get())
		nextPlans.set(plan.projectId, plan)
		options.queue.set(nextPlans)
		return plan
	}

	await options.install?.(plan)
	return plan
}

/**
 * Commits queued install plans exactly as stored.
 *
 * Successful plans are removed; failed plans remain in the queue for retry or user action.
 */
export async function flushInstallQueue<TProject extends BrowseInstallProject>({
	queue,
	install,
	onError,
	onProgress,
}: FlushInstallQueueOptions<TProject>): Promise<FlushInstallQueueResult<TProject>> {
	const queuedPlans = Array.from(queue.get().values())
	const failedPlans = new Map<string, BrowseInstallPlan<TProject>>()
	const successfulPlans: BrowseInstallPlan<TProject>[] = []
	let completed = 0

	for (const plan of queuedPlans) {
		try {
			await install(plan)
			successfulPlans.push(plan)
		} catch (error) {
			failedPlans.set(plan.projectId, plan)
			onError?.(error, plan)
		} finally {
			completed++
			await onProgress?.(completed, queuedPlans.length, plan)
		}
	}

	queue.set(failedPlans)

	return {
		ok: failedPlans.size === 0,
		successfulPlans,
		failedPlans,
	}
}

/**
 * Builds the ordered resolution attempts for an install request.
 */
function getInstallCandidates(
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
	contentType: string,
) {
	const gameVersionMatches =
		!preferences.gameVersions?.length ||
		version.game_versions.some((gameVersion) => preferences.gameVersions?.includes(gameVersion))
	if (!gameVersionMatches) return false
	if (contentType === 'datapack') return true
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

function normalizeInstallPreferences(
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

function createNoCompatibleVersionError(
	contentType: BrowseInstallContentType,
	preferences: BrowseInstallPreferences,
) {
	const versionLabel = preferences.gameVersions?.length
		? preferences.gameVersions.join(', ')
		: 'any game version'
	const loaderLabel = preferences.loaders?.length ? preferences.loaders.join(', ') : 'any loader'

	return new Error(
		contentType === 'datapack'
			? `No compatible version found for ${versionLabel}.`
			: `No compatible version found for ${versionLabel} / ${loaderLabel}.`,
	)
}
