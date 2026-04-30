<template>
	<div class="flex flex-col gap-3 pl-1">
		<div v-if="showProjectRow" class="flex items-start gap-2">
			<div class="my-1.5 flex w-32 items-center gap-2 text-primary">
				<FolderOpenIcon class="size-5" />
				<span class="text-base font-medium">Projects:</span>
			</div>
			<div class="w-fit">
				<MultiSelect
					v-model="draftSelectedProjectIds"
					:options="projectOptions"
					:max-height="QUERY_BUILDER_DROPDOWN_MAX_HEIGHT"
					dropdown-min-width="24rem"
					placeholder="Select projects"
					:searchable="projectOptions.length > 6"
					:max-tag-rows="1"
					include-select-all-option
					select-all-label="All projects"
					@open="handleProjectSelectOpen"
					@close="handleProjectSelectClose"
				>
					<template #input-content="{ isOpen, openDirection }">
						<div class="flex min-h-8 min-w-0 flex-1 items-center gap-2">
							<span class="min-w-0 flex-1 truncate px-1.5 py-1 font-medium text-primary">
								{{ selectedProjectLabel }}
							</span>
							<div class="ml-2 flex shrink-0 items-center gap-1.5">
								<button
									v-if="isOpen && draftSelectedProjectIds.length > 0"
									type="button"
									class="flex cursor-pointer items-center justify-center rounded border-none bg-transparent p-0.5 text-secondary transition-colors hover:text-contrast"
									aria-label="Clear projects"
									@click.stop="clearDraftSelectedProjects"
								>
									<XIcon class="size-5" />
								</button>
								<div class="h-5 w-[1px] shrink-0 bg-surface-5"></div>
								<ChevronLeftIcon
									class="size-5 shrink-0 text-secondary transition-transform duration-150"
									:class="
										isOpen ? (openDirection === 'down' ? 'rotate-90' : '-rotate-90') : '-rotate-90'
									"
								/>
							</div>
						</div>
					</template>
					<template #bottom>
						<DownloadsThresholdInput
							class="border-0 border-t border-solid border-surface-5 px-3 py-2.5"
							label="Projects above"
							input-aria-label="Project downloads threshold"
							:threshold="projectDownloadsThreshold"
							input-width-class="w-20"
							@update:threshold="setProjectDownloadsThreshold"
						/>
					</template>
				</MultiSelect>
			</div>
		</div>

		<div class="flex flex-wrap items-center gap-4">
			<div class="flex items-center gap-2">
				<div class="flex w-32 items-center gap-2 text-primary">
					<CalendarIcon class="size-5" />
					<span class="text-base font-medium">Timeframe:</span>
				</div>
				<div>
					<TimeFramePicker />
				</div>
			</div>
			<div class="flex items-center gap-2">
				<span class="text-base font-medium text-primary">Grouped by</span>
				<div>
					<Combobox
						v-model="selectedGroupBy"
						:options="groupByOptions"
						:max-height="QUERY_BUILDER_DROPDOWN_MAX_HEIGHT"
						:dropdown-min-width="QUERY_BUILDER_DROPDOWN_MIN_WIDTH"
					/>
				</div>
			</div>
		</div>

		<div class="flex flex-wrap items-center gap-4">
			<div class="flex items-center gap-2">
				<div class="flex w-32 items-center gap-2 text-primary">
					<BlocksIcon class="size-5" />
					<span class="text-base font-medium">Breakdown:</span>
				</div>
				<div class="flex flex-col gap-2">
					<div class="flex flex-wrap items-center gap-2">
						<div>
							<Combobox
								v-model="selectedBreakdown"
								:options="breakdownOptions"
								:max-height="QUERY_BUILDER_DROPDOWN_MAX_HEIGHT"
								:dropdown-min-width="QUERY_BUILDER_DROPDOWN_MIN_WIDTH"
							>
								<template #suffix>
									<button
										v-if="selectedBreakdown !== 'none'"
										type="button"
										class="-mr-0.5 inline-flex size-5 shrink-0 items-center justify-center rounded-full border-0 bg-transparent shadow-none transition-colors hover:bg-transparent hover:text-contrast"
										aria-label="Clear breakdown"
										@click.stop="clearSelectedBreakdown"
										@keydown.stop
									>
										<XIcon class="size-4" />
									</button>
								</template>
							</Combobox>
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
import { BlocksIcon, CalendarIcon, ChevronLeftIcon, FolderOpenIcon, XIcon } from '@modrinth/assets'
import { Combobox, type ComboboxOption, MultiSelect, type MultiSelectOption } from '@modrinth/ui'

import {
	type AnalyticsBreakdownPreset,
	type AnalyticsGroupByPreset,
	type AnalyticsSelectedFilters,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

import DownloadsThresholdInput from './DownloadsThresholdInput.vue'
import {
	getAnalyticsStatsForBreakdown,
	getAnalyticsStatsForFilterCategory,
	getEnabledAnalyticsStatsForState,
} from './query-filter/queryFilter'
import QueryBuilderFilter from './query-filter/QueryFilter.vue'
import { ensureMinimumTimeRange, useSelectedAnalyticsTimeRange } from './timeframe-picker/timeframe'
import TimeFramePicker from './timeframe-picker/TimeFramePicker.vue'

const MAX_TIME_SLICES = 1024
const QUERY_BUILDER_DROPDOWN_MAX_HEIGHT = 500
const QUERY_BUILDER_DROPDOWN_MIN_WIDTH = '12rem'

const {
	projects,
	selectedProjectIds,
	selectedGroupBy,
	selectedBreakdown,
	selectedFilters,
	setFetchRequest,
} = injectAnalyticsDashboardContext()
const { selectedTimeRange, selectedTimeframeDurationMinutes } = useSelectedAnalyticsTimeRange()

const projectOptions = computed<MultiSelectOption<string>[]>(() =>
	projects.value.map((project) => ({
		value: project.id,
		label: project.name,
	})),
)

const allProjectIds = computed(() => projectOptions.value.map((project) => project.value))
const isProjectSelectOpen = ref(false)
const draftSelectedProjectIds = ref<string[]>([...selectedProjectIds.value])
const projectDownloadsThreshold = ref<number | null>(null)

function isSameProjectSelection(left: string[], right: string[]) {
	if (left.length !== right.length) {
		return false
	}

	const rightProjectIds = new Set(right)
	return left.every((projectId) => rightProjectIds.has(projectId))
}

function normalizeProjectSelection(projectIds: string[]) {
	return projectIds.length > 0 ? [...projectIds] : [...allProjectIds.value]
}

watch(selectedProjectIds, (nextSelectedProjectIds) => {
	if (isProjectSelectOpen.value) {
		return
	}

	draftSelectedProjectIds.value = [...nextSelectedProjectIds]
})

const areAllProjectsSelected = computed(() => {
	return isSameProjectSelection(draftSelectedProjectIds.value, allProjectIds.value)
})

const selectedProjectLabel = computed(() => {
	if (areAllProjectsSelected.value) {
		return 'All projects'
	}

	if (draftSelectedProjectIds.value.length === 0) {
		return 'Select projects'
	}

	if (draftSelectedProjectIds.value.length === 1) {
		const selectedProject = projectOptions.value.find(
			(project) => project.value === draftSelectedProjectIds.value[0],
		)
		return selectedProject?.label ?? '1 project'
	}

	return `${draftSelectedProjectIds.value.length} projects`
})

function handleProjectSelectOpen() {
	isProjectSelectOpen.value = true
	draftSelectedProjectIds.value = [...selectedProjectIds.value]
}

function handleProjectSelectClose(
	nextSelectedProjectIds: string[] = draftSelectedProjectIds.value,
) {
	isProjectSelectOpen.value = false
	const nextProjectIds = normalizeProjectSelection(nextSelectedProjectIds)

	draftSelectedProjectIds.value = [...nextProjectIds]
	if (!isSameProjectSelection(selectedProjectIds.value, nextProjectIds)) {
		selectedProjectIds.value = nextProjectIds
	}
}

function clearDraftSelectedProjects() {
	draftSelectedProjectIds.value = []
}

function clearSelectedBreakdown() {
	selectedBreakdown.value = 'none'
}

const showProjectRow = computed(() => projects.value.length > 1)

function applyProjectDownloadsThreshold(threshold: number | null) {
	if (threshold === null) {
		return
	}

	draftSelectedProjectIds.value = projects.value
		.filter((project) => project.downloads >= threshold)
		.map((project) => project.id)
}

function setProjectDownloadsThreshold(threshold: number | null) {
	projectDownloadsThreshold.value = threshold
	applyProjectDownloadsThreshold(threshold)
}

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
	{ value: 'version_id', label: 'Project versions' },
	{ value: 'loader', label: 'Loader' },
	{ value: 'game_version', label: 'Game version' },
]

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

function includesStat(stats: readonly string[], stat: string): boolean {
	return stats.includes(stat)
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
	const breakdownStats = getAnalyticsStatsForBreakdown(breakdown)
	const enabledStats = getEnabledAnalyticsStatsForState(breakdown, filters)

	switch (breakdown) {
		case 'country':
			if (includesStat(breakdownStats, 'views') && includesStat(enabledStats, 'views')) {
				views.push('country')
			}
			if (includesStat(breakdownStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
				downloads.push('country')
			}
			break
		case 'monetization':
			if (includesStat(breakdownStats, 'views') && includesStat(enabledStats, 'views')) {
				views.push('monetized')
			}
			break
		case 'download_source':
			if (includesStat(breakdownStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
				downloads.push('domain')
			}
			break
		case 'version_id':
			if (includesStat(breakdownStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
				downloads.push('version_id')
			}
			if (includesStat(breakdownStats, 'playtime') && includesStat(enabledStats, 'playtime')) {
				playtime.push('version_id')
			}
			break
		case 'loader':
			if (includesStat(breakdownStats, 'playtime') && includesStat(enabledStats, 'playtime')) {
				playtime.push('loader')
			}
			break
		case 'game_version':
			if (includesStat(breakdownStats, 'playtime') && includesStat(enabledStats, 'playtime')) {
				playtime.push('game_version')
			}
			break
		default:
			break
	}

	if (filters.country.length > 0) {
		const filterStats = getAnalyticsStatsForFilterCategory('country')
		if (includesStat(filterStats, 'views') && includesStat(enabledStats, 'views')) {
			views.push('country')
		}
		if (includesStat(filterStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
			downloads.push('country')
		}
	}

	if (filters.monetization.length > 0) {
		const filterStats = getAnalyticsStatsForFilterCategory('monetization')
		if (includesStat(filterStats, 'views') && includesStat(enabledStats, 'views')) {
			views.push('monetized')
		}
	}

	if (filters.download_source.length > 0) {
		const filterStats = getAnalyticsStatsForFilterCategory('download_source')
		if (includesStat(filterStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
			downloads.push('domain')
		}
	}

	if (filters.version_id.length > 0) {
		const filterStats = getAnalyticsStatsForFilterCategory('version_id')
		if (includesStat(filterStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
			downloads.push('version_id')
		}
		if (includesStat(filterStats, 'playtime') && includesStat(enabledStats, 'playtime')) {
			playtime.push('version_id')
		}
	}

	if (filters.game_version.length > 0) {
		const filterStats = getAnalyticsStatsForFilterCategory('game_version')
		if (includesStat(filterStats, 'playtime') && includesStat(enabledStats, 'playtime')) {
			playtime.push('game_version')
		}
	}

	if (filters.loader_type.length > 0) {
		const filterStats = getAnalyticsStatsForFilterCategory('loader_type')
		if (includesStat(filterStats, 'playtime') && includesStat(enabledStats, 'playtime')) {
			playtime.push('loader')
		}
	}

	return {
		views: unique(views),
		downloads: unique(downloads),
		playtime: unique(playtime),
		revenue: unique(revenue),
	}
}

const fetchRequest = computed<Labrinth.Analytics.v3.FetchRequest>(() => {
	const rawRange = selectedTimeRange.value
	const { start, end } = ensureMinimumTimeRange(rawRange.start, rawRange.end)

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
