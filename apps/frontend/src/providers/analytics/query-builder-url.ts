import type { LocationQuery, LocationQueryValue, LocationQueryValueRaw } from 'vue-router'

export type AnalyticsQueryFilterCategory =
	| 'project'
	| 'project_status'
	| 'country'
	| 'monetization'
	| 'user_agent'
	| 'download_reason'
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

export type AnalyticsTimeframeMode = 'preset' | 'last' | 'custom_range' | 'custom_datetime_range'
export type AnalyticsLastTimeframeUnit = 'hours' | 'days' | 'weeks' | 'months'

export type AnalyticsGroupByPreset = '1h' | '6h' | 'day' | 'week' | 'month' | 'year'

export type AnalyticsBreakdownPreset =
	| 'none'
	| 'project'
	| 'country'
	| 'monetization'
	| 'user_agent'
	| 'download_reason'
	| 'version_id'
	| 'loader'
	| 'game_version'

export type AnalyticsDashboardStat = 'views' | 'downloads' | 'revenue' | 'playtime'
export type AnalyticsGraphViewMode = 'line' | 'area' | 'bar'

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

export type AnalyticsGraphState = {
	activeStat: AnalyticsDashboardStat
	activeGraphViewMode: AnalyticsGraphViewMode
	isRatioMode: boolean
	showChartEvents: boolean
	hiddenGraphDatasetIds: string[]
}

type MutableRouteQuery = Record<string, LocationQueryValueRaw | LocationQueryValueRaw[] | undefined>

export const DEFAULT_TIMEFRAME_PRESET: AnalyticsTimeframePreset = 'last_30_days'
export const DEFAULT_TIMEFRAME_MODE: AnalyticsTimeframeMode = 'preset'
export const DEFAULT_LAST_TIMEFRAME_AMOUNT = 1
export const DEFAULT_LAST_TIMEFRAME_UNIT: AnalyticsLastTimeframeUnit = 'days'
export const DEFAULT_GROUP_BY_PRESET: AnalyticsGroupByPreset = 'day'
export const DEFAULT_BREAKDOWN_PRESET: AnalyticsBreakdownPreset = 'none'
export const DEFAULT_ANALYTICS_DASHBOARD_STAT: AnalyticsDashboardStat = 'views'
export const DEFAULT_ANALYTICS_GRAPH_VIEW_MODE: AnalyticsGraphViewMode = 'line'
export const DEFAULT_ANALYTICS_GRAPH_RATIO_MODE = false
export const DEFAULT_ANALYTICS_GRAPH_EVENTS_VISIBILITY = true

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

const TIMEFRAME_MODE_VALUES: AnalyticsTimeframeMode[] = [
	'preset',
	'last',
	'custom_range',
	'custom_datetime_range',
]
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
	'project',
	'country',
	'monetization',
	'user_agent',
	'download_reason',
	'version_id',
	'loader',
	'game_version',
]

const ANALYTICS_DASHBOARD_STAT_VALUES: AnalyticsDashboardStat[] = [
	'views',
	'downloads',
	'revenue',
	'playtime',
]

const ANALYTICS_GRAPH_VIEW_MODE_VALUES: AnalyticsGraphViewMode[] = ['line', 'area', 'bar']

const PROJECT_STATUS_FILTER_VALUES = [
	'approved',
	'archived',
	'rejected',
	'draft',
	'unlisted',
	'withheld',
	'private',
	'other',
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
const QUERY_KEY_FILTER_PROJECT_STATUS = 'a_project_status'
const QUERY_KEY_FILTER_COUNTRY = 'a_country'
const QUERY_KEY_FILTER_MONETIZATION = 'a_monetization'
const QUERY_KEY_FILTER_USER_AGENT = 'a_user_agent'
const QUERY_KEY_FILTER_LEGACY_DOWNLOAD_SOURCE = 'a_download_source'
const QUERY_KEY_FILTER_DOWNLOAD_REASON = 'a_download_reason'
const QUERY_KEY_FILTER_VERSION_ID = 'a_version_id'
const QUERY_KEY_FILTER_GAME_VERSION = 'a_game_version'
const QUERY_KEY_FILTER_LOADER_TYPE = 'a_loader_type'
const QUERY_KEY_STAT = 'a_stat'
const QUERY_KEY_GRAPH_VIEW_MODE = 'a_chart'
const QUERY_KEY_GRAPH_RATIO_MODE = 'a_ratio'
const QUERY_KEY_GRAPH_EVENTS_VISIBILITY = 'a_events'
const QUERY_KEY_GRAPH_HIDDEN_SERIES = 'a_hidden_series'
const QUERY_KEY_LEGACY_GRAPH_TOP_BREAKDOWN_FILTER = 'a_top_breakdown'
const QUERY_KEY_LEGACY_GRAPH_LEGEND_EXPANSION = 'a_legend_expanded'

const URL_FILTER_CATEGORIES: Exclude<AnalyticsQueryFilterCategory, 'project'>[] = [
	'project_status',
	'country',
	'monetization',
	'user_agent',
	'download_reason',
	'version_id',
	'game_version',
	'loader_type',
]

const FILTER_QUERY_KEY_BY_CATEGORY: Record<
	Exclude<AnalyticsQueryFilterCategory, 'project'>,
	string
> = {
	project_status: QUERY_KEY_FILTER_PROJECT_STATUS,
	country: QUERY_KEY_FILTER_COUNTRY,
	monetization: QUERY_KEY_FILTER_MONETIZATION,
	user_agent: QUERY_KEY_FILTER_USER_AGENT,
	download_reason: QUERY_KEY_FILTER_DOWNLOAD_REASON,
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
	QUERY_KEY_FILTER_PROJECT_STATUS,
	QUERY_KEY_FILTER_COUNTRY,
	QUERY_KEY_FILTER_MONETIZATION,
	QUERY_KEY_FILTER_USER_AGENT,
	QUERY_KEY_FILTER_LEGACY_DOWNLOAD_SOURCE,
	QUERY_KEY_FILTER_DOWNLOAD_REASON,
	QUERY_KEY_FILTER_VERSION_ID,
	QUERY_KEY_FILTER_GAME_VERSION,
	QUERY_KEY_FILTER_LOADER_TYPE,
	QUERY_KEY_STAT,
	QUERY_KEY_GRAPH_VIEW_MODE,
	QUERY_KEY_GRAPH_RATIO_MODE,
	QUERY_KEY_GRAPH_EVENTS_VISIBILITY,
	QUERY_KEY_GRAPH_HIDDEN_SERIES,
	QUERY_KEY_LEGACY_GRAPH_TOP_BREAKDOWN_FILTER,
	QUERY_KEY_LEGACY_GRAPH_LEGEND_EXPANSION,
]

export function buildEmptySelectedFilters(): AnalyticsSelectedFilters {
	return {
		project: [],
		project_status: [],
		country: [],
		monetization: [],
		user_agent: [],
		download_reason: [],
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
	if (category === 'project_status') {
		return values
			.map((value) => value.trim().toLowerCase())
			.filter((value) => PROJECT_STATUS_FILTER_VALUES.includes(value))
	}

	if (category !== 'loader_type') {
		return values
	}

	return Array.from(
		new Set(values.map((value) => value.trim().toLowerCase()).filter((value) => value.length > 0)),
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

function parseAnalyticsBreakdownQueryValue(
	value: LocationQueryValue | LocationQueryValue[] | undefined,
	fallbackValue: AnalyticsBreakdownPreset,
): AnalyticsBreakdownPreset {
	const rawValue = Array.isArray(value) ? value[0] : value
	if (rawValue === 'download_source') return 'user_agent'
	return parsePresetQueryValue(value, BREAKDOWN_PRESET_VALUES, fallbackValue)
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

function parseEnabledQueryValue(
	value: LocationQueryValue | LocationQueryValue[] | undefined,
): boolean {
	const rawValue = Array.isArray(value) ? value[0] : value
	return rawValue === '1'
}

function parseVisibleQueryValue(
	value: LocationQueryValue | LocationQueryValue[] | undefined,
): boolean {
	const rawValue = Array.isArray(value) ? value[0] : value
	return rawValue !== '0'
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

function getDefaultCustomDateTimeValue(value: string): string {
	return new Date(`${value}T00:00:00`).toISOString()
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

function parseDateTimeQueryValue(
	value: LocationQueryValue | LocationQueryValue[] | undefined,
	fallbackValue: string,
): string {
	const rawValue = Array.isArray(value) ? value[0] : value
	if (!rawValue || !/^\d{4}-\d{2}-\d{2}T/.test(rawValue)) return fallbackValue

	const date = new Date(rawValue)
	if (Number.isNaN(date.getTime())) return fallbackValue

	return date.toISOString()
}

function isTimeframeRangeEndBeforeStart(
	mode: AnalyticsTimeframeMode,
	startValue: string,
	endValue: string,
): boolean {
	if (mode === 'custom_datetime_range') {
		return new Date(endValue).getTime() < new Date(startValue).getTime()
	}

	return endValue < startValue
}

export function buildDefaultAnalyticsGraphState(): AnalyticsGraphState {
	return {
		activeStat: DEFAULT_ANALYTICS_DASHBOARD_STAT,
		activeGraphViewMode: DEFAULT_ANALYTICS_GRAPH_VIEW_MODE,
		isRatioMode: DEFAULT_ANALYTICS_GRAPH_RATIO_MODE,
		showChartEvents: DEFAULT_ANALYTICS_GRAPH_EVENTS_VISIBILITY,
		hiddenGraphDatasetIds: [],
	}
}

export function buildDefaultAnalyticsQueryBuilderState(
	availableProjectIds: string[],
): AnalyticsQueryBuilderState {
	return {
		selectedProjectIds: [...availableProjectIds],
		selectedTimeframeMode: DEFAULT_TIMEFRAME_MODE,
		selectedTimeframe: DEFAULT_TIMEFRAME_PRESET,
		selectedLastTimeframeAmount: DEFAULT_LAST_TIMEFRAME_AMOUNT,
		selectedLastTimeframeUnit: DEFAULT_LAST_TIMEFRAME_UNIT,
		selectedCustomTimeframeStartDate: getDefaultCustomStartDate(),
		selectedCustomTimeframeEndDate: getDefaultCustomEndDate(),
		selectedGroupBy: DEFAULT_GROUP_BY_PRESET,
		selectedBreakdown: getDefaultAnalyticsBreakdownPreset(availableProjectIds),
		selectedFilters: buildEmptySelectedFilters(),
	}
}

export function getDefaultAnalyticsBreakdownPreset(
	selectedProjectIds: readonly string[],
): AnalyticsBreakdownPreset {
	return selectedProjectIds.length > 1 ? 'project' : DEFAULT_BREAKDOWN_PRESET
}

export function isAnalyticsQueryBuilderStateDefault(
	state: AnalyticsQueryBuilderState,
	availableProjectIds: string[],
): boolean {
	const defaultState = buildDefaultAnalyticsQueryBuilderState(availableProjectIds)
	const areDefaultProjectsSelected =
		availableProjectIds.length === 0
			? state.selectedProjectIds.length === 0
			: areAllProjectsSelected(state.selectedProjectIds, availableProjectIds)

	return (
		areDefaultProjectsSelected &&
		state.selectedTimeframeMode === defaultState.selectedTimeframeMode &&
		state.selectedTimeframe === defaultState.selectedTimeframe &&
		state.selectedLastTimeframeAmount === defaultState.selectedLastTimeframeAmount &&
		state.selectedLastTimeframeUnit === defaultState.selectedLastTimeframeUnit &&
		state.selectedCustomTimeframeStartDate === defaultState.selectedCustomTimeframeStartDate &&
		state.selectedCustomTimeframeEndDate === defaultState.selectedCustomTimeframeEndDate &&
		state.selectedGroupBy === defaultState.selectedGroupBy &&
		state.selectedBreakdown === getDefaultAnalyticsBreakdownPreset(state.selectedProjectIds) &&
		areSelectedFiltersEqual(state.selectedFilters, defaultState.selectedFilters)
	)
}

export function isAnalyticsGraphStateDefault(state: AnalyticsGraphState): boolean {
	const defaultState = buildDefaultAnalyticsGraphState()

	return (
		state.activeStat === defaultState.activeStat &&
		state.activeGraphViewMode === defaultState.activeGraphViewMode &&
		state.isRatioMode === defaultState.isRatioMode &&
		state.showChartEvents === defaultState.showChartEvents &&
		areStringArraysEqual(state.hiddenGraphDatasetIds, defaultState.hiddenGraphDatasetIds)
	)
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

export function readAnalyticsGraphState(query: LocationQuery): AnalyticsGraphState {
	const defaultState = buildDefaultAnalyticsGraphState()

	return {
		activeStat: parsePresetQueryValue(
			query[QUERY_KEY_STAT],
			ANALYTICS_DASHBOARD_STAT_VALUES,
			defaultState.activeStat,
		),
		activeGraphViewMode: parsePresetQueryValue(
			query[QUERY_KEY_GRAPH_VIEW_MODE],
			ANALYTICS_GRAPH_VIEW_MODE_VALUES,
			defaultState.activeGraphViewMode,
		),
		isRatioMode: parseEnabledQueryValue(query[QUERY_KEY_GRAPH_RATIO_MODE]),
		showChartEvents: parseVisibleQueryValue(query[QUERY_KEY_GRAPH_EVENTS_VISIBILITY]),
		hiddenGraphDatasetIds: parseListQueryValue(query[QUERY_KEY_GRAPH_HIDDEN_SERIES]),
	}
}

export function readAnalyticsQueryBuilderState(
	query: LocationQuery,
	availableProjectIds: string[],
): AnalyticsQueryBuilderState {
	const defaultState = buildDefaultAnalyticsQueryBuilderState(availableProjectIds)
	const selectedProjectIdsFromQuery = parseListQueryValue(query[QUERY_KEY_PROJECT_IDS])
	const selectedProjectIds =
		selectedProjectIdsFromQuery.length > 0
			? selectedProjectIdsFromQuery
			: defaultState.selectedProjectIds

	const selectedFilters = buildEmptySelectedFilters()
	for (const category of URL_FILTER_CATEGORIES) {
		const categoryQueryKey = FILTER_QUERY_KEY_BY_CATEGORY[category]
		const rawQueryValue =
			category === 'user_agent' && query[categoryQueryKey] === undefined
				? query[QUERY_KEY_FILTER_LEGACY_DOWNLOAD_SOURCE]
				: query[categoryQueryKey]
		selectedFilters[category] = normalizeFilterQueryValues(
			category,
			parseListQueryValue(rawQueryValue),
		)
	}

	const selectedTimeframeMode = parsePresetQueryValue(
		query[QUERY_KEY_TIMEFRAME_MODE],
		TIMEFRAME_MODE_VALUES,
		defaultState.selectedTimeframeMode,
	)
	const isCustomDateTimeRange = selectedTimeframeMode === 'custom_datetime_range'
	const parseTimeframeRangeQueryValue = isCustomDateTimeRange
		? parseDateTimeQueryValue
		: parseDateQueryValue
	const customTimeframeStartFallback = isCustomDateTimeRange
		? getDefaultCustomDateTimeValue(defaultState.selectedCustomTimeframeStartDate)
		: defaultState.selectedCustomTimeframeStartDate
	const customTimeframeEndFallback = isCustomDateTimeRange
		? getDefaultCustomDateTimeValue(defaultState.selectedCustomTimeframeEndDate)
		: defaultState.selectedCustomTimeframeEndDate

	const selectedCustomTimeframeStartDate = parseTimeframeRangeQueryValue(
		query[QUERY_KEY_TIMEFRAME_START],
		customTimeframeStartFallback,
	)
	const rawCustomTimeframeEndDate = parseTimeframeRangeQueryValue(
		query[QUERY_KEY_TIMEFRAME_END],
		customTimeframeEndFallback,
	)
	const selectedCustomTimeframeEndDate = isTimeframeRangeEndBeforeStart(
		selectedTimeframeMode,
		selectedCustomTimeframeStartDate,
		rawCustomTimeframeEndDate,
	)
		? selectedCustomTimeframeStartDate
		: rawCustomTimeframeEndDate

	return {
		selectedProjectIds,
		selectedTimeframeMode,
		selectedTimeframe: parsePresetQueryValue(
			query[QUERY_KEY_TIMEFRAME],
			TIMEFRAME_PRESET_VALUES,
			defaultState.selectedTimeframe,
		),
		selectedLastTimeframeAmount: parsePositiveIntegerQueryValue(
			query[QUERY_KEY_TIMEFRAME_LAST_AMOUNT],
			defaultState.selectedLastTimeframeAmount,
		),
		selectedLastTimeframeUnit: parsePresetQueryValue(
			query[QUERY_KEY_TIMEFRAME_LAST_UNIT],
			LAST_TIMEFRAME_UNIT_VALUES,
			defaultState.selectedLastTimeframeUnit,
		),
		selectedCustomTimeframeStartDate,
		selectedCustomTimeframeEndDate,
		selectedGroupBy: parsePresetQueryValue(
			query[QUERY_KEY_GROUP_BY],
			GROUP_BY_PRESET_VALUES,
			defaultState.selectedGroupBy,
		),
		selectedBreakdown: parseAnalyticsBreakdownQueryValue(
			query[QUERY_KEY_BREAKDOWN],
			getDefaultAnalyticsBreakdownPreset(selectedProjectIds),
		),
		selectedFilters,
	}
}

export function hasAnalyticsBreakdownQuery(query: LocationQuery): boolean {
	return parseListQueryValue(query[QUERY_KEY_BREAKDOWN]).length > 0
}

export function hasAnalyticsProjectSelectionQuery(query: LocationQuery): boolean {
	return parseListQueryValue(query[QUERY_KEY_PROJECT_IDS]).length > 0
}

export function buildAnalyticsQueryBuilderRouteQuery(
	currentRouteQuery: LocationQuery,
	state: AnalyticsQueryBuilderState,
	availableProjectIds: string[],
	graphState?: AnalyticsGraphState,
): MutableRouteQuery {
	const nextRouteQuery = {
		...currentRouteQuery,
	} as MutableRouteQuery

	const projectIdsQueryValue = areAllProjectsSelected(state.selectedProjectIds, availableProjectIds)
		? undefined
		: serializeListQueryValue(state.selectedProjectIds)
	const isCustomTimeframeMode =
		state.selectedTimeframeMode === 'custom_range' ||
		state.selectedTimeframeMode === 'custom_datetime_range'

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
	nextRouteQuery[QUERY_KEY_TIMEFRAME_START] = isCustomTimeframeMode
		? state.selectedCustomTimeframeStartDate
		: undefined
	nextRouteQuery[QUERY_KEY_TIMEFRAME_END] = isCustomTimeframeMode
		? state.selectedCustomTimeframeEndDate
		: undefined
	nextRouteQuery[QUERY_KEY_GROUP_BY] =
		state.selectedGroupBy !== DEFAULT_GROUP_BY_PRESET ? state.selectedGroupBy : undefined
	const defaultBreakdown = getDefaultAnalyticsBreakdownPreset(state.selectedProjectIds)
	nextRouteQuery[QUERY_KEY_BREAKDOWN] =
		state.selectedBreakdown !== defaultBreakdown ? state.selectedBreakdown : undefined

	for (const category of URL_FILTER_CATEGORIES) {
		const categoryQueryKey = FILTER_QUERY_KEY_BY_CATEGORY[category]
		nextRouteQuery[categoryQueryKey] = serializeListQueryValue(state.selectedFilters[category])
	}
	nextRouteQuery[QUERY_KEY_FILTER_LEGACY_DOWNLOAD_SOURCE] = undefined

	if (graphState) {
		nextRouteQuery[QUERY_KEY_STAT] =
			graphState.activeStat !== DEFAULT_ANALYTICS_DASHBOARD_STAT ? graphState.activeStat : undefined
		nextRouteQuery[QUERY_KEY_GRAPH_VIEW_MODE] =
			graphState.activeGraphViewMode !== DEFAULT_ANALYTICS_GRAPH_VIEW_MODE
				? graphState.activeGraphViewMode
				: undefined
		nextRouteQuery[QUERY_KEY_GRAPH_RATIO_MODE] = graphState.isRatioMode ? '1' : undefined
		nextRouteQuery[QUERY_KEY_GRAPH_EVENTS_VISIBILITY] = graphState.showChartEvents ? undefined : '0'
		nextRouteQuery[QUERY_KEY_LEGACY_GRAPH_TOP_BREAKDOWN_FILTER] = undefined
		nextRouteQuery[QUERY_KEY_LEGACY_GRAPH_LEGEND_EXPANSION] = undefined
		nextRouteQuery[QUERY_KEY_GRAPH_HIDDEN_SERIES] = serializeListQueryValue(
			[...graphState.hiddenGraphDatasetIds].sort((left, right) => left.localeCompare(right)),
		)
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
