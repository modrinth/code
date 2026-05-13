<template>
	<div class="flex flex-col gap-3 pl-1">
		<div v-if="showProjectRow" class="flex items-start gap-2">
			<div class="my-1.5 flex w-32 items-center gap-2 text-primary">
				<FolderOpenIcon class="size-5" />
				<span class="text-base font-medium">Project:</span>
			</div>
			<div class="w-fit">
				<MultiSelect
					v-model="draftSelectedProjectIds"
					:options="projectSelectOptions"
					:disabled="!hasProjectOptions"
					:max-height="QUERY_BUILDER_DROPDOWN_MAX_HEIGHT"
					dropdown-min-width="330px"
					placeholder="Select projects"
					:no-options-message="noProjectsMessage"
					:searchable="projectOptions.length > 6"
					:max-tag-rows="1"
					show-selection-actions
					@open="handleProjectSelectOpen"
					@close="handleProjectSelectClose"
				>
					<template #input-content="{ isOpen, openDirection }">
						<div class="flex min-h-8 min-w-0 flex-1 items-center gap-2 pr-1">
							<div class="flex items-center gap-0.5">
								<component
									:is="selectedProjectIcon"
									v-if="selectedProjectIcon"
									class="size-6 shrink-0 items-center text-primary"
								/>
								<span class="min-w-0 flex-1 truncate px-1.5 py-1 font-semibold text-primary">
									{{ selectedProjectLabel }}
								</span>
							</div>
							<div class="flex shrink-0 items-center gap-1.5">
								<template v-if="canClearDraftSelectedProjects">
									<button
										type="button"
										class="flex cursor-pointer items-center justify-center rounded border-none bg-transparent p-0.5 text-secondary transition-colors hover:text-contrast"
										aria-label="Clear projects"
										@click.stop="clearDraftSelectedProjects"
									>
										<XIcon class="size-4 text-primary" />
									</button>
									<div class="h-5 w-[1px] shrink-0 bg-surface-5"></div>
								</template>

								<ChevronLeftIcon
									class="size-5 shrink-0 text-primary transition-transform duration-150"
									:class="
										isOpen ? (openDirection === 'down' ? 'rotate-90' : '-rotate-90') : '-rotate-90'
									"
								/>
							</div>
						</div>
					</template>
					<template v-if="hasProjectOptions" #top>
						<div class="px-3">
							<button
								type="button"
								class="flex w-full cursor-pointer items-center gap-2.5 rounded-xl border-0 bg-transparent p-3 text-left text-contrast shadow-none transition-colors duration-150 hover:bg-surface-5 focus:bg-surface-5"
								:aria-selected="isAllProjectsOptionSelected"
								role="option"
								@click="selectAllProjectsMode"
								@keydown.enter.stop
								@keydown.space.stop
							>
								<span
									class="checkbox-shadow flex h-5 w-5 shrink-0 items-center justify-center rounded-md border-[1px] border-solid"
									:class="
										isAllProjectsOptionSelected
											? 'border-button-border bg-brand text-brand-inverted'
											: 'border-surface-5 bg-surface-2'
									"
								>
									<CheckIcon
										v-if="isAllProjectsOptionSelected"
										aria-hidden="true"
										stroke-width="3"
									/>
								</span>
								<span class="font-semibold leading-tight text-primary">All projects</span>
							</button>
						</div>
					</template>
					<template v-if="hasProjectOptions" #bottom>
						<DownloadsThresholdInput
							class="border-0 border-t border-solid border-surface-5 px-6 py-2.5"
							label="Projects above"
							input-aria-label="Project downloads threshold"
							:threshold="projectDownloadsThreshold"
							input-width-class="w-20"
							@update:threshold="setProjectDownloadsThreshold"
							@submit="runProjectDownloadsThresholdQuery"
						/>
					</template>
				</MultiSelect>
			</div>
		</div>

		<div class="flex flex-wrap items-center gap-2">
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

		<div class="flex flex-wrap items-start gap-2">
			<div class="flex items-center gap-2">
				<div class="flex w-32 items-center gap-2 text-primary">
					<BlocksIcon class="size-5" />
					<span class="text-base font-medium">Breakdown:</span>
				</div>
				<div class="flex flex-col gap-2">
					<div class="flex flex-wrap items-center gap-2">
						<div>
							<Combobox
								v-model="selectedBreakdownValue"
								:options="breakdownOptions"
								:max-height="QUERY_BUILDER_DROPDOWN_MAX_HEIGHT"
								:dropdown-min-width="QUERY_BUILDER_DROPDOWN_MIN_WIDTH"
							>
								<template #suffix>
									<div class="mr-0.5 flex gap-1.5">
										<button
											v-if="selectedBreakdownValue !== 'none'"
											type="button"
											class="inline-flex size-5 shrink-0 items-center justify-center rounded-full border-0 bg-transparent shadow-none transition-colors hover:bg-transparent hover:text-contrast"
											aria-label="Clear breakdown"
											@click.stop="clearSelectedBreakdown"
											@keydown.stop
										>
											<XIcon class="size-4 text-primary" />
										</button>
										<div
											v-if="selectedBreakdownValue !== 'none'"
											class="h-5 w-[1px] shrink-0 bg-surface-5"
										></div>
									</div>
								</template>
							</Combobox>
						</div>
					</div>
				</div>
			</div>
			<QueryBuilderFilter />
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	BlocksIcon,
	BoxIcon,
	CalendarIcon,
	CheckIcon,
	ChevronLeftIcon,
	FolderOpenIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Combobox,
	type ComboboxOption,
	MultiSelect,
	type MultiSelectItem,
	type MultiSelectOption,
} from '@modrinth/ui'
import { defineAsyncComponent, h, markRaw } from 'vue'

import {
	type AnalyticsBreakdownPreset,
	type AnalyticsDashboardProject,
	type AnalyticsGroupByPreset,
	type AnalyticsSelectedFilters,
	getProjectIdsMatchingStatusFilter,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

import DownloadsThresholdInput from './DownloadsThresholdInput.vue'
import {
	cloneSelectedFilters,
	getAnalyticsFilterCategoryForBreakdown,
	getAnalyticsStatsForBreakdown,
	getAnalyticsStatsForFilterCategory,
	getEnabledAnalyticsStatsForState,
} from './query-filter/queryFilter'
import QueryBuilderFilter from './query-filter/QueryFilter.vue'
import { ensureMinimumTimeRange, useSelectedAnalyticsTimeRange } from './timeframe-picker/timeframe'
import TimeFramePicker from './timeframe-picker/TimeFramePicker.vue'

const MAX_TIME_SLICES = 256
const QUERY_BUILDER_DROPDOWN_MAX_HEIGHT = 500
const QUERY_BUILDER_DROPDOWN_MIN_WIDTH = '12rem'

const {
	hasProjectContext,
	projectGroups,
	projects,
	selectedProjectIds,
	selectedTimeframeMode,
	selectedTimeframe,
	selectedGroupBy,
	selectedBreakdown,
	selectedFilters,
	projectStatusById,
	queryResetToken,
	refreshAnalyticsQuery,
	setFetchRequest,
} = injectAnalyticsDashboardContext()
const route = useRoute()
const { selectedTimeRange, selectedTimeframeDurationMinutes } = useSelectedAnalyticsTimeRange()

function getProjectOption(
	project: AnalyticsDashboardProject,
	groupTitle?: string,
): MultiSelectOption<string> {
	return {
		value: project.id,
		label: project.name,
		icon: getProjectIcon(project),
		searchTerms: groupTitle ? [groupTitle] : undefined,
	}
}

const projectOptions = computed<MultiSelectOption<string>[]>(() =>
	projects.value.map((project) => getProjectOption(project)),
)

const projectSelectOptions = computed<MultiSelectItem<string>[]>(() => {
	const options: MultiSelectItem<string>[] = []

	for (const group of projectGroups.value) {
		if (group.projects.length === 0) {
			continue
		}

		if (group.title) {
			options.push({
				type: 'section-header',
				label: group.title,
				key: group.key ?? `organization-${group.title}`,
			})
		}

		options.push(...group.projects.map((project) => getProjectOption(project, group.title)))
	}

	return options
})

const allProjectIds = computed(() => projectOptions.value.map((project) => project.value))
const hasProjectOptions = computed(() => projectOptions.value.length > 0)
const noProjectsMessage = computed(() =>
	hasProjectContext.value ? 'No data available for analytics' : 'No projects available',
)
const isProjectSelectOpen = ref(false)
const draftSelectedProjectIds = ref<string[]>([...selectedProjectIds.value])
const projectDownloadsThreshold = ref<number | null>(null)
const projectDownloadsThresholdProjectIds = ref<string[] | null>(null)

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

watch(draftSelectedProjectIds, (nextSelectedProjectIds) => {
	if (projectDownloadsThreshold.value === null) {
		return
	}

	const normalizedProjectIds = normalizeProjectSelection(nextSelectedProjectIds)
	if (
		projectDownloadsThresholdProjectIds.value &&
		isSameProjectSelection(normalizedProjectIds, projectDownloadsThresholdProjectIds.value)
	) {
		return
	}

	clearProjectDownloadsThreshold()
})

watch(queryResetToken, () => {
	isProjectSelectOpen.value = false
	clearProjectDownloadsThreshold()
	draftSelectedProjectIds.value = isSameProjectSelection(
		selectedProjectIds.value,
		allProjectIds.value,
	)
		? []
		: [...selectedProjectIds.value]
})

const areAllProjectsSelected = computed(() => {
	return isSameProjectSelection(draftSelectedProjectIds.value, allProjectIds.value)
})
const isAllProjectsOptionSelected = computed(() => draftSelectedProjectIds.value.length === 0)
const canClearDraftSelectedProjects = computed(() => {
	return !isAllProjectsOptionSelected.value && !areAllProjectsSelected.value
})

const selectedProjectLabel = computed(() => {
	if (!hasProjectOptions.value) {
		return noProjectsMessage.value
	}

	if (isAllProjectsOptionSelected.value || areAllProjectsSelected.value) {
		return 'All projects'
	}

	if (draftSelectedProjectIds.value.length === 1) {
		const selectedProject = projectOptions.value.find(
			(project) => project.value === draftSelectedProjectIds.value[0],
		)
		return selectedProject?.label ?? '1 project'
	}

	return `${draftSelectedProjectIds.value.length} projects`
})

const selectedProjectIcon = computed(() => {
	if (
		isAllProjectsOptionSelected.value ||
		areAllProjectsSelected.value ||
		draftSelectedProjectIds.value.length !== 1
	) {
		return undefined
	}

	return projectOptions.value.find((project) => project.value === draftSelectedProjectIds.value[0])
		?.icon
})

function getProjectIcon(project: AnalyticsDashboardProject) {
	const iconUrl = project.iconUrl
	const projectName = project.name
	if (!iconUrl) {
		return markRaw({
			inheritAttrs: false,
			setup: () => () =>
				h('div', { class: 'h-6 w-6 text-primary' }, [h(BoxIcon, { class: 'h-full w-full' })]),
		})
	}

	return markRaw(
		defineAsyncComponent(() =>
			Promise.resolve({
				setup: () => () =>
					h('img', {
						src: iconUrl,
						alt: `${projectName} Icon`,
						class: 'h-6 w-6 rounded object-cover',
					}),
			}),
		),
	)
}

function handleProjectSelectOpen() {
	isProjectSelectOpen.value = true
	draftSelectedProjectIds.value = isSameProjectSelection(
		selectedProjectIds.value,
		allProjectIds.value,
	)
		? []
		: [...selectedProjectIds.value]
}

function handleProjectSelectClose(
	nextSelectedProjectIds: string[] = draftSelectedProjectIds.value,
) {
	isProjectSelectOpen.value = false
	commitDraftSelectedProjects(nextSelectedProjectIds)
}

function commitDraftSelectedProjects(
	nextSelectedProjectIds: string[] = draftSelectedProjectIds.value,
) {
	const nextProjectIds = normalizeProjectSelection(nextSelectedProjectIds)

	draftSelectedProjectIds.value = [...nextProjectIds]
	if (!isSameProjectSelection(selectedProjectIds.value, nextProjectIds)) {
		selectedProjectIds.value = nextProjectIds
	}
}

function clearDraftSelectedProjects() {
	clearProjectDownloadsThreshold()
	draftSelectedProjectIds.value = []
	if (!isProjectSelectOpen.value) {
		handleProjectSelectClose()
	}
}

function selectAllProjectsMode() {
	clearProjectDownloadsThreshold()
	draftSelectedProjectIds.value = []
}

const selectedBreakdownValue = computed<AnalyticsBreakdownPreset>({
	get: () => selectedBreakdown.value,
	set: (nextBreakdown) => {
		selectedBreakdown.value = nextBreakdown

		const filterCategory = getAnalyticsFilterCategoryForBreakdown(nextBreakdown)
		if (!filterCategory || selectedFilters.value[filterCategory].length === 0) {
			return
		}

		const nextFilters = cloneSelectedFilters(selectedFilters.value)
		nextFilters[filterCategory] = []
		selectedFilters.value = nextFilters
	},
})

function clearSelectedBreakdown() {
	selectedBreakdown.value = 'none'
}

const isDashboardAnalyticsRoute = computed(
	() => route.path.replace(/\/$/, '') === '/dashboard/analytics',
)
const showProjectRow = computed(() => isDashboardAnalyticsRoute.value || projects.value.length > 1)

function applyProjectDownloadsThreshold(threshold: number | null) {
	if (threshold === null) {
		return
	}

	const projectIds = projects.value
		.filter((project) => project.downloads >= threshold)
		.map((project) => project.id)

	projectDownloadsThresholdProjectIds.value = projectIds
	draftSelectedProjectIds.value = projectIds
}

function setProjectDownloadsThreshold(threshold: number | null) {
	projectDownloadsThreshold.value = threshold
	if (threshold === null) {
		projectDownloadsThresholdProjectIds.value = null
		draftSelectedProjectIds.value = []
		return
	}

	applyProjectDownloadsThreshold(threshold)
}

function clearProjectDownloadsThreshold() {
	projectDownloadsThreshold.value = null
	projectDownloadsThresholdProjectIds.value = null
}

function closeProjectSelectDropdown(event: KeyboardEvent) {
	const eventTarget = event.target
	if (!(eventTarget instanceof HTMLElement)) {
		isProjectSelectOpen.value = false
		return
	}

	const dropdown = eventTarget.closest('[role="listbox"][aria-multiselectable="true"]')
	if (!dropdown) {
		isProjectSelectOpen.value = false
		return
	}

	dropdown.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
}

async function runProjectDownloadsThresholdQuery(event: KeyboardEvent) {
	commitDraftSelectedProjects()
	closeProjectSelectDropdown(event)
	await nextTick()
	await refreshAnalyticsQuery()
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
	{ value: 'download_reason', label: 'Download type' },
	{ value: 'version_id', label: 'Project version' },
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

function getAllTimeYearGroupStart(end: Date): Date {
	const start = new Date(end)
	start.setFullYear(2021)
	return start
}

const groupByOptions = computed<ComboboxOption<AnalyticsGroupByPreset>[]>(() => {
	const timeframeMinutes = selectedTimeframeDurationMinutes.value
	const options = groupByPresetOptions.map((option) => {
		const isTooCoarse = option.minutes >= timeframeMinutes
		const isTooFine = timeframeMinutes / option.minutes > MAX_TIME_SLICES
		return {
			value: option.value,
			label: option.label,
			disabled: isTooCoarse || isTooFine,
		}
	})

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

		const fallbackOption = nextOptions.find((option) => !option.disabled) ?? nextOptions[0]
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
			if (includesStat(breakdownStats, 'playtime') && includesStat(enabledStats, 'playtime')) {
				playtime.push('country')
			}
			break
		case 'monetization':
			if (includesStat(breakdownStats, 'views') && includesStat(enabledStats, 'views')) {
				views.push('monetized')
			}
			if (includesStat(breakdownStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
				downloads.push('monetized')
			}
			break
		case 'download_source':
			if (includesStat(breakdownStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
				downloads.push('domain')
			}
			break
		case 'download_reason':
			if (includesStat(breakdownStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
				downloads.push('reason')
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
			if (includesStat(breakdownStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
				downloads.push('loader')
			}
			if (includesStat(breakdownStats, 'playtime') && includesStat(enabledStats, 'playtime')) {
				playtime.push('loader')
			}
			break
		case 'game_version':
			if (includesStat(breakdownStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
				downloads.push('game_version')
			}
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
		if (includesStat(filterStats, 'playtime') && includesStat(enabledStats, 'playtime')) {
			playtime.push('country')
		}
	}

	if (filters.monetization.length > 0) {
		const filterStats = getAnalyticsStatsForFilterCategory('monetization')
		if (includesStat(filterStats, 'views') && includesStat(enabledStats, 'views')) {
			views.push('monetized')
		}
		if (includesStat(filterStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
			downloads.push('monetized')
		}
	}

	if (filters.download_source.length > 0) {
		const filterStats = getAnalyticsStatsForFilterCategory('download_source')
		if (includesStat(filterStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
			downloads.push('domain')
		}
	}

	if (filters.download_reason.length > 0) {
		const filterStats = getAnalyticsStatsForFilterCategory('download_reason')
		if (includesStat(filterStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
			downloads.push('reason')
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
		if (includesStat(filterStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
			downloads.push('game_version')
		}
		if (includesStat(filterStats, 'playtime') && includesStat(enabledStats, 'playtime')) {
			playtime.push('game_version')
		}
	}

	if (filters.loader_type.length > 0) {
		const filterStats = getAnalyticsStatsForFilterCategory('loader_type')
		if (includesStat(filterStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
			downloads.push('loader')
		}
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
	const rawStart =
		selectedTimeframeMode.value === 'preset' &&
		selectedTimeframe.value === 'all_time' &&
		selectedGroupBy.value === 'year'
			? getAllTimeYearGroupStart(rawRange.end)
			: rawRange.start
	const { start, end } = ensureMinimumTimeRange(rawStart, rawRange.end)

	const groupByMs = getGroupByMinutes(selectedGroupBy.value) * 60 * 1000
	const desiredSlices = Math.max(1, Math.floor((end.getTime() - start.getTime()) / groupByMs))
	const resolutionSlices = Math.min(MAX_TIME_SLICES, desiredSlices)

	const bucketBy = withBreakdownFields(selectedBreakdown.value, selectedFilters.value)
	const filteredProjectIds = getProjectIdsMatchingStatusFilter(
		selectedProjectIds.value,
		projectStatusById.value,
		selectedFilters.value,
	)

	return {
		time_range: {
			start: start.toISOString(),
			end: end.toISOString(),
			resolution: {
				slices: resolutionSlices,
			},
		},
		project_ids: sortStrings(filteredProjectIds),
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

<style lang="scss" scoped>
.checkbox-shadow {
	box-shadow: 1px 1px 2px 0 rgba(0, 0, 0, 0.08);
}
</style>
