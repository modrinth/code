import type { Labrinth } from '@modrinth/api-client'

import type {
	AnalyticsDashboardFilterOptions,
	AnalyticsFacetsFilterOptionSummary,
	AnalyticsProjectVersionSource,
	AnalyticsSelectedFilters,
	AnalyticsVersionMetadata,
	NormalizedAnalyticsSelectedFilters,
	ProjectVersionFilterOptionSummary,
} from './analytics-types'

export function sortStringValues(values: string[]): string[] {
	return [...values].sort((left, right) => left.localeCompare(right))
}

function toAnalyticsVersionMetadata(
	version: Labrinth.Versions.v3.Version,
): AnalyticsVersionMetadata {
	return {
		id: version.id,
		versionNumber: version.version_number,
		datePublished: version.date_published,
		projectId: version.project_id,
		downloads: version.downloads,
		gameVersions: [...version.game_versions],
		loaders:
			version.mrpack_loaders && version.mrpack_loaders.length > 0
				? [...version.mrpack_loaders]
				: [...version.loaders],
	}
}

export function getProjectVersionFilterOptionSummary(
	versions: AnalyticsVersionMetadata[],
): ProjectVersionFilterOptionSummary {
	const gameVersions = new Set<string>()
	const loaders = new Set<string>()
	const versionIds = new Set<string>()

	for (const version of versions) {
		versionIds.add(version.id)

		for (const gameVersion of version.gameVersions) {
			const normalizedGameVersion = gameVersion.trim()
			if (normalizedGameVersion.length > 0) {
				gameVersions.add(normalizedGameVersion)
			}
		}

		for (const loader of version.loaders) {
			const normalizedLoader = loader.trim().toLowerCase()
			if (normalizedLoader.length > 0 && normalizedLoader !== 'mrpack') {
				loaders.add(normalizedLoader)
			}
		}
	}

	return {
		gameVersions: sortStringValues([...gameVersions]),
		loaderTypes: sortStringValues([...loaders]),
		versionIds: sortStringValues([...versionIds]),
	}
}

export async function fetchAnalyticsVersionMetadataByIds(
	versionIds: string[],
	getVersions: (ids: string[]) => Promise<Labrinth.Versions.v3.Version[]>,
): Promise<AnalyticsVersionMetadata[]> {
	const metadata: AnalyticsVersionMetadata[] = []
	const segmentSize = 800

	for (let index = 0; index < versionIds.length; index += segmentSize) {
		const versions = await getVersions(versionIds.slice(index, index + segmentSize))
		metadata.push(...versions.map(toAnalyticsVersionMetadata))
	}

	return metadata
}

export function getAnalyticsVersionIdsFromProjects(
	projects: readonly AnalyticsProjectVersionSource[],
	projectIds: readonly string[],
): string[] {
	const selectedProjectIds = new Set(projectIds)
	const versionIds = new Set<string>()

	for (const project of projects) {
		if (!selectedProjectIds.has(project.id)) {
			continue
		}

		for (const versionId of project.versions ?? []) {
			const normalizedVersionId = versionId.trim()
			if (normalizedVersionId.length > 0) {
				versionIds.add(normalizedVersionId)
			}
		}
	}

	return sortStringValues([...versionIds])
}

function retainAvailableSelectedFilterValues(
	values: string[],
	availableValues: string[],
): string[] {
	const availableValueSet = new Set(availableValues)
	return values.filter((value) => availableValueSet.has(value))
}

export function sanitizeAnalyticsSelectedFiltersForAvailableOptions(
	filters: AnalyticsSelectedFilters,
	filterOptions: AnalyticsDashboardFilterOptions,
): AnalyticsSelectedFilters {
	return {
		...filters,
		download_reason: retainAvailableSelectedFilterValues(
			filters.download_reason,
			filterOptions.downloadReasons,
		),
		game_version: retainAvailableSelectedFilterValues(
			filters.game_version,
			filterOptions.gameVersions,
		),
		loader_type: retainAvailableSelectedFilterValues(
			filters.loader_type,
			filterOptions.loaderTypes,
		),
	}
}

export function cloneAnalyticsSelectedFilters(
	filters: AnalyticsSelectedFilters,
): AnalyticsSelectedFilters {
	return {
		project: [...filters.project],
		project_status: [...filters.project_status],
		country: [...filters.country],
		monetization: [...filters.monetization],
		user_agent: [...filters.user_agent],
		download_reason: [...filters.download_reason],
		version_id: [...filters.version_id],
		game_version: [...filters.game_version],
		loader_type: [...filters.loader_type],
	}
}

export function cloneAnalyticsFilterOptions(
	filterOptions: AnalyticsDashboardFilterOptions,
): AnalyticsDashboardFilterOptions {
	return {
		countries: [...filterOptions.countries],
		downloadSources: [...filterOptions.downloadSources],
		downloadReasons: [...filterOptions.downloadReasons],
		gameVersions: [...filterOptions.gameVersions],
		loaderTypes: [...filterOptions.loaderTypes],
		versionIds: [...filterOptions.versionIds],
	}
}

function getEmptyAnalyticsFacetsFilterOptionSummary(): AnalyticsFacetsFilterOptionSummary {
	return {
		countries: [],
		downloadSources: [],
		downloadReasons: [],
		gameVersions: [],
		loaderTypes: [],
		versionIds: [],
		projectDownloadsById: new Map(),
		projectVersionDownloadsById: new Map(),
		gameVersionDownloadsByVersion: new Map(),
		countryDownloadsByCode: new Map(),
	}
}

function getAnalyticsFacetValues<T>(
	facets: Labrinth.Analytics.v3.AnalyticsFacet<T>[] | null | undefined,
): T[] {
	return facets?.map((facet) => facet.value) ?? []
}

function getAnalyticsFacetDownloadsByValue<T>(
	facets: Labrinth.Analytics.v3.AnalyticsFacet<T>[] | null | undefined,
	getKey: (value: T) => string,
): Map<string, number> {
	const downloadsByValue = new Map<string, number>()
	for (const facet of facets ?? []) {
		const key = getKey(facet.value)
		if (key.length === 0) {
			continue
		}

		const downloads = getAnalyticsFacetDownloadCount(facet)
		if (downloads === null) {
			continue
		}

		downloadsByValue.set(key, (downloadsByValue.get(key) ?? 0) + downloads)
	}

	return downloadsByValue
}

function getAnalyticsFacetDownloadCount<T>(
	facet: Labrinth.Analytics.v3.AnalyticsFacet<T>,
): number | null {
	const count = (facet as { count?: number }).count ?? facet.downloads
	return Number.isFinite(count) ? count : null
}

function doesAnalyticsFacetHaveDownloadCount<T>(
	facet: Labrinth.Analytics.v3.AnalyticsFacet<T>,
): boolean {
	return !!getAnalyticsFacetDownloadCount(facet)
}

export function shouldFetchAnalyticsDownloadCountFallback(
	facets: Labrinth.Analytics.v3.AnalyticsFacets | null | undefined,
): boolean {
	if (!facets) {
		return false
	}

	const projectDownloadFacets = Object.values(facets.project_downloads)
	const downloadFacets =
		projectDownloadFacets.flat() as Labrinth.Analytics.v3.AnalyticsFacet<unknown>[]
	return downloadFacets.some((facet) => !doesAnalyticsFacetHaveDownloadCount(facet))
}

export function getAnalyticsFacetsFilterOptionSummary(
	facets: Labrinth.Analytics.v3.AnalyticsFacets | null | undefined,
): AnalyticsFacetsFilterOptionSummary {
	if (!facets) {
		return getEmptyAnalyticsFacetsFilterOptionSummary()
	}

	const downloadCountries = getAnalyticsFacetValues(facets.project_downloads.country)
	const downloadGameVersions = getAnalyticsFacetValues(facets.project_downloads.game_version)
	const downloadLoaders = getAnalyticsFacetValues(facets.project_downloads.loader)
	const downloadVersionIds = getAnalyticsFacetValues(facets.project_downloads.version_id)
	const viewCountries = getAnalyticsFacetValues(facets.project_views.country)
	const playtimeCountries = getAnalyticsFacetValues(facets.project_playtime.country)
	const playtimeGameVersions = getAnalyticsFacetValues(facets.project_playtime.game_version)
	const playtimeLoaders = getAnalyticsFacetValues(facets.project_playtime.loader)
	const playtimeVersionIds = getAnalyticsFacetValues(facets.project_playtime.version_id)
	const countries = new Set([...viewCountries, ...downloadCountries, ...playtimeCountries])
	const gameVersions = new Set([...downloadGameVersions, ...playtimeGameVersions])
	const loaderTypes = new Set<string>()
	for (const loader of [...downloadLoaders, ...playtimeLoaders]) {
		const normalizedLoader = loader.trim().toLowerCase()
		if (normalizedLoader.length > 0 && normalizedLoader !== 'mrpack') {
			loaderTypes.add(normalizedLoader)
		}
	}

	return {
		countries: sortStringValues(
			[...countries]
				.map((country) => country.trim().toUpperCase())
				.filter((country) => country.length > 0),
		),
		downloadSources: sortStringValues(getAnalyticsFacetValues(facets.project_downloads.user_agent)),
		downloadReasons: sortStringValues(getAnalyticsFacetValues(facets.project_downloads.reason)),
		gameVersions: sortStringValues(
			[...gameVersions]
				.map((gameVersion) => gameVersion.trim())
				.filter((gameVersion) => gameVersion.length > 0),
		),
		loaderTypes: sortStringValues([...loaderTypes]),
		versionIds: sortStringValues([...new Set([...downloadVersionIds, ...playtimeVersionIds])]),
		projectDownloadsById: getAnalyticsFacetDownloadsByValue(
			facets.project_downloads.project_id,
			(projectId) => projectId.trim(),
		),
		projectVersionDownloadsById: getAnalyticsFacetDownloadsByValue(
			facets.project_downloads.version_id,
			(versionId) => versionId.trim(),
		),
		gameVersionDownloadsByVersion: getAnalyticsFacetDownloadsByValue(
			facets.project_downloads.game_version,
			(gameVersion) => gameVersion.trim(),
		),
		countryDownloadsByCode: getAnalyticsFacetDownloadsByValue(
			facets.project_downloads.country,
			(country) => country.trim().toUpperCase(),
		),
	}
}

export function doesAnalyticsPointMatchFilters(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
	filters: AnalyticsSelectedFilters,
): boolean {
	return doesAnalyticsPointMatchNormalizedFilters(
		dataPoint,
		normalizeAnalyticsSelectedFilters(filters),
	)
}

export function normalizeAnalyticsSelectedFilters(
	filters: AnalyticsSelectedFilters,
): NormalizedAnalyticsSelectedFilters {
	return {
		country: normalizeAnalyticsFilterValues(filters.country),
		monetization: normalizeAnalyticsFilterValues(filters.monetization),
		userAgent: normalizeAnalyticsFilterValues(filters.user_agent),
		downloadReason: normalizeAnalyticsFilterValues(filters.download_reason),
		versionId: normalizeAnalyticsFilterValues(filters.version_id),
		gameVersion: normalizeAnalyticsFilterValues(filters.game_version),
		loaderType: normalizeAnalyticsFilterValues(filters.loader_type),
	}
}

function normalizeAnalyticsFilterValues(values: string[]): ReadonlySet<string> {
	const normalizedValues = new Set<string>()
	for (const value of values) {
		const normalizedValue = value.trim().toLowerCase()
		if (normalizedValue.length > 0) {
			normalizedValues.add(normalizedValue)
		}
	}
	return normalizedValues
}

export function doesAnalyticsPointMatchNormalizedFilters(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
	filters: NormalizedAnalyticsSelectedFilters,
): boolean {
	switch (dataPoint.metric_kind) {
		case 'views':
			return (
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.country,
					getCountryFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.monetization,
					getMonetizationFilterValue,
				)
			)
		case 'downloads':
			return (
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.country,
					getCountryFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.monetization,
					getMonetizationFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.userAgent,
					getDownloadSourceFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.downloadReason,
					getDownloadReasonFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.versionId,
					getVersionFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.gameVersion,
					getGameVersionFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(dataPoint, filters.loaderType, getLoaderFilterValue)
			)
		case 'playtime':
			return (
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.country,
					getCountryFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.versionId,
					getVersionFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(
					dataPoint,
					filters.gameVersion,
					getGameVersionFilterValue,
				) &&
				doesAnalyticsPointMatchNormalizedFilter(dataPoint, filters.loaderType, getLoaderFilterValue)
			)
		case 'revenue':
			return true
		default:
			return true
	}
}

function doesAnalyticsPointMatchNormalizedFilter(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
	filterValues: ReadonlySet<string>,
	getPointValue: (dataPoint: Labrinth.Analytics.v3.ProjectAnalytics) => string | null | undefined,
): boolean {
	if (filterValues.size === 0) {
		return true
	}

	const pointValue = getPointValue(dataPoint)
	if (pointValue === undefined) {
		return true
	}
	if (pointValue === null) {
		return false
	}

	const normalizedPointValue = pointValue.trim().toLowerCase()
	return filterValues.has(normalizedPointValue)
}

function getCountryFilterValue(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
): string | null | undefined {
	if (
		dataPoint.metric_kind !== 'views' &&
		dataPoint.metric_kind !== 'downloads' &&
		dataPoint.metric_kind !== 'playtime'
	) {
		return undefined
	}

	return dataPoint.country ?? null
}

function getMonetizationFilterValue(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
): string | null | undefined {
	if (dataPoint.metric_kind !== 'views' && dataPoint.metric_kind !== 'downloads') {
		return undefined
	}
	if (typeof dataPoint.monetized !== 'boolean') {
		return null
	}

	return dataPoint.monetized ? 'monetized' : 'unmonetized'
}

function getDownloadSourceFilterValue(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
): string | null | undefined {
	if (dataPoint.metric_kind !== 'downloads') {
		return undefined
	}

	return dataPoint.user_agent ?? null
}

function getDownloadReasonFilterValue(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
): string | null | undefined {
	if (dataPoint.metric_kind !== 'downloads') {
		return undefined
	}

	return dataPoint.reason ?? null
}

function getVersionFilterValue(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
): string | null | undefined {
	if (dataPoint.metric_kind !== 'downloads' && dataPoint.metric_kind !== 'playtime') {
		return undefined
	}

	return dataPoint.version_id ?? null
}

function getGameVersionFilterValue(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
): string | null | undefined {
	if (dataPoint.metric_kind !== 'downloads' && dataPoint.metric_kind !== 'playtime') {
		return undefined
	}

	return dataPoint.game_version ?? null
}

function getLoaderFilterValue(
	dataPoint: Labrinth.Analytics.v3.ProjectAnalytics,
): string | null | undefined {
	if (dataPoint.metric_kind !== 'downloads' && dataPoint.metric_kind !== 'playtime') {
		return undefined
	}

	return dataPoint.loader ?? null
}
