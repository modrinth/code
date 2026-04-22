<template>
	<div
		class="flex flex-col gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4"
	>
		<div v-if="showProjectRow" class="flex items-center gap-2">
			<div class="flex items-center gap-2 text-primary">
				<FolderOpenIcon class="size-4" />
				<span class="text-sm font-medium">Projects:</span>
			</div>
			<div class="w-[20rem]">
				<MultiSelect
					v-model="selectedProjectIds"
					:options="projectOptions"
					placeholder="Select projects"
					:searchable="projectOptions.length > 6"
					:max-tag-rows="1"
					include-select-all-option
					select-all-label="All projects"
				/>
			</div>
		</div>

		<div class="flex items-center gap-2">
			<div class="flex items-center gap-2 text-primary">
				<CalendarIcon class="size-4" />
				<span class="text-sm font-medium">Timeframe:</span>
			</div>
			<div class="flex flex-wrap items-center gap-2">
				<div class="w-[10rem]">
					<Combobox v-model="selectedTimeframe" :options="timeframeOptions" />
				</div>
				<span class="text-sm font-medium text-primary">Grouped by</span>
				<div class="w-[8rem]">
					<Combobox v-model="selectedGroupBy" :options="groupByOptions" />
				</div>
			</div>
		</div>

		<div class="flex items-center gap-2">
			<div class="flex items-center gap-2 text-primary md:pt-2">
				<BlocksIcon class="size-4" />
				<span class="text-sm font-medium">Breakdown:</span>
			</div>
			<div class="flex flex-col gap-2">
				<div class="flex flex-wrap items-center gap-2">
					<div class="w-[11rem]">
						<Combobox v-model="selectedBreakdown" :options="breakdownOptions" />
					</div>
					<span class="text-sm font-medium text-primary">Filtered by</span>
					<Menu :triggers="['click']" :auto-hide="false" no-auto-focus placement="bottom-start">
						<button
							type="button"
							class="inline-flex items-center gap-2 rounded-xl border border-dashed border-surface-5 bg-surface-2 px-3 py-1.5 text-sm font-semibold text-primary transition-colors hover:bg-surface-4"
						>
							<PlusIcon class="size-4" />
							Add
						</button>

						<template #popper>
							<div
								class="flex w-[16rem] flex-col gap-1 rounded-xl border border-solid border-surface-5 bg-surface-4 p-2"
							>
								<Menu
									v-for="category in filterCategories"
									:key="category.key"
									:triggers="['hover', 'focus']"
									:auto-hide="false"
									no-auto-focus
									placement="right-start"
								>
									<button
										type="button"
										class="flex w-full items-center justify-between rounded-lg px-2 py-1.5 text-left text-sm font-medium text-primary transition-colors hover:bg-surface-5"
									>
										<span>{{ category.label }}</span>
										<div class="flex items-center gap-1">
											<span
												v-if="getCategorySelectionCount(category.key) > 0"
												class="rounded-full bg-surface-5 px-1.5 py-0.5 text-xs text-primary"
											>
												{{ getCategorySelectionCount(category.key) }}
											</span>
											<ChevronRightIcon class="size-4 text-primary" />
										</div>
									</button>

									<template #popper>
										<div>
											<p class="mb-2 text-xs font-semibold uppercase tracking-wide text-primary">
												{{ category.label }}
											</p>
											<div class="flex max-h-56 flex-col gap-2 overflow-y-auto">
												<Checkbox
													v-for="option in category.options"
													:key="`${category.key}-${option.value}`"
													:model-value="isFilterValueSelected(category.key, option.value)"
													:label="option.label"
													@update:model-value="
														(nextValue) => toggleFilterValue(category.key, option.value, nextValue)
													"
												/>
											</div>
										</div>
									</template>
								</Menu>
							</div>
						</template>
					</Menu>
				</div>

				<div v-if="activeFilterLabels.length > 0" class="flex flex-wrap items-center gap-2">
					<span
						v-for="filterLabel in activeFilterLabels"
						:key="filterLabel"
						class="inline-flex items-center rounded-full border border-solid border-surface-5 bg-surface-3 px-2.5 py-1 text-xs font-medium text-primary"
					>
						{{ filterLabel }}
					</span>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	BlocksIcon,
	CalendarIcon,
	ChevronRightIcon,
	FolderOpenIcon,
	PlusIcon,
} from '@modrinth/assets'
import {
	Checkbox,
	Combobox,
	type ComboboxOption,
	MultiSelect,
	type MultiSelectOption,
} from '@modrinth/ui'
import { Menu } from 'floating-vue'

import { injectAnalyticsDashboardContext } from '~/providers/analytics'

const ALL_FILTER_VALUE = '__all__'
const MIN_RANGE_MS = 60 * 60 * 1000
const MAX_TIME_SLICES = 1024

export type QueryBuilderProject = {
	id: string
	name: string
}

export type QueryBuilderFilterCategory =
	| 'project'
	| 'country'
	| 'monetization'
	| 'download_source'
	| 'download_type'
	| 'game_version'
	| 'loader_type'

type TimeframePreset =
	| 'today'
	| 'yesterday'
	| 'last_24_hours'
	| 'last_7_days'
	| 'last_14_days'
	| 'last_30_days'
	| 'last_90_days'
	| 'last_190_days'
	| 'this_month'
	| 'year_to_date'
	| 'all_time'

type GroupByPreset = '1h' | '6h' | 'day' | 'week' | 'month' | 'year'

type BreakdownPreset =
	| 'none'
	| 'country'
	| 'monetization'
	| 'download_source'
	| 'download_type'
	| 'loader'
	| 'game_version'

const analyticsDashboardContext = injectAnalyticsDashboardContext()
const projects = computed(() => analyticsDashboardContext.projects.value)

const selectedProjectIds = computed<string[]>({
	get: () => analyticsDashboardContext.selectedProjectIds.value,
	set: (nextProjectIds) => {
		analyticsDashboardContext.selectedProjectIds.value = nextProjectIds
	},
})
const selectedTimeframe = ref<TimeframePreset>('yesterday')
const selectedGroupBy = ref<GroupByPreset>('1h')
const selectedBreakdown = ref<BreakdownPreset>('none')

const selectedFilters = ref<Record<QueryBuilderFilterCategory, string[]>>({
	project: [],
	country: [],
	monetization: [],
	download_source: [],
	download_type: [],
	game_version: [],
	loader_type: [],
})

const projectOptions = computed<MultiSelectOption<string>[]>(() =>
	projects.value.map((project) => ({
		value: project.id,
		label: project.name,
	})),
)

const showProjectRow = computed(() => projects.value.length > 1)

const timeframeOptions: ComboboxOption<TimeframePreset>[] = [
	{ value: 'today', label: 'Today' },
	{ value: 'yesterday', label: 'Yesterday' },
	{ value: 'last_24_hours', label: 'Last 24 hours' },
	{ value: 'last_7_days', label: 'Last 7 days' },
	{ value: 'last_14_days', label: 'Last 14 days' },
	{ value: 'last_30_days', label: 'Last 30 days' },
	{ value: 'last_90_days', label: 'Last 90 days' },
	{ value: 'last_190_days', label: 'Last 190 days' },
	{ value: 'this_month', label: 'This month' },
	{ value: 'year_to_date', label: 'Year to date' },
	{ value: 'all_time', label: 'All time' },
]

const groupByOptions: ComboboxOption<GroupByPreset>[] = [
	{ value: '1h', label: '1h' },
	{ value: '6h', label: '6h' },
	{ value: 'day', label: 'Day' },
	{ value: 'week', label: 'Week' },
	{ value: 'month', label: 'Month' },
	{ value: 'year', label: 'Year' },
]

const breakdownOptions: ComboboxOption<BreakdownPreset>[] = [
	{ value: 'none', label: 'None' },
	{ value: 'country', label: 'Country' },
	{ value: 'monetization', label: 'Monetization' },
	{ value: 'download_source', label: 'Download source' },
	{ value: 'download_type', label: 'Download type' },
	{ value: 'loader', label: 'Loader' },
	{ value: 'game_version', label: 'Game version' },
]

type FilterOption = {
	value: string
	label: string
}

type FilterCategory = {
	key: QueryBuilderFilterCategory
	label: string
	options: FilterOption[]
}

const filterCategories = computed<FilterCategory[]>(() => [
	{
		key: 'project',
		label: 'Project',
		options: [
			{ value: ALL_FILTER_VALUE, label: 'All projects' },
			...projects.value.map((project) => ({ value: project.id, label: project.name })),
		],
	},
	{
		key: 'country',
		label: 'Country',
		options: [{ value: ALL_FILTER_VALUE, label: 'All countries' }],
	},
	{
		key: 'monetization',
		label: 'Monetization',
		options: [
			{ value: ALL_FILTER_VALUE, label: 'All monetization states' },
			{ value: 'monetized', label: 'Monetized' },
			{ value: 'unmonetized', label: 'Unmonetized' },
		],
	},
	{
		key: 'download_source',
		label: 'Download source',
		options: [{ value: ALL_FILTER_VALUE, label: 'All download sources' }],
	},
	{
		key: 'download_type',
		label: 'Download type',
		options: [
			{ value: ALL_FILTER_VALUE, label: 'All download types' },
			{ value: 'modpack', label: 'Modpack' },
			{ value: 'standalone', label: 'Standalone' },
			{ value: 'dependency', label: 'Dependency' },
		],
	},
	{
		key: 'game_version',
		label: 'Game version',
		options: [{ value: ALL_FILTER_VALUE, label: 'All game versions' }],
	},
	{
		key: 'loader_type',
		label: 'Loader type',
		options: [{ value: ALL_FILTER_VALUE, label: 'All loader types' }],
	},
])

const optionLabelByCategoryAndValue = computed(() => {
	const optionLabelMap = new Map<string, string>()

	for (const category of filterCategories.value) {
		for (const option of category.options) {
			optionLabelMap.set(`${category.key}:${option.value}`, option.label)
		}
	}

	return optionLabelMap
})

const activeFilterLabels = computed(() => {
	const labels: string[] = []

	for (const category of filterCategories.value) {
		if (category.key === 'project') {
			continue
		}

		const selectedValues = selectedFilters.value[category.key]
		for (const value of selectedValues) {
			const optionLabel = optionLabelByCategoryAndValue.value.get(`${category.key}:${value}`)
			if (optionLabel) {
				labels.push(`${category.label}: ${optionLabel}`)
			}
		}
	}

	if (selectedProjectIds.value.length !== projects.value.length) {
		for (const selectedProjectId of selectedProjectIds.value) {
			const project = projects.value.find((item) => item.id === selectedProjectId)
			if (project) {
				labels.push(`Project: ${project.name}`)
			}
		}
	}

	return labels
})

function isFilterValueSelected(categoryKey: QueryBuilderFilterCategory, value: string): boolean {
	if (categoryKey === 'project') {
		if (value === ALL_FILTER_VALUE) {
			return selectedProjectIds.value.length === projects.value.length
		}

		return selectedProjectIds.value.includes(value)
	}

	const values = selectedFilters.value[categoryKey]
	if (value === ALL_FILTER_VALUE) {
		return values.length === 0
	}
	return values.includes(value)
}

function toggleFilterValue(
	categoryKey: QueryBuilderFilterCategory,
	value: string,
	nextValue: boolean,
) {
	if (categoryKey === 'project') {
		if (value === ALL_FILTER_VALUE) {
			if (nextValue) {
				selectedProjectIds.value = projects.value.map((project) => project.id)
			}
			return
		}

		if (nextValue) {
			if (!selectedProjectIds.value.includes(value)) {
				selectedProjectIds.value = [...selectedProjectIds.value, value]
			}
			return
		}

		selectedProjectIds.value = selectedProjectIds.value.filter((projectId) => projectId !== value)
		if (selectedProjectIds.value.length === 0) {
			selectedProjectIds.value = projects.value.map((project) => project.id)
		}
		return
	}

	const currentValues = selectedFilters.value[categoryKey]

	if (value === ALL_FILTER_VALUE) {
		if (nextValue) {
			selectedFilters.value[categoryKey] = []
		}
		return
	}

	if (nextValue) {
		if (!currentValues.includes(value)) {
			selectedFilters.value[categoryKey] = [...currentValues, value]
		}
	} else {
		selectedFilters.value[categoryKey] = currentValues.filter((item) => item !== value)
	}
}

function getCategorySelectionCount(categoryKey: QueryBuilderFilterCategory): number {
	if (categoryKey === 'project') {
		return selectedProjectIds.value.length === projects.value.length
			? 0
			: selectedProjectIds.value.length
	}

	return selectedFilters.value[categoryKey].length
}

function startOfDay(date: Date): Date {
	const nextDate = new Date(date)
	nextDate.setHours(0, 0, 0, 0)
	return nextDate
}

function getTimeRangeForPreset(preset: TimeframePreset): { start: Date; end: Date } {
	const now = new Date()
	const end = new Date(now)

	switch (preset) {
		case 'today':
			return { start: startOfDay(now), end }
		case 'yesterday': {
			const todayStart = startOfDay(now)
			return {
				start: new Date(todayStart.getTime() - 24 * 60 * 60 * 1000),
				end: todayStart,
			}
		}
		case 'last_24_hours':
			return {
				start: new Date(end.getTime() - 24 * 60 * 60 * 1000),
				end,
			}
		case 'last_7_days':
			return {
				start: new Date(end.getTime() - 7 * 24 * 60 * 60 * 1000),
				end,
			}
		case 'last_14_days':
			return {
				start: new Date(end.getTime() - 14 * 24 * 60 * 60 * 1000),
				end,
			}
		case 'last_30_days':
			return {
				start: new Date(end.getTime() - 30 * 24 * 60 * 60 * 1000),
				end,
			}
		case 'last_90_days':
			return {
				start: new Date(end.getTime() - 90 * 24 * 60 * 60 * 1000),
				end,
			}
		case 'last_190_days':
			return {
				start: new Date(end.getTime() - 190 * 24 * 60 * 60 * 1000),
				end,
			}
		case 'this_month': {
			const monthStart = new Date(now.getFullYear(), now.getMonth(), 1)
			monthStart.setHours(0, 0, 0, 0)
			return { start: monthStart, end }
		}
		case 'year_to_date': {
			const yearStart = new Date(now.getFullYear(), 0, 1)
			yearStart.setHours(0, 0, 0, 0)
			return { start: yearStart, end }
		}
		case 'all_time':
			return {
				start: new Date(Date.UTC(2022, 0, 1, 0, 0, 0, 0)),
				end,
			}
		default:
			return {
				start: new Date(end.getTime() - 24 * 60 * 60 * 1000),
				end,
			}
	}
}

function getGroupByMinutes(preset: GroupByPreset): number {
	switch (preset) {
		case '1h':
			return 60
		case '6h':
			return 360
		case 'day':
			return 24 * 60
		case 'week':
			return 7 * 24 * 60
		case 'month':
			return 30 * 24 * 60
		case 'year':
			return 365 * 24 * 60
		default:
			return 60
	}
}

function ensureMinimumRange(start: Date, end: Date): { start: Date; end: Date } {
	if (end.getTime() <= start.getTime()) {
		return {
			start: new Date(end.getTime() - MIN_RANGE_MS),
			end,
		}
	}

	if (end.getTime() - start.getTime() < MIN_RANGE_MS) {
		return {
			start: new Date(end.getTime() - MIN_RANGE_MS),
			end,
		}
	}

	return { start, end }
}

function unique<T>(values: T[]): T[] {
	return Array.from(new Set(values))
}

function withBreakdownFields(
	breakdown: BreakdownPreset,
	filters: Record<QueryBuilderFilterCategory, string[]>,
): {
	views: Labrinth.Analytics.v3.ProjectViewsField[]
	downloads: Labrinth.Analytics.v3.ProjectDownloadsField[]
	playtime: Labrinth.Analytics.v3.ProjectPlaytimeField[]
	revenue: Labrinth.Analytics.v3.ProjectRevenueField[]
} {
	const views: Labrinth.Analytics.v3.ProjectViewsField[] = ['project_id']
	const downloads: Labrinth.Analytics.v3.ProjectDownloadsField[] = ['project_id']
	const playtime: Labrinth.Analytics.v3.ProjectPlaytimeField[] = ['project_id']
	const revenue: Labrinth.Analytics.v3.ProjectRevenueField[] = ['project_id']

	switch (breakdown) {
		case 'country':
			views.push('country')
			downloads.push('country')
			break
		case 'monetization':
			views.push('monetized')
			break
		case 'download_source':
			views.push('domain')
			downloads.push('domain')
			break
		case 'download_type':
			downloads.push('version_id')
			playtime.push('version_id')
			break
		case 'loader':
			playtime.push('loader')
			break
		case 'game_version':
			playtime.push('game_version')
			break
		default:
			break
	}

	if (filters.country.length > 0) {
		views.push('country')
		downloads.push('country')
	}

	if (filters.monetization.length > 0) {
		views.push('monetized')
	}

	if (filters.download_source.length > 0) {
		views.push('domain')
		downloads.push('domain')
	}

	if (filters.download_type.length > 0) {
		downloads.push('version_id')
		playtime.push('version_id')
	}

	if (filters.game_version.length > 0) {
		playtime.push('game_version')
	}

	if (filters.loader_type.length > 0) {
		playtime.push('loader')
	}

	return {
		views: unique(views),
		downloads: unique(downloads),
		playtime: unique(playtime),
		revenue: unique(revenue),
	}
}

const fetchRequest = computed<Labrinth.Analytics.v3.FetchRequest>(() => {
	const rawRange = getTimeRangeForPreset(selectedTimeframe.value)
	const { start, end } = ensureMinimumRange(rawRange.start, rawRange.end)

	const groupByMs = getGroupByMinutes(selectedGroupBy.value) * 60 * 1000
	const desiredSlices = Math.max(1, Math.ceil((end.getTime() - start.getTime()) / groupByMs))
	const resolutionSlices = Math.min(MAX_TIME_SLICES, desiredSlices)

	const bucketBy = withBreakdownFields(selectedBreakdown.value, selectedFilters.value)

	return {
		time_range: {
			start: start.toISOString(),
			end: end.toISOString(),
			resolution: {
				slices: resolutionSlices,
			},
		},
		return_metrics: {
			project_views: {
				bucket_by: bucketBy.views,
			},
			project_downloads: {
				bucket_by: bucketBy.downloads,
			},
			project_playtime: {
				bucket_by: bucketBy.playtime,
			},
			project_revenue: {
				bucket_by: bucketBy.revenue,
			},
		},
	}
})

watch(
	fetchRequest,
	(nextFetchRequest) => {
		analyticsDashboardContext.setFetchRequest(nextFetchRequest)
	},
	{ deep: true, immediate: true },
)

defineExpose({
	fetchRequest,
})
</script>
