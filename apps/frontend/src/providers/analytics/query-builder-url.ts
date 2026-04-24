import type { LocationQuery, LocationQueryValue, LocationQueryValueRaw } from 'vue-router'

export type AnalyticsQueryFilterCategory =
	| 'project'
	| 'country'
	| 'monetization'
	| 'download_source'
	| 'download_type'
	| 'game_version'
	| 'loader_type'

export type AnalyticsTimeframePreset =
	| 'today'
	| 'yesterday'
	| 'last_7_days'
	| 'last_14_days'
	| 'last_30_days'
	| 'last_90_days'
	| 'last_180_days'
	| 'year_to_date'
	| 'all_time'

export type AnalyticsGroupByPreset = '1h' | '6h' | 'day' | 'week' | 'month' | 'year'

export type AnalyticsBreakdownPreset =
	| 'none'
	| 'country'
	| 'monetization'
	| 'download_source'
	| 'download_type'
	| 'loader'
	| 'game_version'

export type AnalyticsSelectedFilters = Record<AnalyticsQueryFilterCategory, string[]>

export type AnalyticsQueryBuilderState = {
	selectedProjectIds: string[]
	selectedTimeframe: AnalyticsTimeframePreset
	selectedGroupBy: AnalyticsGroupByPreset
	selectedBreakdown: AnalyticsBreakdownPreset
	selectedFilters: AnalyticsSelectedFilters
}

type MutableRouteQuery = Record<string, LocationQueryValueRaw | LocationQueryValueRaw[] | undefined>

export const DEFAULT_TIMEFRAME_PRESET: AnalyticsTimeframePreset = 'yesterday'
export const DEFAULT_GROUP_BY_PRESET: AnalyticsGroupByPreset = '1h'
export const DEFAULT_BREAKDOWN_PRESET: AnalyticsBreakdownPreset = 'none'

const TIMEFRAME_PRESET_VALUES: AnalyticsTimeframePreset[] = [
	'today',
	'yesterday',
	'last_7_days',
	'last_14_days',
	'last_30_days',
	'last_90_days',
	'last_180_days',
	'year_to_date',
	'all_time',
]

const GROUP_BY_PRESET_VALUES: AnalyticsGroupByPreset[] = [
	'1h',
	'6h',
	'day',
	'week',
	'month',
	'year',
]

const BREAKDOWN_PRESET_VALUES: AnalyticsBreakdownPreset[] = [
	'none',
	'country',
	'monetization',
	'download_source',
	'download_type',
	'loader',
	'game_version',
]

const QUERY_KEY_PROJECT_IDS = 'a_projects'
const QUERY_KEY_TIMEFRAME = 'a_timeframe'
const QUERY_KEY_GROUP_BY = 'a_group_by'
const QUERY_KEY_BREAKDOWN = 'a_breakdown'
const QUERY_KEY_FILTER_COUNTRY = 'a_country'
const QUERY_KEY_FILTER_MONETIZATION = 'a_monetization'
const QUERY_KEY_FILTER_DOWNLOAD_SOURCE = 'a_download_source'
const QUERY_KEY_FILTER_DOWNLOAD_TYPE = 'a_download_type'
const QUERY_KEY_FILTER_GAME_VERSION = 'a_game_version'
const QUERY_KEY_FILTER_LOADER_TYPE = 'a_loader_type'

const URL_FILTER_CATEGORIES: Exclude<AnalyticsQueryFilterCategory, 'project'>[] = [
	'country',
	'monetization',
	'download_source',
	'download_type',
	'game_version',
	'loader_type',
]

const FILTER_QUERY_KEY_BY_CATEGORY: Record<
	Exclude<AnalyticsQueryFilterCategory, 'project'>,
	string
> = {
	country: QUERY_KEY_FILTER_COUNTRY,
	monetization: QUERY_KEY_FILTER_MONETIZATION,
	download_source: QUERY_KEY_FILTER_DOWNLOAD_SOURCE,
	download_type: QUERY_KEY_FILTER_DOWNLOAD_TYPE,
	game_version: QUERY_KEY_FILTER_GAME_VERSION,
	loader_type: QUERY_KEY_FILTER_LOADER_TYPE,
}

const ANALYTICS_QUERY_KEYS = [
	QUERY_KEY_PROJECT_IDS,
	QUERY_KEY_TIMEFRAME,
	QUERY_KEY_GROUP_BY,
	QUERY_KEY_BREAKDOWN,
	QUERY_KEY_FILTER_COUNTRY,
	QUERY_KEY_FILTER_MONETIZATION,
	QUERY_KEY_FILTER_DOWNLOAD_SOURCE,
	QUERY_KEY_FILTER_DOWNLOAD_TYPE,
	QUERY_KEY_FILTER_GAME_VERSION,
	QUERY_KEY_FILTER_LOADER_TYPE,
]

export function buildEmptySelectedFilters(): AnalyticsSelectedFilters {
	return {
		project: [],
		country: [],
		monetization: [],
		download_source: [],
		download_type: [],
		game_version: [],
		loader_type: [],
	}
}

function parseListQueryValue(
	value: LocationQueryValue | LocationQueryValue[] | undefined,
): string[] {
	if (value === undefined) return []

	const values = Array.isArray(value) ? value : [value]
	const parsedValues: string[] = []
	for (const item of values) {
		if (!item) continue
		const parts = item.split(',')
		for (const part of parts) {
			const trimmed = part.trim()
			if (trimmed.length > 0) {
				parsedValues.push(trimmed)
			}
		}
	}

	return Array.from(new Set(parsedValues))
}

function parsePresetQueryValue<T extends string>(
	value: LocationQueryValue | LocationQueryValue[] | undefined,
	allowedValues: readonly T[],
	fallbackValue: T,
): T {
	const rawValue = Array.isArray(value) ? value[0] : value
	if (!rawValue) return fallbackValue
	if (!allowedValues.includes(rawValue as T)) return fallbackValue
	return rawValue as T
}

function serializeListQueryValue(values: string[]): string | undefined {
	if (values.length === 0) return undefined
	return values.join(',')
}

function normalizeQueryValue(
	value:
		| LocationQueryValue
		| LocationQueryValue[]
		| LocationQueryValueRaw
		| LocationQueryValueRaw[]
		| undefined,
): string[] {
	if (value === undefined || value === null) return []
	if (Array.isArray(value)) {
		return value
			.filter(
				(item): item is LocationQueryValue | LocationQueryValueRaw =>
					item !== undefined && item !== null,
			)
			.map((item) => String(item))
	}
	return [String(value)]
}

function areQueryValuesEqual(
	left:
		| LocationQueryValue
		| LocationQueryValue[]
		| LocationQueryValueRaw
		| LocationQueryValueRaw[]
		| undefined,
	right:
		| LocationQueryValue
		| LocationQueryValue[]
		| LocationQueryValueRaw
		| LocationQueryValueRaw[]
		| undefined,
): boolean {
	const leftValues = normalizeQueryValue(left)
	const rightValues = normalizeQueryValue(right)

	if (leftValues.length !== rightValues.length) return false
	for (let index = 0; index < leftValues.length; index += 1) {
		if (leftValues[index] !== rightValues[index]) return false
	}
	return true
}

export function areStringArraysEqual(left: string[], right: string[]): boolean {
	if (left.length !== right.length) return false
	for (let index = 0; index < left.length; index += 1) {
		if (left[index] !== right[index]) return false
	}
	return true
}

export function areSelectedFiltersEqual(
	left: AnalyticsSelectedFilters,
	right: AnalyticsSelectedFilters,
): boolean {
	if (!areStringArraysEqual(left.project, right.project)) return false
	for (const category of URL_FILTER_CATEGORIES) {
		if (!areStringArraysEqual(left[category], right[category])) return false
	}
	return true
}

function areAllProjectsSelected(selectedProjectIds: string[], allProjectIds: string[]): boolean {
	if (allProjectIds.length === 0 || selectedProjectIds.length !== allProjectIds.length) {
		return false
	}
	const allProjectIdSet = new Set(allProjectIds)
	return selectedProjectIds.every((projectId) => allProjectIdSet.has(projectId))
}

export function readAnalyticsQueryBuilderState(
	query: LocationQuery,
	availableProjectIds: string[],
): AnalyticsQueryBuilderState {
	const selectedProjectIdsFromQuery = parseListQueryValue(query[QUERY_KEY_PROJECT_IDS])
	const selectedProjectIds =
		selectedProjectIdsFromQuery.length > 0 ? selectedProjectIdsFromQuery : availableProjectIds

	const selectedFilters = buildEmptySelectedFilters()
	for (const category of URL_FILTER_CATEGORIES) {
		selectedFilters[category] = parseListQueryValue(query[FILTER_QUERY_KEY_BY_CATEGORY[category]])
	}

	return {
		selectedProjectIds,
		selectedTimeframe: parsePresetQueryValue(
			query[QUERY_KEY_TIMEFRAME],
			TIMEFRAME_PRESET_VALUES,
			DEFAULT_TIMEFRAME_PRESET,
		),
		selectedGroupBy: parsePresetQueryValue(
			query[QUERY_KEY_GROUP_BY],
			GROUP_BY_PRESET_VALUES,
			DEFAULT_GROUP_BY_PRESET,
		),
		selectedBreakdown: parsePresetQueryValue(
			query[QUERY_KEY_BREAKDOWN],
			BREAKDOWN_PRESET_VALUES,
			DEFAULT_BREAKDOWN_PRESET,
		),
		selectedFilters,
	}
}

export function buildAnalyticsQueryBuilderRouteQuery(
	currentRouteQuery: LocationQuery,
	state: AnalyticsQueryBuilderState,
	availableProjectIds: string[],
): MutableRouteQuery {
	const nextRouteQuery = {
		...currentRouteQuery,
	} as MutableRouteQuery

	const projectIdsQueryValue = areAllProjectsSelected(state.selectedProjectIds, availableProjectIds)
		? undefined
		: serializeListQueryValue(state.selectedProjectIds)

	nextRouteQuery[QUERY_KEY_PROJECT_IDS] = projectIdsQueryValue
	nextRouteQuery[QUERY_KEY_TIMEFRAME] =
		state.selectedTimeframe !== DEFAULT_TIMEFRAME_PRESET ? state.selectedTimeframe : undefined
	nextRouteQuery[QUERY_KEY_GROUP_BY] =
		state.selectedGroupBy !== DEFAULT_GROUP_BY_PRESET ? state.selectedGroupBy : undefined
	nextRouteQuery[QUERY_KEY_BREAKDOWN] =
		state.selectedBreakdown !== DEFAULT_BREAKDOWN_PRESET ? state.selectedBreakdown : undefined

	for (const category of URL_FILTER_CATEGORIES) {
		const categoryQueryKey = FILTER_QUERY_KEY_BY_CATEGORY[category]
		nextRouteQuery[categoryQueryKey] = serializeListQueryValue(state.selectedFilters[category])
	}

	return nextRouteQuery
}

export function hasAnalyticsQueryBuilderRouteChange(
	currentRouteQuery: LocationQuery,
	nextRouteQuery: MutableRouteQuery,
): boolean {
	return ANALYTICS_QUERY_KEYS.some(
		(key) => !areQueryValuesEqual(currentRouteQuery[key], nextRouteQuery[key]),
	)
}
