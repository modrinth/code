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
					dropdown-min-width="360px"
					placeholder="Select projects"
					:no-options-message="noProjectsMessage"
					:searchable="projectOptions.length > 6"
					:max-tag-rows="1"
					checkbox-position="right"
					show-selection-actions
					@open="handleProjectSelectOpen"
					@close="handleProjectSelectClose"
				>
					<template #input-content="{ isOpen, openDirection }">
						<div class="flex min-h-7 min-w-0 flex-1 items-center gap-2 pr-1">
							<div class="flex items-center gap-0.5">
								<img
									v-if="selectedProjectIconUrl"
									:src="selectedProjectIconUrl"
									:alt="`${selectedProjectLabel} Icon`"
									class="size-6 shrink-0 rounded object-cover"
									loading="lazy"
									decoding="async"
								/>
								<LayersIcon
									v-else-if="isAllProjectsOptionSelected || areAllProjectsSelected"
									class="size-6 shrink-0 text-primary"
								/>
								<BoxIcon v-else class="size-6 shrink-0 text-primary" />
								<span class="min-w-0 flex-1 truncate px-1.5 font-semibold text-primary">
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
					<template #option="{ item, selected }">
						<div class="flex min-w-0 flex-1 items-center gap-2">
							<img
								v-if="getProjectIconUrl(item.value)"
								:src="getProjectIconUrl(item.value)"
								:alt="`${item.label} Icon`"
								class="h-5 w-5 shrink-0 rounded object-cover"
								loading="lazy"
								decoding="async"
							/>
							<BoxIcon
								v-else
								class="h-5 w-5 shrink-0 text-primary"
								:class="selected ? 'text-contrast' : 'text-primary'"
							/>
							<span
								v-tooltip="item.label"
								class="min-w-0 truncate font-semibold leading-tight"
								:class="selected ? 'text-contrast' : 'text-primary'"
							>
								{{ item.label }}
							</span>
						</div>
					</template>
					<template v-if="hasProjectOptions" #top>
						<div>
							<button
								type="button"
								class="flex w-full cursor-pointer items-center gap-2 border-0 bg-surface-4 px-4 py-3 text-left shadow-none transition-all duration-150 hover:brightness-[115%] focus:brightness-[115%]"
								:aria-selected="isAllProjectsOptionSelected"
								:class="isAllProjectsOptionSelected ? 'text-contrast' : 'text-primary'"
								role="option"
								@click="selectAllProjectsMode"
								@keydown.enter.stop
								@keydown.space.stop
							>
								<LayersIcon
									class="h-5 w-5 shrink-0 text-primary"
									:class="isAllProjectsOptionSelected ? 'text-contrast' : 'text-primary'"
								/>
								<span class="min-w-0 flex-1 font-semibold leading-tight"> All projects </span>
								<span class="flex shrink-0 items-center justify-center text-brand">
									<CheckIcon v-if="isAllProjectsOptionSelected" aria-hidden="true" class="size-5" />
								</span>
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
			<ButtonStyled v-if="!isTimeframeAndGroupByDefault" type="transparent">
				<button
					type="button"
					:disabled="isTimeframeAndGroupByDefault"
					@click="resetTimeframeAndGroupBy"
				>
					Reset
				</button>
			</ButtonStyled>
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
							<MultiSelect
								v-model="selectedBreakdownValue"
								:options="breakdownOptions"
								:max-height="QUERY_BUILDER_DROPDOWN_MAX_HEIGHT"
								:dropdown-min-width="QUERY_BUILDER_DROPDOWN_MIN_WIDTH"
								:max-tag-rows="1"
								placeholder="No breakdown"
								checkbox-position="right"
								clearable
							>
								<template #input-content="{ isOpen, openDirection }">
									<div class="flex min-h-7 min-w-0 flex-1 items-center gap-2 pr-1">
										<span
											class="min-w-0 flex-1 truncate px-1.5 font-semibold text-primary"
											:title="selectedBreakdownLabel"
										>
											{{ selectedBreakdownLabel }}
										</span>
										<div class="flex shrink-0 items-center gap-1.5">
											<template v-if="canClearSelectedBreakdowns">
												<button
													type="button"
													class="flex cursor-pointer items-center justify-center rounded border-none bg-transparent p-0.5 text-secondary transition-colors hover:text-contrast"
													aria-label="Clear breakdowns"
													@click.stop="clearSelectedBreakdowns"
												>
													<XIcon class="size-4 text-primary" />
												</button>
												<div class="h-5 w-[1px] shrink-0 bg-surface-5"></div>
											</template>

											<ChevronLeftIcon
												class="size-5 shrink-0 text-primary transition-transform duration-150"
												:class="
													isOpen
														? openDirection === 'down'
															? 'rotate-90'
															: '-rotate-90'
														: '-rotate-90'
												"
											/>
										</div>
									</div>
								</template>
							</MultiSelect>
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
	LayersIcon,
	XIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	Combobox,
	type ComboboxOption,
	MultiSelect,
	type MultiSelectItem,
	type MultiSelectOption,
} from '@modrinth/ui'

import {
	type AnalyticsBreakdownPreset,
	type AnalyticsDashboardProject,
	type AnalyticsGroupByPreset,
	type AnalyticsSelectedBreakdowns,
	type AnalyticsSelectedFilters,
	getProjectIdsMatchingStatusFilter,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'
import {
	buildDefaultAnalyticsQueryBuilderState,
	getAnalyticsBreakdownPresetsForProjectSelection,
	MAX_ANALYTICS_BREAKDOWN_PRESETS,
} from '~/providers/analytics/query-builder-url'

import DownloadsThresholdInput from './DownloadsThresholdInput.vue'
import {
	getAnalyticsStatsForBreakdown,
	getAnalyticsStatsForFilterCategory,
	getEnabledAnalyticsStatsForState,
} from './query-filter/queryFilter'
import QueryBuilderFilter from './query-filter/QueryFilter.vue'
import {
	ensureMinimumTimeRange,
	getAnalyticsGroupByPresetMinutes,
	MAX_ANALYTICS_TIME_SLICES,
	useSelectedAnalyticsTimeRange,
} from './timeframe-picker/timeframe'
import TimeFramePicker from './timeframe-picker/TimeFramePicker.vue'

const QUERY_BUILDER_DROPDOWN_MAX_HEIGHT = 500
const QUERY_BUILDER_DROPDOWN_MIN_WIDTH = '12rem'
const projectOptionCollator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' })

const {
	hasProjectContext,
	projectGroups,
	projects,
	selectedProjectIds,
	selectedTimeframeMode,
	selectedTimeframe,
	selectedLastTimeframeAmount,
	selectedLastTimeframeUnit,
	selectedCustomTimeframeStartDate,
	selectedCustomTimeframeEndDate,
	selectedGroupBy,
	selectedBreakdowns,
	selectedFilters,
	activeStat,
	projectStatusById,
	queryResetToken,
	refreshAnalyticsQuery,
	setFetchRequest,
} = injectAnalyticsDashboardContext()
const route = useRoute()
const { selectedTimeRange, selectedTimeframeDurationMinutes } = useSelectedAnalyticsTimeRange()
const defaultQueryState = buildDefaultAnalyticsQueryBuilderState([])

function getProjectOption(
	project: AnalyticsDashboardProject,
	groupTitle?: string,
): MultiSelectOption<string> {
	return {
		value: project.id,
		label: project.name,
		searchTerms: groupTitle ? [groupTitle] : undefined,
	}
}

function compareProjectOptions(
	left: MultiSelectOption<string>,
	right: MultiSelectOption<string>,
): number {
	return projectOptionCollator.compare(left.label, right.label)
}

const projectOptions = computed<MultiSelectOption<string>[]>(() =>
	projects.value.map((project) => getProjectOption(project)),
)
const projectIconUrlsById = computed(
	() =>
		new Map(
			projects.value
				.filter((project) => project.iconUrl)
				.map((project) => [project.id, project.iconUrl as string]),
		),
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

		options.push(
			...group.projects
				.map((project) => getProjectOption(project, group.title))
				.sort(compareProjectOptions),
		)
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
	selectedBreakdownCommitRequestId++
	draftSelectedBreakdowns.value = [...selectedBreakdowns.value]
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

const selectedProjectIconUrl = computed(() => {
	if (
		isAllProjectsOptionSelected.value ||
		areAllProjectsSelected.value ||
		draftSelectedProjectIds.value.length !== 1
	) {
		return undefined
	}

	return getProjectIconUrl(draftSelectedProjectIds.value[0])
})

function getProjectIconUrl(projectId: string): string | undefined {
	return projectIconUrlsById.value.get(projectId)
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

const draftSelectedBreakdowns = ref<AnalyticsSelectedBreakdowns>([...selectedBreakdowns.value])
let selectedBreakdownCommitRequestId = 0

const selectedBreakdownValue = computed<AnalyticsSelectedBreakdowns>({
	get: () => draftSelectedBreakdowns.value,
	set: (nextBreakdowns) => {
		draftSelectedBreakdowns.value = getAnalyticsBreakdownPresetsForProjectSelection(
			nextBreakdowns.slice(0, MAX_ANALYTICS_BREAKDOWN_PRESETS),
			selectedProjectIds.value,
		)
		void scheduleSelectedBreakdownCommit()
	},
})

watch(selectedBreakdowns, (nextBreakdowns) => {
	selectedBreakdownCommitRequestId++
	draftSelectedBreakdowns.value = [...nextBreakdowns]
})

async function scheduleSelectedBreakdownCommit() {
	const requestId = ++selectedBreakdownCommitRequestId
	const nextBreakdowns = [...draftSelectedBreakdowns.value]

	await waitForDeferredQueryBuilderCommit()

	if (requestId !== selectedBreakdownCommitRequestId) {
		return
	}

	if (!areSelectedBreakdownsEqual(selectedBreakdowns.value, nextBreakdowns)) {
		selectedBreakdowns.value = nextBreakdowns
	}
}

function areSelectedBreakdownsEqual(
	left: readonly AnalyticsBreakdownPreset[],
	right: readonly AnalyticsBreakdownPreset[],
) {
	if (left.length !== right.length) return false
	for (let index = 0; index < left.length; index += 1) {
		if (left[index] !== right[index]) return false
	}
	return true
}

function waitForDeferredQueryBuilderCommit(): Promise<void> {
	if (!import.meta.client) {
		return nextTick()
	}

	return new Promise((resolve) => {
		nextTick(() => {
			requestAnimationFrame(() => {
				requestAnimationFrame(() => resolve())
			})
		})
	})
}

const isDashboardAnalyticsRoute = computed(
	() => route.path.replace(/\/$/, '') === '/dashboard/analytics',
)
const showProjectRow = computed(() => isDashboardAnalyticsRoute.value || projects.value.length > 1)
const isTimeframeAndGroupByDefault = computed(
	() =>
		selectedTimeframeMode.value === defaultQueryState.selectedTimeframeMode &&
		selectedTimeframe.value === defaultQueryState.selectedTimeframe &&
		selectedGroupBy.value === defaultQueryState.selectedGroupBy,
)

function resetTimeframeAndGroupBy() {
	if (isTimeframeAndGroupByDefault.value) {
		return
	}

	selectedTimeframeMode.value = defaultQueryState.selectedTimeframeMode
	selectedTimeframe.value = defaultQueryState.selectedTimeframe
	selectedLastTimeframeAmount.value = defaultQueryState.selectedLastTimeframeAmount
	selectedLastTimeframeUnit.value = defaultQueryState.selectedLastTimeframeUnit
	selectedCustomTimeframeStartDate.value = defaultQueryState.selectedCustomTimeframeStartDate
	selectedCustomTimeframeEndDate.value = defaultQueryState.selectedCustomTimeframeEndDate
	selectedGroupBy.value = defaultQueryState.selectedGroupBy
}

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
}> = [
	{ value: '1h', label: '1h' },
	{ value: '6h', label: '6h' },
	{ value: 'day', label: 'Day' },
	{ value: 'week', label: 'Week' },
	{ value: 'month', label: 'Month' },
	{ value: 'year', label: 'Year' },
]

const selectedProjectCount = computed(() => selectedProjectIds.value.length)
const selectedBreakdownLabel = computed(() => {
	if (selectedBreakdownValue.value.length === 0) {
		return 'No breakdown'
	}

	return selectedBreakdownValue.value
		.map((breakdown) => getBreakdownOptionLabel(breakdown))
		.join(' + ')
})
const canClearSelectedBreakdowns = computed(() => selectedBreakdownValue.value.length > 0)
const breakdownOptions = computed<MultiSelectOption<Exclude<AnalyticsBreakdownPreset, 'none'>>[]>(
	() => {
		const selectedBreakdownSet = new Set(selectedBreakdownValue.value)
		const hasReachedBreakdownLimit =
			selectedBreakdownValue.value.length >= MAX_ANALYTICS_BREAKDOWN_PRESETS
		const options: MultiSelectOption<Exclude<AnalyticsBreakdownPreset, 'none'>>[] = [
			...(selectedProjectCount.value > 1 ? [{ value: 'project' as const, label: 'Project' }] : []),
			{ value: 'country', label: 'Country' },
			{ value: 'monetization', label: 'Monetization' },
			{ value: 'user_agent', label: 'Download source' },
			{ value: 'download_reason', label: 'Download reason' },
			{ value: 'version_id', label: 'Project version' },
			{ value: 'loader', label: 'Loader' },
			{ value: 'game_version', label: 'Game version' },
		]

		return options.map((option) => ({
			...option,
			disabled: hasReachedBreakdownLimit && !selectedBreakdownSet.has(option.value),
		}))
	},
)

function getBreakdownOptionLabel(breakdown: Exclude<AnalyticsBreakdownPreset, 'none'>): string {
	return breakdownOptions.value.find((option) => option.value === breakdown)?.label ?? breakdown
}

function clearSelectedBreakdowns() {
	selectedBreakdownValue.value = []
}

function isRevenueHourlyGroupBy(groupBy: AnalyticsGroupByPreset): boolean {
	return groupBy === '1h' || groupBy === '6h'
}

function getAllTimeYearGroupStart(end: Date): Date {
	const start = new Date(end)
	start.setFullYear(2021)
	return start
}

const groupByOptions = computed<ComboboxOption<AnalyticsGroupByPreset>[]>(() => {
	const timeframeMinutes = selectedTimeframeDurationMinutes.value
	const options = groupByPresetOptions.map((option) => {
		const groupByMinutes = getAnalyticsGroupByPresetMinutes(option.value)
		const isTooCoarse = groupByMinutes >= timeframeMinutes
		const isTooFine = timeframeMinutes / groupByMinutes > MAX_ANALYTICS_TIME_SLICES
		const isRevenueHourlyGroupByOption =
			activeStat.value === 'revenue' && isRevenueHourlyGroupBy(option.value)
		const isRevenueDailyFallback = activeStat.value === 'revenue' && option.value === 'day'
		return {
			value: option.value,
			label: option.label,
			disabled:
				isRevenueHourlyGroupByOption || (!isRevenueDailyFallback && (isTooCoarse || isTooFine)),
		}
	})

	if (options.every((option) => option.disabled)) {
		const fallbackOption =
			options.find((option) => activeStat.value === 'revenue' && option.value === 'day') ??
			options[0]
		if (fallbackOption) {
			fallbackOption.disabled = false
		}
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
	breakdowns: readonly AnalyticsBreakdownPreset[],
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
	const enabledStats = getEnabledAnalyticsStatsForState(breakdowns, filters)

	for (const breakdown of breakdowns) {
		const breakdownStats = getAnalyticsStatsForBreakdown(breakdown)

		switch (breakdown) {
			case 'project':
			case 'none':
				break
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
			case 'user_agent':
				if (includesStat(breakdownStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
					downloads.push('user_agent')
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

	if (filters.user_agent.length > 0) {
		const filterStats = getAnalyticsStatsForFilterCategory('user_agent')
		if (includesStat(filterStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
			downloads.push('user_agent')
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

	const groupByMs = getAnalyticsGroupByPresetMinutes(selectedGroupBy.value) * 60 * 1000
	const desiredSlices = Math.max(1, Math.floor((end.getTime() - start.getTime()) / groupByMs))
	const resolutionSlices = Math.min(MAX_ANALYTICS_TIME_SLICES, desiredSlices)

	const bucketBy = withBreakdownFields(selectedBreakdowns.value, selectedFilters.value)
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
