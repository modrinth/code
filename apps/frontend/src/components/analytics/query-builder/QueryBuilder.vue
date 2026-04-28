<template>
	<div
		class="flex flex-col gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4"
	>
		<div v-if="showProjectRow" class="flex items-start gap-2">
			<div class="my-1.5 flex w-32 items-center gap-2 text-primary">
				<FolderOpenIcon class="size-5" />
				<span class="text-base font-medium">Projects:</span>
			</div>
			<div class="grow">
				<MultiSelect
					v-model="selectedProjectIds"
					:options="projectOptions"
					:max-height="QUERY_BUILDER_DROPDOWN_MAX_HEIGHT"
					placeholder="Select projects"
					:searchable="projectOptions.length > 6"
					:max-tag-rows="1"
					include-select-all-option
					select-all-label="All projects"
				/>
			</div>
		</div>

		<div class="flex flex-wrap items-center gap-x-6 gap-y-4">
			<div class="flex items-center gap-2">
				<div class="flex w-32 items-center gap-2 text-primary">
					<CalendarIcon class="size-5" />
					<span class="text-base font-medium">Timeframe:</span>
				</div>
				<div class="w-48">
					<Combobox
						v-model="selectedTimeframe"
						:options="timeframeOptions"
						:max-height="QUERY_BUILDER_DROPDOWN_MAX_HEIGHT"
					/>
				</div>
			</div>
			<div class="flex items-center gap-2">
				<span class="text-base font-medium text-primary">Grouped by</span>
				<div class="w-48">
					<Combobox
						v-model="selectedGroupBy"
						:options="groupByOptions"
						:max-height="QUERY_BUILDER_DROPDOWN_MAX_HEIGHT"
					/>
				</div>
			</div>
		</div>

		<div class="flex flex-wrap items-center gap-x-6 gap-y-4">
			<div class="flex items-center gap-2">
				<div class="flex w-32 items-center gap-2 text-primary">
					<BlocksIcon class="size-5" />
					<span class="text-base font-medium">Breakdown:</span>
				</div>
				<div class="flex flex-col gap-2">
					<div class="flex flex-wrap items-center gap-2">
						<div class="w-48">
							<Combobox
								v-model="selectedBreakdown"
								:options="breakdownOptions"
								:max-height="QUERY_BUILDER_DROPDOWN_MAX_HEIGHT"
							/>
						</div>
					</div>
				</div>
			</div>
			<div class="flex items-center gap-2">
				<QueryBuilderFilter />
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { BlocksIcon, CalendarIcon, FolderOpenIcon } from '@modrinth/assets'
import { Combobox, type ComboboxOption, MultiSelect, type MultiSelectOption } from '@modrinth/ui'

import {
	type AnalyticsBreakdownPreset,
	type AnalyticsGroupByPreset,
	type AnalyticsSelectedFilters,
	type AnalyticsTimeframePreset,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

import QueryBuilderFilter from './QueryFilter.vue'

const MIN_RANGE_MS = 60 * 60 * 1000
const MAX_TIME_SLICES = 1024
const QUERY_BUILDER_DROPDOWN_MAX_HEIGHT = 500
const TIME_RANGE_ROUNDING_MS = 60 * 1000

const {
	projects,
	selectedProjectIds,
	selectedTimeframe,
	selectedGroupBy,
	selectedBreakdown,
	selectedFilters,
	setFetchRequest,
} = injectAnalyticsDashboardContext()

const projectOptions = computed<MultiSelectOption<string>[]>(() =>
	projects.value.map((project) => ({
		value: project.id,
		label: project.name,
	})),
)

const showProjectRow = computed(() => projects.value.length > 1)

const timeframeOptions: ComboboxOption<AnalyticsTimeframePreset>[] = [
	{ value: 'today', label: 'Today' },
	{ value: 'yesterday', label: 'Yesterday' },
	{ value: 'last_7_days', label: 'Last 7 days' },
	{ value: 'last_14_days', label: 'Last 14 days' },
	{ value: 'last_30_days', label: 'Last 30 days' },
	{ value: 'last_90_days', label: 'Last 90 days' },
	{ value: 'last_180_days', label: 'Last 180 days' },
	{ value: 'year_to_date', label: 'Year to date' },
	{ value: 'all_time', label: 'All time' },
]

const groupByPresetOptions: Array<{
	value: AnalyticsGroupByPreset
	label: string
	minutes: number
}> = [
	{ value: '1h', label: '1h', minutes: 60 },
	{ value: '6h', label: '6h', minutes: 360 },
	{ value: 'day', label: 'Day', minutes: 24 * 60 },
	{ value: 'week', label: 'Week', minutes: 7 * 24 * 60 },
	{ value: 'month', label: 'Month', minutes: 30 * 24 * 60 },
	{ value: 'year', label: 'Year', minutes: 365 * 24 * 60 },
]

const breakdownOptions: ComboboxOption<AnalyticsBreakdownPreset>[] = [
	{ value: 'none', label: 'None' },
	{ value: 'country', label: 'Country' },
	{ value: 'monetization', label: 'Monetization' },
	{ value: 'download_source', label: 'Download source' },
	{ value: 'download_type', label: 'Download type' },
	{ value: 'loader', label: 'Loader' },
	{ value: 'game_version', label: 'Game version' },
]

function startOfDay(date: Date): Date {
	const nextDate = new Date(date)
	nextDate.setHours(0, 0, 0, 0)
	return nextDate
}

function getRoundedNow(): Date {
	const timestamp = Math.floor(Date.now() / TIME_RANGE_ROUNDING_MS) * TIME_RANGE_ROUNDING_MS
	return new Date(timestamp)
}

function getTimeRangeForPreset(preset: AnalyticsTimeframePreset): { start: Date; end: Date } {
	const now = getRoundedNow()
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
		case 'last_180_days':
			return {
				start: new Date(end.getTime() - 180 * 24 * 60 * 60 * 1000),
				end,
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

function getGroupByMinutes(preset: AnalyticsGroupByPreset): number {
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

const selectedTimeframeDurationMinutes = computed(() => {
	const rawRange = getTimeRangeForPreset(selectedTimeframe.value)
	const { start, end } = ensureMinimumRange(rawRange.start, rawRange.end)
	const durationMs = end.getTime() - start.getTime()
	return Math.max(1, Math.floor(durationMs / (60 * 1000)))
})

const groupByOptions = computed<ComboboxOption<AnalyticsGroupByPreset>[]>(() => {
	const options = groupByPresetOptions.map((option) => ({
		value: option.value,
		label: option.label,
		disabled: option.minutes >= selectedTimeframeDurationMinutes.value,
	}))

	if (options.every((option) => option.disabled)) {
		options[0].disabled = false
	}

	return options
})

watch(
	groupByOptions,
	(nextOptions) => {
		const selectedOption = nextOptions.find((option) => option.value === selectedGroupBy.value)
		if (selectedOption && !selectedOption.disabled) {
			return
		}

		const fallbackOption =
			[...nextOptions].reverse().find((option) => !option.disabled) ?? nextOptions[0]
		if (fallbackOption && selectedGroupBy.value !== fallbackOption.value) {
			selectedGroupBy.value = fallbackOption.value
		}
	},
	{ immediate: true },
)

function unique<T>(values: T[]): T[] {
	return Array.from(new Set(values))
}

function sortStrings<T extends string>(values: T[]): T[] {
	return [...values].sort((left, right) => left.localeCompare(right))
}

function withBreakdownFields(
	breakdown: AnalyticsBreakdownPreset,
	filters: AnalyticsSelectedFilters,
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
		project_ids: sortStrings(selectedProjectIds.value),
		return_metrics: {
			project_views: {
				bucket_by: sortStrings(bucketBy.views),
			},
			project_downloads: {
				bucket_by: sortStrings(bucketBy.downloads),
			},
			project_playtime: {
				bucket_by: sortStrings(bucketBy.playtime),
			},
			project_revenue: {
				bucket_by: sortStrings(bucketBy.revenue),
			},
		},
	}
})

watch(
	fetchRequest,
	(nextFetchRequest) => {
		setFetchRequest(nextFetchRequest)
	},
	{ deep: true, immediate: true },
)

defineExpose({
	fetchRequest,
})
</script>
