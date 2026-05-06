import type {
	AnalyticsBreakdownPreset,
	AnalyticsDashboardStat,
	AnalyticsQueryFilterCategory,
	AnalyticsSelectedFilters,
} from '~/providers/analytics/analytics'

export type AnalyticsDashboardDimension =
	| 'project'
	| 'version_id'
	| 'country'
	| 'monetization'
	| 'download_source'
	| 'download_reason'
	| 'game_version'
	| 'loader_type'

export const ALL_FILTER_VALUE = '__all__'
export const FILTER_VALUE_CATEGORIES: Exclude<AnalyticsQueryFilterCategory, 'project'>[] = [
	'country',
	'monetization',
	'download_source',
	'download_reason',
	'version_id',
	'game_version',
	'loader_type',
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
	download_source: ['downloads'],
	download_reason: ['downloads'],
	game_version: ['downloads', 'playtime'],
	loader_type: ['downloads', 'playtime'],
}

const ANALYTICS_DIMENSION_BY_BREAKDOWN: Record<
	AnalyticsBreakdownPreset,
	AnalyticsDashboardDimension
> = {
	none: 'project',
	country: 'country',
	monetization: 'monetization',
	download_source: 'download_source',
	download_reason: 'download_reason',
	version_id: 'version_id',
	loader: 'loader_type',
	game_version: 'game_version',
}

const ANALYTICS_DIMENSION_BY_FILTER_CATEGORY: Record<
	Exclude<AnalyticsQueryFilterCategory, 'project'>,
	AnalyticsDashboardDimension
> = {
	country: 'country',
	monetization: 'monetization',
	download_source: 'download_source',
	download_reason: 'download_reason',
	version_id: 'version_id',
	game_version: 'game_version',
	loader_type: 'loader_type',
}

export type FilterOption = {
	value: string
	label: string
	searchTerms?: string[]
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

export function getAnalyticsStatsForFilterCategory(
	category: AnalyticsQueryFilterCategory,
): readonly AnalyticsDashboardStat[] {
	if (category === 'project') {
		return ANALYTICS_DASHBOARD_STAT_ORDER
	}

	return getAnalyticsStatsForDimension(ANALYTICS_DIMENSION_BY_FILTER_CATEGORY[category])
}

function getAnalyticsStatsForFilterScope(
	breakdown: AnalyticsBreakdownPreset,
	filters: AnalyticsSelectedFilters,
	ignoredCategory?: AnalyticsQueryFilterCategory,
): readonly AnalyticsDashboardStat[] {
	let stats = [...getAnalyticsStatsForBreakdown(breakdown)]

	for (const category of FILTER_VALUE_CATEGORIES) {
		if (category === ignoredCategory || filters[category].length === 0) {
			continue
		}

		stats = intersectAnalyticsStats(stats, getAnalyticsStatsForFilterCategory(category))
	}

	return stats
}

export function getEnabledAnalyticsStatsForState(
	breakdown: AnalyticsBreakdownPreset,
	filters: AnalyticsSelectedFilters,
): readonly AnalyticsDashboardStat[] {
	return getAnalyticsStatsForFilterScope(breakdown, filters)
}

export function getVisibleAnalyticsFilterCategoriesForState(
	breakdown: AnalyticsBreakdownPreset,
	filters: AnalyticsSelectedFilters,
): readonly Exclude<AnalyticsQueryFilterCategory, 'project'>[] {
	return FILTER_VALUE_CATEGORIES.filter((category) =>
		haveAnalyticsStatOverlap(
			getAnalyticsStatsForFilterScope(breakdown, filters, category),
			getAnalyticsStatsForFilterCategory(category),
		),
	)
}

export function sanitizeAnalyticsSelectedFilters(
	breakdown: AnalyticsBreakdownPreset,
	filters: AnalyticsSelectedFilters,
): AnalyticsSelectedFilters {
	const nextFilters = cloneSelectedFilters(filters)
	let availableStats = [...getAnalyticsStatsForBreakdown(breakdown)]

	for (const category of FILTER_VALUE_CATEGORIES) {
		if (filters[category].length === 0) {
			continue
		}

		const categoryStats = getAnalyticsStatsForFilterCategory(category)
		if (!haveAnalyticsStatOverlap(availableStats, categoryStats)) {
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
		country: [...filters.country],
		monetization: [...filters.monetization],
		download_source: [...filters.download_source],
		download_reason: [...filters.download_reason],
		version_id: [...filters.version_id],
		game_version: [...filters.game_version],
		loader_type: [...filters.loader_type],
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
	const knownValues = new Set(options.map((option) => option.value))
	const missingSelectedOptions = selectedValues
		.filter((value) => !knownValues.has(value))
		.map((value) => ({
			value,
			label: getMissingSelectedOptionLabel(value),
		}))

	return [...options, ...missingSelectedOptions]
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
	if (categoryKey === 'loader_type') {
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
