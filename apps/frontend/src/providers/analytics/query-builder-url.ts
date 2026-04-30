import type { LocationQuery, LocationQueryValue, LocationQueryValueRaw } from 'vue-router'

export type AnalyticsQueryFilterCategory =
	| 'project'
	| 'country'
	| 'monetization'
	| 'download_source'
	| 'version_id'
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

export type AnalyticsTimeframeMode = 'preset' | 'last' | 'custom_range'
export type AnalyticsLastTimeframeUnit = 'hours' | 'days' | 'weeks' | 'months'

export type AnalyticsGroupByPreset = '1h' | '6h' | 'day' | 'week' | 'month' | 'year'

export type AnalyticsBreakdownPreset =
	| 'none'
	| 'country'
	| 'monetization'
	| 'download_source'
	| 'version_id'
	| 'loader'
	| 'game_version'

export type AnalyticsSelectedFilters = Record<AnalyticsQueryFilterCategory, string[]>

export type AnalyticsQueryBuilderState = {
	selectedProjectIds: string[]
	selectedTimeframeMode: AnalyticsTimeframeMode
	selectedTimeframe: AnalyticsTimeframePreset
	selectedLastTimeframeAmount: number
	selectedLastTimeframeUnit: AnalyticsLastTimeframeUnit
	selectedCustomTimeframeStartDate: string
	selectedCustomTimeframeEndDate: string
	selectedGroupBy: AnalyticsGroupByPreset
	selectedBreakdown: AnalyticsBreakdownPreset
	selectedFilters: AnalyticsSelectedFilters
}

type MutableRouteQuery = Record<string, LocationQueryValueRaw | LocationQueryValueRaw[] | undefined>

export const DEFAULT_TIMEFRAME_PRESET: AnalyticsTimeframePreset = 'yesterday'
export const DEFAULT_TIMEFRAME_MODE: AnalyticsTimeframeMode = 'preset'
export const DEFAULT_LAST_TIMEFRAME_AMOUNT = 1
export const DEFAULT_LAST_TIMEFRAME_UNIT: AnalyticsLastTimeframeUnit = 'days'
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

const TIMEFRAME_MODE_VALUES: AnalyticsTimeframeMode[] = ['preset', 'last', 'custom_range']
const LAST_TIMEFRAME_UNIT_VALUES: AnalyticsLastTimeframeUnit[] = [
	'hours',
	'days',
	'weeks',
	'months',
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
	'version_id',
	'loader',
	'game_version',
]

const QUERY_KEY_PROJECT_IDS = 'a_projects'
const QUERY_KEY_TIMEFRAME_MODE = 'a_timeframe_mode'
const QUERY_KEY_TIMEFRAME = 'a_timeframe'
const QUERY_KEY_TIMEFRAME_LAST_AMOUNT = 'a_timeframe_last_amount'
const QUERY_KEY_TIMEFRAME_LAST_UNIT = 'a_timeframe_last_unit'
const QUERY_KEY_TIMEFRAME_START = 'a_timeframe_start'
const QUERY_KEY_TIMEFRAME_END = 'a_timeframe_end'
const QUERY_KEY_GROUP_BY = 'a_group_by'
const QUERY_KEY_BREAKDOWN = 'a_breakdown'
const QUERY_KEY_FILTER_COUNTRY = 'a_country'
const QUERY_KEY_FILTER_MONETIZATION = 'a_monetization'
const QUERY_KEY_FILTER_DOWNLOAD_SOURCE = 'a_download_source'
const QUERY_KEY_FILTER_VERSION_ID = 'a_version_id'
const QUERY_KEY_FILTER_GAME_VERSION = 'a_game_version'
const QUERY_KEY_FILTER_LOADER_TYPE = 'a_loader_type'

const URL_FILTER_CATEGORIES: Exclude<AnalyticsQueryFilterCategory, 'project'>[] = [
	'country',
	'monetization',
	'download_source',
	'version_id',
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
	version_id: QUERY_KEY_FILTER_VERSION_ID,
	game_version: QUERY_KEY_FILTER_GAME_VERSION,
	loader_type: QUERY_KEY_FILTER_LOADER_TYPE,
}

const ANALYTICS_QUERY_KEYS = [
	QUERY_KEY_PROJECT_IDS,
	QUERY_KEY_TIMEFRAME_MODE,
	QUERY_KEY_TIMEFRAME,
	QUERY_KEY_TIMEFRAME_LAST_AMOUNT,
	QUERY_KEY_TIMEFRAME_LAST_UNIT,
	QUERY_KEY_TIMEFRAME_START,
	QUERY_KEY_TIMEFRAME_END,
	QUERY_KEY_GROUP_BY,
	QUERY_KEY_BREAKDOWN,
	QUERY_KEY_FILTER_COUNTRY,
	QUERY_KEY_FILTER_MONETIZATION,
	QUERY_KEY_FILTER_DOWNLOAD_SOURCE,
	QUERY_KEY_FILTER_VERSION_ID,
	QUERY_KEY_FILTER_GAME_VERSION,
	QUERY_KEY_FILTER_LOADER_TYPE,
]

export function buildEmptySelectedFilters(): AnalyticsSelectedFilters {
	return {
		project: [],
		country: [],
		monetization: [],
		download_source: [],
		version_id: [],
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

function normalizeFilterQueryValues(
	category: Exclude<AnalyticsQueryFilterCategory, 'project'>,
	values: string[],
): string[] {
	if (category !== 'loader_type') {
		return values
	}

	return Array.from(
		new Set(
			values
				.map((value) => value.trim().toLowerCase())
				.filter((value) => value.length > 0),
		),
	)
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

function parsePositiveIntegerQueryValue(
	value: LocationQueryValue | LocationQueryValue[] | undefined,
	fallbackValue: number,
): number {
	const rawValue = Array.isArray(value) ? value[0] : value
	if (!rawValue) return fallbackValue

	const parsedValue = Number.parseInt(rawValue, 10)
	if (!Number.isFinite(parsedValue) || parsedValue < 1) return fallbackValue
	return parsedValue
}

function getLocalDateQueryValue(date: Date): string {
	const year = date.getFullYear()
	const month = String(date.getMonth() + 1).padStart(2, '0')
	const day = String(date.getDate()).padStart(2, '0')
	return `${year}-${month}-${day}`
}

function getDefaultCustomStartDate(): string {
	const date = new Date()
	date.setDate(date.getDate() - 1)
	return getLocalDateQueryValue(date)
}

function getDefaultCustomEndDate(): string {
	return getLocalDateQueryValue(new Date())
}

function parseDateQueryValue(
	value: LocationQueryValue | LocationQueryValue[] | undefined,
	fallbackValue: string,
): string {
	const rawValue = Array.isArray(value) ? value[0] : value
	if (!rawValue || !/^\d{4}-\d{2}-\d{2}$/.test(rawValue)) return fallbackValue

	const date = new Date(`${rawValue}T00:00:00`)
	if (Number.isNaN(date.getTime())) return fallbackValue
	if (getLocalDateQueryValue(date) !== rawValue) return fallbackValue

	return rawValue
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
		selectedFilters[category] = normalizeFilterQueryValues(
			category,
			parseListQueryValue(query[FILTER_QUERY_KEY_BY_CATEGORY[category]]),
		)
	}

	const selectedCustomTimeframeStartDate = parseDateQueryValue(
		query[QUERY_KEY_TIMEFRAME_START],
		getDefaultCustomStartDate(),
	)
	const rawCustomTimeframeEndDate = parseDateQueryValue(
		query[QUERY_KEY_TIMEFRAME_END],
		getDefaultCustomEndDate(),
	)
	const selectedCustomTimeframeEndDate =
		rawCustomTimeframeEndDate < selectedCustomTimeframeStartDate
			? selectedCustomTimeframeStartDate
			: rawCustomTimeframeEndDate

	return {
		selectedProjectIds,
		selectedTimeframeMode: parsePresetQueryValue(
			query[QUERY_KEY_TIMEFRAME_MODE],
			TIMEFRAME_MODE_VALUES,
			DEFAULT_TIMEFRAME_MODE,
		),
		selectedTimeframe: parsePresetQueryValue(
			query[QUERY_KEY_TIMEFRAME],
			TIMEFRAME_PRESET_VALUES,
			DEFAULT_TIMEFRAME_PRESET,
		),
		selectedLastTimeframeAmount: parsePositiveIntegerQueryValue(
			query[QUERY_KEY_TIMEFRAME_LAST_AMOUNT],
			DEFAULT_LAST_TIMEFRAME_AMOUNT,
		),
		selectedLastTimeframeUnit: parsePresetQueryValue(
			query[QUERY_KEY_TIMEFRAME_LAST_UNIT],
			LAST_TIMEFRAME_UNIT_VALUES,
			DEFAULT_LAST_TIMEFRAME_UNIT,
		),
		selectedCustomTimeframeStartDate,
		selectedCustomTimeframeEndDate,
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
	nextRouteQuery[QUERY_KEY_TIMEFRAME_MODE] =
		state.selectedTimeframeMode !== DEFAULT_TIMEFRAME_MODE ? state.selectedTimeframeMode : undefined
	nextRouteQuery[QUERY_KEY_TIMEFRAME] =
		state.selectedTimeframeMode === 'preset' && state.selectedTimeframe !== DEFAULT_TIMEFRAME_PRESET
			? state.selectedTimeframe
			: undefined
	nextRouteQuery[QUERY_KEY_TIMEFRAME_LAST_AMOUNT] =
		state.selectedTimeframeMode === 'last' ? String(state.selectedLastTimeframeAmount) : undefined
	nextRouteQuery[QUERY_KEY_TIMEFRAME_LAST_UNIT] =
		state.selectedTimeframeMode === 'last' ? state.selectedLastTimeframeUnit : undefined
	nextRouteQuery[QUERY_KEY_TIMEFRAME_START] =
		state.selectedTimeframeMode === 'custom_range'
			? state.selectedCustomTimeframeStartDate
			: undefined
	nextRouteQuery[QUERY_KEY_TIMEFRAME_END] =
		state.selectedTimeframeMode === 'custom_range'
			? state.selectedCustomTimeframeEndDate
			: undefined
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
