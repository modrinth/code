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
export const ADD_MENU_WIDTH = 250
export const DROPDOWN_GAP = 12
export const DROPDOWN_VIEWPORT_MARGIN = 8
export const FILTER_VALUE_CATEGORIES: Exclude<AnalyticsQueryFilterCategory, 'project'>[] = [
	'country',
	'monetization',
	'download_source',
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
	country: ['views', 'downloads'],
	monetization: ['views'],
	download_source: ['downloads'],
	download_reason: [],
	game_version: ['playtime'],
	loader_type: ['playtime'],
}

const ANALYTICS_DIMENSION_BY_BREAKDOWN: Record<
	AnalyticsBreakdownPreset,
	AnalyticsDashboardDimension
> = {
	none: 'project',
	country: 'country',
	monetization: 'monetization',
	download_source: 'download_source',
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
	version_id: 'version_id',
	game_version: 'game_version',
	loader_type: 'loader_type',
}

export type FilterOption = {
	value: string
	label: string
	searchTerms?: string[]
}

export type FilterCategory = {
	key: AnalyticsQueryFilterCategory
	label: string
	options: FilterOption[]
	searchable?: boolean
	searchPlaceholder?: string
}

export type FilterSelectionSource = 'committed' | 'draft'

export type Point = {
	x: number
	y: number
}

type MenuPositionOptions = {
	triggerRect: DOMRect
	dropdownRect: DOMRect
	viewportWidth: number
	viewportHeight: number
}

type SubmenuPositionOptions = {
	buttonRect: DOMRect
	submenuWidth: number
	submenuHeight: number
	viewportWidth: number
	viewportHeight: number
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

export function isFilterValueSelected(
	categoryKey: AnalyticsQueryFilterCategory,
	value: string,
	selectedValues: string[],
	projectCount: number,
): boolean {
	if (categoryKey === 'project') {
		if (value === ALL_FILTER_VALUE) {
			return selectedValues.length === projectCount
		}

		return selectedValues.includes(value)
	}

	if (value === ALL_FILTER_VALUE) {
		return selectedValues.length === 0
	}
	return selectedValues.includes(value)
}

export function getCategorySelectionCount(
	categoryKey: AnalyticsQueryFilterCategory,
	selectedValues: string[],
	projectCount: number,
): number {
	if (categoryKey === 'project') {
		return selectedValues.length === projectCount ? 0 : selectedValues.length
	}

	return selectedValues.length
}

export function getCategorySelectionSummary(
	category: FilterCategory,
	selectedValues: string[],
	count: number,
	projectIds: string[],
): string {
	if (count === 0) {
		return ''
	}

	if (count === 1) {
		const selectedValue =
			category.key === 'project'
				? selectedValues.find((projectId) => projectIds.includes(projectId))
				: selectedValues[0]
		return category.options.find((option) => option.value === selectedValue)?.label ?? '1 selected'
	}

	return `${count} selected`
}

export function getAddMenuPosition({
	triggerRect,
	dropdownRect,
	viewportWidth,
	viewportHeight,
}: MenuPositionOptions) {
	const dropdownWidth = Math.max(ADD_MENU_WIDTH, triggerRect.width)
	const hasSpaceBelow =
		triggerRect.bottom + dropdownRect.height + DROPDOWN_GAP + DROPDOWN_VIEWPORT_MARGIN <=
		viewportHeight
	const hasSpaceAbove =
		triggerRect.top - dropdownRect.height - DROPDOWN_GAP - DROPDOWN_VIEWPORT_MARGIN > 0
	const opensUp = !hasSpaceBelow && hasSpaceAbove
	const top = opensUp
		? triggerRect.top - dropdownRect.height - DROPDOWN_GAP
		: triggerRect.bottom + DROPDOWN_GAP
	const left = Math.min(triggerRect.left, viewportWidth - dropdownWidth - DROPDOWN_VIEWPORT_MARGIN)

	return {
		left: `${Math.max(DROPDOWN_VIEWPORT_MARGIN, left)}px`,
		minWidth: `${triggerRect.width}px`,
		top: `${Math.max(DROPDOWN_VIEWPORT_MARGIN, top)}px`,
		width: `${dropdownWidth}px`,
	}
}

export function getSubmenuPosition({
	buttonRect,
	submenuWidth,
	submenuHeight,
	viewportWidth,
	viewportHeight,
}: SubmenuPositionOptions): Point {
	const gap = 20
	const viewportPadding = 8
	const preferredLeft = buttonRect.right + gap
	const left =
		preferredLeft + submenuWidth + viewportPadding <= viewportWidth
			? preferredLeft
			: Math.max(viewportPadding, buttonRect.left - submenuWidth - gap)
	const top = Math.min(
		Math.max(viewportPadding, buttonRect.top),
		Math.max(viewportPadding, viewportHeight - submenuHeight - viewportPadding),
	)

	return {
		x: left,
		y: top,
	}
}

export function isCursorAimingAtSubmenu(
	cursor: Point | null,
	origin: Point | null,
	submenuRect: DOMRect | null,
): boolean {
	if (!submenuRect || !cursor || !origin) {
		return false
	}

	const submenuTargetX =
		origin.x <= submenuRect.left
			? submenuRect.left
			: origin.x >= submenuRect.right
				? submenuRect.right
				: cursor.x <= submenuRect.left
					? submenuRect.left
					: submenuRect.right
	const upperTarget: Point = {
		x: submenuTargetX,
		y: submenuRect.top + 20,
	}
	const lowerTarget: Point = {
		x: submenuTargetX,
		y: submenuRect.bottom + 20,
	}

	return isPointInTriangle(cursor, origin, upperTarget, lowerTarget)
}

function isPointInTriangle(point: Point, a: Point, b: Point, c: Point): boolean {
	const area = triangleArea(a, b, c)
	const area1 = triangleArea(point, b, c)
	const area2 = triangleArea(a, point, c)
	const area3 = triangleArea(a, b, point)

	return Math.abs(area - (area1 + area2 + area3)) < 0.5
}

function triangleArea(a: Point, b: Point, c: Point): number {
	return Math.abs((a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y)) / 2)
}
