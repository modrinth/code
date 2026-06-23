import type {
	AnalyticsBreakdownPreset,
	AnalyticsDashboardStat,
	AnalyticsQueryFilterCategory,
	AnalyticsSelectedBreakdowns,
	AnalyticsSelectedFilters,
} from '~/providers/analytics/analytics'

export type AnalyticsDashboardDimension =
	| 'project'
	| 'project_status'
	| 'version_id'
	| 'country'
	| 'monetization'
	| 'user_agent'
	| 'download_reason'
	| 'user_id'
	| 'dependent_project_download'
	| 'dependent_project_id'
	| 'dependent_project_type'
	| 'game_version'
	| 'loader_type'

export const ALL_FILTER_VALUE = '__all__'
export const FILTER_VALUE_CATEGORIES: Exclude<AnalyticsQueryFilterCategory, 'project'>[] = [
	'project_status',
	'country',
	'monetization',
	'user_agent',
	'download_reason',
	'user_id',
	'version_id',
	'game_version',
	'loader_type',
	'dependent_project_id',
	'dependent_project_type',
]

const ANALYTICS_DASHBOARD_STAT_ORDER: AnalyticsDashboardStat[] = [
	'views',
	'downloads',
	'revenue',
	'playtime',
]

const ANALYTICS_STATS_BY_DIMENSION: Record<
	AnalyticsDashboardDimension,
	readonly AnalyticsDashboardStat[]
> = {
	project: ANALYTICS_DASHBOARD_STAT_ORDER,
	version_id: ['downloads', 'playtime'],
	country: ['views', 'downloads', 'playtime'],
	monetization: ['views', 'downloads'],
	user_agent: ['downloads'],
	download_reason: ['downloads'],
	user_id: ['revenue'],
	dependent_project_download: ['downloads'],
	dependent_project_type: ['downloads'],
	game_version: ['downloads', 'playtime'],
	loader_type: ['downloads', 'playtime'],
	dependent_project_id: ['downloads'],
	project_status: ANALYTICS_DASHBOARD_STAT_ORDER,
}

const ANALYTICS_DIMENSION_BY_BREAKDOWN: Record<
	AnalyticsBreakdownPreset,
	AnalyticsDashboardDimension
> = {
	none: 'project',
	project: 'project',
	country: 'country',
	monetization: 'monetization',
	user_agent: 'user_agent',
	download_reason: 'download_reason',
	user_id: 'user_id',
	dependent_project_download: 'dependent_project_download',
	version_id: 'version_id',
	loader: 'loader_type',
	game_version: 'game_version',
}

const ANALYTICS_DIMENSION_BY_FILTER_CATEGORY: Record<
	Exclude<AnalyticsQueryFilterCategory, 'project'>,
	AnalyticsDashboardDimension
> = {
	project_status: 'project_status',
	country: 'country',
	monetization: 'monetization',
	user_agent: 'user_agent',
	download_reason: 'download_reason',
	user_id: 'user_id',
	version_id: 'version_id',
	game_version: 'game_version',
	loader_type: 'loader_type',
	dependent_project_id: 'dependent_project_id',
	dependent_project_type: 'dependent_project_type',
}

const ANALYTICS_FILTER_CATEGORY_BY_BREAKDOWN: Record<
	AnalyticsBreakdownPreset,
	Exclude<AnalyticsQueryFilterCategory, 'project'> | null
> = {
	none: null,
	project: null,
	country: 'country',
	monetization: 'monetization',
	user_agent: 'user_agent',
	download_reason: 'download_reason',
	user_id: 'user_id',
	dependent_project_download: null,
	version_id: 'version_id',
	loader: 'loader_type',
	game_version: 'game_version',
}

export type FilterOption = {
	value: string
	label: string
	searchTerms?: string[]
}

export type ProjectVersionFilterOption = FilterOption

export type ProjectVersionFilterOptionProjectMetadata = {
	name: string
	iconUrl?: string
}

type AnalyticsBreakdownInput = AnalyticsBreakdownPreset | readonly AnalyticsBreakdownPreset[]

function getOptionalDateTimestamp(date: string | undefined): number | undefined {
	if (!date) {
		return undefined
	}

	const timestamp = new Date(date).getTime()
	return Number.isFinite(timestamp) ? timestamp : undefined
}

export function getProjectVersionFilterOptionsCacheKey(
	versionIds: string[],
	versionNumbersById: Map<string, string>,
	versionPublishedDatesById: Map<string, string>,
	versionProjectNamesById: Map<string, string>,
): string {
	return versionIds
		.map(
			(versionId) =>
				`${versionId}\x1f${versionNumbersById.get(versionId) ?? ''}\x1f${
					versionPublishedDatesById.get(versionId) ?? ''
				}\x1f${versionProjectNamesById.get(versionId) ?? ''}`,
		)
		.join('\x1e')
}

export function getProjectVersionFilterOptionProjectMetadataCacheKey(
	versionIds: string[],
	versionProjectNamesById: Map<string, string>,
	versionProjectIconUrlsById: Map<string, string>,
): string {
	return versionIds
		.map(
			(versionId) =>
				`${versionId}\x1f${versionProjectNamesById.get(versionId) ?? ''}\x1f${
					versionProjectIconUrlsById.get(versionId) ?? ''
				}`,
		)
		.join('\x1e')
}

export function getProjectVersionFilterOptionMetadataIds(
	versionIds: string[],
	selectedVersionIds: string[],
): string[] {
	const knownVersionIds = new Set(versionIds)
	const metadataIds = [...versionIds]

	for (const versionId of selectedVersionIds) {
		if (!knownVersionIds.has(versionId)) {
			metadataIds.push(versionId)
			knownVersionIds.add(versionId)
		}
	}

	return metadataIds
}

export function buildProjectVersionFilterOptions(
	versionIds: string[],
	versionNumbersById: Map<string, string>,
	versionPublishedDatesById: Map<string, string>,
	versionProjectNamesById: Map<string, string>,
): ProjectVersionFilterOption[] {
	return versionIds
		.map((versionId) => {
			const projectName = versionProjectNamesById.get(versionId)
			return {
				option: {
					value: versionId,
					label: versionNumbersById.get(versionId) ?? versionId,
					searchTerms: projectName ? [versionId, projectName] : [versionId],
				},
				publishedTimestamp: getOptionalDateTimestamp(versionPublishedDatesById.get(versionId)),
			}
		})
		.sort((left, right) => {
			if (left.publishedTimestamp !== undefined && right.publishedTimestamp !== undefined) {
				return right.publishedTimestamp - left.publishedTimestamp
			}
			if (left.publishedTimestamp !== undefined) {
				return -1
			}
			if (right.publishedTimestamp !== undefined) {
				return 1
			}

			return left.option.label.localeCompare(right.option.label)
		})
		.map(({ option }) => option)
}

export function buildProjectVersionFilterOptionProjectMetadataById(
	versionIds: string[],
	versionProjectNamesById: Map<string, string>,
	versionProjectIconUrlsById: Map<string, string>,
): Map<string, ProjectVersionFilterOptionProjectMetadata[]> {
	const metadataById = new Map<string, ProjectVersionFilterOptionProjectMetadata[]>()

	for (const versionId of versionIds) {
		const projectName = versionProjectNamesById.get(versionId)
		if (!projectName) {
			continue
		}

		const metadata: ProjectVersionFilterOptionProjectMetadata = { name: projectName }
		const iconUrl = versionProjectIconUrlsById.get(versionId)
		if (iconUrl) {
			metadata.iconUrl = iconUrl
		}

		metadataById.set(versionId, [metadata])
	}

	return metadataById
}

function intersectAnalyticsStats(
	left: readonly AnalyticsDashboardStat[],
	right: readonly AnalyticsDashboardStat[],
): AnalyticsDashboardStat[] {
	const rightStats = new Set(right)
	return left.filter((stat) => rightStats.has(stat))
}

function haveAnalyticsStatOverlap(
	left: readonly AnalyticsDashboardStat[],
	right: readonly AnalyticsDashboardStat[],
): boolean {
	return left.some((stat) => right.includes(stat))
}

export function getAnalyticsStatsForDimension(
	dimension: AnalyticsDashboardDimension,
): readonly AnalyticsDashboardStat[] {
	return ANALYTICS_STATS_BY_DIMENSION[dimension]
}

export function getAnalyticsStatsForBreakdown(
	breakdown: AnalyticsBreakdownPreset,
): readonly AnalyticsDashboardStat[] {
	return getAnalyticsStatsForDimension(ANALYTICS_DIMENSION_BY_BREAKDOWN[breakdown])
}

function normalizeAnalyticsBreakdowns(
	breakdowns: AnalyticsBreakdownInput,
): Exclude<AnalyticsBreakdownPreset, 'none'>[] {
	const values = Array.isArray(breakdowns) ? breakdowns : [breakdowns]
	const normalizedBreakdowns: Exclude<AnalyticsBreakdownPreset, 'none'>[] = []

	for (const breakdown of values) {
		if (breakdown === 'none') {
			continue
		}
		if (!normalizedBreakdowns.includes(breakdown)) {
			normalizedBreakdowns.push(breakdown)
		}
	}

	return normalizedBreakdowns
}

export function getAnalyticsStatsForBreakdowns(
	breakdowns: AnalyticsBreakdownInput,
): readonly AnalyticsDashboardStat[] {
	const normalizedBreakdowns = normalizeAnalyticsBreakdowns(breakdowns)
	if (normalizedBreakdowns.length === 0) {
		return getAnalyticsStatsForBreakdown('none')
	}

	let stats = [...getAnalyticsStatsForBreakdown(normalizedBreakdowns[0])]
	for (const breakdown of normalizedBreakdowns.slice(1)) {
		stats = intersectAnalyticsStats(stats, getAnalyticsStatsForBreakdown(breakdown))
	}

	return stats
}

export function getAnalyticsBreakdownsWithSharedStats(
	breakdowns: AnalyticsBreakdownInput,
): AnalyticsSelectedBreakdowns {
	const normalizedBreakdowns = normalizeAnalyticsBreakdowns(breakdowns)
	const compatibleBreakdowns: AnalyticsSelectedBreakdowns = []
	let sharedStats: readonly AnalyticsDashboardStat[] | null = null

	for (const breakdown of normalizedBreakdowns) {
		const breakdownStats = getAnalyticsStatsForBreakdown(breakdown)
		if (sharedStats === null) {
			compatibleBreakdowns.push(breakdown)
			sharedStats = breakdownStats
			continue
		}

		const nextSharedStats = intersectAnalyticsStats(sharedStats, breakdownStats)
		if (nextSharedStats.length === 0) {
			continue
		}

		compatibleBreakdowns.push(breakdown)
		sharedStats = nextSharedStats
	}

	return compatibleBreakdowns
}

export function getAnalyticsStatsForFilterCategory(
	category: AnalyticsQueryFilterCategory,
): readonly AnalyticsDashboardStat[] {
	if (category === 'project') {
		return ANALYTICS_DASHBOARD_STAT_ORDER
	}

	return getAnalyticsStatsForDimension(ANALYTICS_DIMENSION_BY_FILTER_CATEGORY[category])
}

export function getAnalyticsFilterCategoryForBreakdown(
	breakdown: AnalyticsBreakdownPreset,
): Exclude<AnalyticsQueryFilterCategory, 'project'> | null {
	return ANALYTICS_FILTER_CATEGORY_BY_BREAKDOWN[breakdown]
}

function getAnalyticsStatsForFilterScope(
	breakdowns: AnalyticsBreakdownInput,
	filters: AnalyticsSelectedFilters,
	ignoredCategory?: AnalyticsQueryFilterCategory,
): readonly AnalyticsDashboardStat[] {
	let stats = [...getAnalyticsStatsForBreakdowns(breakdowns)]

	for (const category of FILTER_VALUE_CATEGORIES) {
		if (category === ignoredCategory || filters[category].length === 0) {
			continue
		}

		stats = intersectAnalyticsStats(stats, getAnalyticsStatsForFilterCategory(category))
	}

	return stats
}

export function getEnabledAnalyticsStatsForState(
	breakdowns: AnalyticsBreakdownInput,
	filters: AnalyticsSelectedFilters,
): readonly AnalyticsDashboardStat[] {
	return getAnalyticsStatsForFilterScope(breakdowns, filters)
}

export function getVisibleAnalyticsFilterCategoriesForState(
	breakdowns: AnalyticsBreakdownInput,
	filters: AnalyticsSelectedFilters,
): readonly Exclude<AnalyticsQueryFilterCategory, 'project'>[] {
	const normalizedBreakdowns = normalizeAnalyticsBreakdowns(breakdowns)
	return FILTER_VALUE_CATEGORIES.filter((category) => {
		if (
			category === 'dependent_project_type' &&
			!normalizedBreakdowns.includes('dependent_project_download')
		) {
			return false
		}

		return haveAnalyticsStatOverlap(
			getAnalyticsStatsForFilterScope(breakdowns, filters, category),
			getAnalyticsStatsForFilterCategory(category),
		)
	})
}

export function sanitizeAnalyticsSelectedFilters(
	breakdowns: AnalyticsBreakdownInput,
	filters: AnalyticsSelectedFilters,
): AnalyticsSelectedFilters {
	const nextFilters = cloneSelectedFilters(filters)
	let availableStats = [...getAnalyticsStatsForBreakdowns(breakdowns)]
	const normalizedBreakdowns = normalizeAnalyticsBreakdowns(breakdowns)

	for (const category of FILTER_VALUE_CATEGORIES) {
		if (filters[category].length === 0) {
			continue
		}

		const categoryStats = getAnalyticsStatsForFilterCategory(category)
		if (
			!haveAnalyticsStatOverlap(availableStats, categoryStats) ||
			(category === 'dependent_project_type' &&
				!normalizedBreakdowns.includes('dependent_project_download'))
		) {
			nextFilters[category] = []
			continue
		}

		availableStats = intersectAnalyticsStats(availableStats, categoryStats)
	}

	return nextFilters
}

export function cloneSelectedFilters(filters: AnalyticsSelectedFilters): AnalyticsSelectedFilters {
	return {
		project: [...filters.project],
		project_status: [...filters.project_status],
		country: [...filters.country],
		monetization: [...filters.monetization],
		user_agent: [...filters.user_agent],
		download_reason: [...filters.download_reason],
		user_id: [...filters.user_id],
		version_id: [...filters.version_id],
		game_version: [...filters.game_version],
		loader_type: [...filters.loader_type],
		dependent_project_id: [...filters.dependent_project_id],
		dependent_project_type: [...filters.dependent_project_type],
	}
}

export function areStringArraysEqual(left: string[], right: string[]): boolean {
	if (left.length !== right.length) {
		return false
	}

	for (let index = 0; index < left.length; index += 1) {
		if (left[index] !== right[index]) {
			return false
		}
	}

	return true
}

export function areSelectedFiltersEqual(
	left: AnalyticsSelectedFilters,
	right: AnalyticsSelectedFilters,
): boolean {
	if (!areStringArraysEqual(left.project, right.project)) {
		return false
	}

	for (const categoryKey of FILTER_VALUE_CATEGORIES) {
		if (!areStringArraysEqual(left[categoryKey], right[categoryKey])) {
			return false
		}
	}

	return true
}

export function getOptionsWithSelectedValues(
	options: FilterOption[],
	selectedValues: string[],
	getMissingSelectedOptionLabel: (value: string) => string = (value) => value,
): FilterOption[] {
	if (selectedValues.length === 0) {
		return options
	}

	const knownValues = new Set(options.map((option) => option.value))
	const missingSelectedOptions = selectedValues
		.filter((value) => !knownValues.has(value))
		.map((value) => ({
			value,
			label: getMissingSelectedOptionLabel(value),
		}))

	return missingSelectedOptions.length === 0 ? options : [...options, ...missingSelectedOptions]
}

export function normalizeSelectedValues(
	categoryKey: AnalyticsQueryFilterCategory,
	values: string[],
	projectIds: string[],
): string[] {
	const uniqueValues = Array.from(new Set(values))

	if (categoryKey === 'project') {
		if (uniqueValues.includes(ALL_FILTER_VALUE)) {
			return projectIds
		}

		const allProjectIds = new Set(projectIds)
		const selectedProjects = uniqueValues.filter((value) => allProjectIds.has(value))

		return selectedProjects.length > 0 ? selectedProjects : projectIds
	}

	if (uniqueValues.includes(ALL_FILTER_VALUE) || uniqueValues.length === 0) {
		return []
	}

	const selectedValues = uniqueValues.filter((value) => value !== ALL_FILTER_VALUE)
	if (categoryKey === 'project_status') {
		return selectedValues
			.map((value) => value.trim().toLowerCase())
			.filter(isProjectStatusFilterValue)
	}
	if (categoryKey === 'loader_type' || categoryKey === 'dependent_project_type') {
		return Array.from(
			new Set(
				selectedValues
					.map((value) => value.trim().toLowerCase())
					.filter((value) => value.length > 0),
			),
		)
	}

	return selectedValues
}

export const PROJECT_STATUS_FILTER_VALUES = [
	'approved',
	'archived',
	'rejected',
	'draft',
	'unlisted',
	'withheld',
	'private',
	'other',
] as const

export type ProjectStatusFilterValue = (typeof PROJECT_STATUS_FILTER_VALUES)[number]

const projectStatusFilterValueSet = new Set<string>(PROJECT_STATUS_FILTER_VALUES)

export function isProjectStatusFilterValue(value: string): value is ProjectStatusFilterValue {
	return projectStatusFilterValueSet.has(value)
}

export function getProjectStatusFilterValue(
	status: string | null | undefined,
): ProjectStatusFilterValue {
	const normalizedStatus = status?.trim().toLowerCase() ?? ''
	return isProjectStatusFilterValue(normalizedStatus) ? normalizedStatus : 'other'
}
