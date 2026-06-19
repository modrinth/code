<template>
	<div class="md:pl-1">
		<div class="flex flex-wrap items-center gap-2 md:hidden">
			<MultiSelect
				v-if="showProjectRow"
				v-model="draftSelectedProjectIds"
				class="min-w-0 max-w-full"
				:options="projectSelectOptions"
				:disabled="!hasProjectOptions"
				:max-height="QUERY_BUILDER_DROPDOWN_MAX_HEIGHT"
				dropdown-min-width="360px"
				:placeholder="formatMessage(analyticsMessages.selectProjects)"
				:no-options-message="noProjectsMessage"
				:searchable="projectOptions.length > 6"
				:max-tag-rows="1"
				:trigger-class="analyticsQueryChipTriggerClass"
				fit-content
				checkbox-position="right"
				show-selection-actions
				@open="handleProjectSelectOpen"
				@close="handleProjectSelectClose"
			>
				<template #input-content="{ isOpen, openDirection }">
					<div class="flex min-h-7 min-w-0 max-w-full flex-1 items-center gap-1.5 pr-1">
						<img
							v-if="selectedProjectIconUrl"
							:src="selectedProjectIconUrl"
							:alt="formatMessage(analyticsMessages.projectIconAlt, { name: selectedProjectLabel })"
							class="size-5 shrink-0 rounded object-cover"
							loading="lazy"
							decoding="async"
						/>
						<LayersIcon v-else class="size-5 shrink-0 text-primary" />
						<span class="min-w-0 flex-1 truncate px-0.5 font-semibold text-primary">
							{{ selectedProjectLabel }}
						</span>
						<ChevronLeftIcon
							class="size-5 shrink-0 text-primary transition-transform duration-150"
							:class="
								isOpen ? (openDirection === 'down' ? 'rotate-90' : '-rotate-90') : '-rotate-90'
							"
						/>
					</div>
				</template>
				<template #option="{ item, selected }">
					<div class="flex min-w-0 flex-1 items-center gap-2">
						<img
							v-if="getProjectIconUrl(item.value)"
							:src="getProjectIconUrl(item.value)"
							:alt="formatMessage(analyticsMessages.projectIconAlt, { name: item.label })"
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
							v-if="showProjectPresets"
							type="button"
							class="flex w-full cursor-pointer items-center gap-1.5 border-0 bg-surface-4 px-4 py-3 text-left shadow-none transition-all duration-150 hover:brightness-[115%] focus:brightness-[115%]"
							:aria-selected="isUserProjectsOptionSelected"
							:class="isUserProjectsOptionSelected ? 'text-contrast' : 'text-primary'"
							role="option"
							@click="selectUserProjectsMode"
							@keydown.enter.stop
							@keydown.space.stop
						>
							<UserIcon
								class="h-5 w-5 shrink-0 text-primary"
								:class="isUserProjectsOptionSelected ? 'text-contrast' : 'text-primary'"
							/>
							<span class="min-w-0 flex-1 font-semibold leading-tight">
								{{ userProjectsLabel }}
							</span>
							<span class="flex shrink-0 items-center justify-center text-brand">
								<CheckIcon v-if="isUserProjectsOptionSelected" aria-hidden="true" class="size-5" />
							</span>
						</button>
						<button
							v-if="!showProjectPresets || showAllProjectsPreset"
							type="button"
							class="flex w-full cursor-pointer items-center gap-1.5 border-0 bg-surface-4 px-4 py-3 text-left shadow-none transition-all duration-150 hover:brightness-[115%] focus:brightness-[115%]"
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
							<span class="min-w-0 flex-1 font-semibold leading-tight">
								{{ formatMessage(analyticsMessages.allProjects) }}
							</span>
							<span class="flex shrink-0 items-center justify-center text-brand">
								<CheckIcon v-if="isAllProjectsOptionSelected" aria-hidden="true" class="size-5" />
							</span>
						</button>
					</div>
				</template>
				<template v-if="hasProjectOptions" #bottom>
					<DownloadsThresholdInput
						class="border-0 border-t border-solid border-surface-5 px-6 py-2.5"
						:label="formatMessage(analyticsMessages.projectsAbove)"
						:input-aria-label="formatMessage(analyticsMessages.projectDownloadsThresholdAria)"
						:threshold="projectDownloadsThreshold"
						input-width-class="w-20"
						@update:threshold="setProjectDownloadsThreshold"
						@submit="runProjectDownloadsThresholdQuery"
					/>
				</template>
			</MultiSelect>

			<TimeFramePicker
				class="!w-auto min-w-0 max-w-full"
				:trigger-class="analyticsQueryChipTriggerClass"
			>
				<template #prefix>
					<CalendarIcon class="size-5 shrink-0 text-primary" />
				</template>
			</TimeFramePicker>

			<Combobox
				v-model="selectedGroupBy"
				class="!w-auto min-w-0 max-w-full"
				:options="groupByOptions"
				:max-height="QUERY_BUILDER_DROPDOWN_MAX_HEIGHT"
				:dropdown-min-width="QUERY_BUILDER_DROPDOWN_MIN_WIDTH"
				:display-value="selectedGroupByLabel"
				:trigger-class="analyticsQueryChipTriggerClass"
			>
				<template #prefix>
					<ClockIcon class="size-5 shrink-0 text-primary" />
				</template>
			</Combobox>

			<MultiSelect
				v-model="selectedBreakdownValue"
				class="min-w-0 max-w-full"
				:options="breakdownOptions"
				:max-height="QUERY_BUILDER_DROPDOWN_MAX_HEIGHT"
				:dropdown-width="QUERY_BUILDER_DROPDOWN_MIN_WIDTH"
				:dropdown-min-width="QUERY_BUILDER_DROPDOWN_MIN_WIDTH"
				:trigger-class="analyticsQueryChipTriggerClass"
				fit-content
				checkbox-position="right"
				:placeholder="formatMessage(analyticsMessages.none)"
				show-selection-actions
				@open="handleBreakdownSelectOpen"
				@close="handleBreakdownSelectClose"
			>
				<template #input-content="{ isOpen, openDirection }">
					<div class="flex min-h-7 min-w-0 max-w-full flex-1 items-center gap-1.5 pr-1">
						<BlocksIcon class="size-5 shrink-0 text-primary" />
						<span
							class="min-w-0 flex-1 truncate px-0.5 font-semibold text-primary"
							:title="mobileSelectedBreakdownLabel"
						>
							{{ mobileSelectedBreakdownLabel }}
						</span>
						<ChevronLeftIcon
							class="size-5 shrink-0 text-primary transition-transform duration-150"
							:class="
								isOpen ? (openDirection === 'down' ? 'rotate-90' : '-rotate-90') : '-rotate-90'
							"
						/>
					</div>
				</template>
			</MultiSelect>

			<QueryBuilderFilter
				:add-label="formatMessage(analyticsMessages.addFilterButton)"
				:show-label="false"
				show-preview-filter-icon
				:show-clear-action="false"
				:add-button-class="analyticsQueryAddFilterButtonClass"
			/>
		</div>

		<div class="hidden flex-col gap-3 md:flex">
			<div v-if="showProjectRow" class="flex items-start gap-2">
				<div class="my-1.5 flex w-32 items-center gap-2 text-primary">
					<FolderOpenIcon class="size-5" />
					<span class="text-base font-medium">{{
						formatMessage(analyticsMessages.projectLabel)
					}}</span>
				</div>
				<div class="w-fit">
					<MultiSelect
						v-model="draftSelectedProjectIds"
						:options="projectSelectOptions"
						:disabled="!hasProjectOptions"
						:max-height="QUERY_BUILDER_DROPDOWN_MAX_HEIGHT"
						dropdown-min-width="360px"
						:placeholder="formatMessage(analyticsMessages.selectProjects)"
						:no-options-message="noProjectsMessage"
						:searchable="projectOptions.length > 6"
						:max-tag-rows="1"
						checkbox-position="right"
						show-selection-actions
						@open="handleProjectSelectOpen"
						@close="handleProjectSelectClose"
					>
						<template #input-content="{ isOpen, openDirection }">
							<div class="flex min-h-7 min-w-0 flex-1 items-center gap-1.5 pr-1">
								<div class="flex items-center gap-0.5">
									<img
										v-if="selectedProjectIconUrl"
										:src="selectedProjectIconUrl"
										:alt="
											formatMessage(analyticsMessages.projectIconAlt, {
												name: selectedProjectLabel,
											})
										"
										class="size-6 shrink-0 rounded object-cover"
										loading="lazy"
										decoding="async"
									/>
									<LayersIcon
										v-else-if="
											isUserProjectsOptionSelected ||
											isAllProjectsOptionSelected ||
											areAllProjectRowsSelected
										"
										class="size-5 shrink-0 text-primary"
									/>
									<BoxIcon v-else class="size-5 shrink-0 text-primary" />
									<span class="min-w-0 flex-1 truncate px-1.5 font-semibold text-primary">
										{{ selectedProjectLabel }}
									</span>
								</div>
								<ChevronLeftIcon
									class="size-5 shrink-0 text-primary transition-transform duration-150"
									:class="
										isOpen ? (openDirection === 'down' ? 'rotate-90' : '-rotate-90') : '-rotate-90'
									"
								/>
							</div>
						</template>
						<template #option="{ item, selected }">
							<div class="flex min-w-0 flex-1 items-center gap-2">
								<img
									v-if="getProjectIconUrl(item.value)"
									:src="getProjectIconUrl(item.value)"
									:alt="formatMessage(analyticsMessages.projectIconAlt, { name: item.label })"
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
									v-if="showProjectPresets"
									type="button"
									class="flex w-full cursor-pointer items-center gap-2 border-0 bg-surface-4 px-4 py-3 text-left shadow-none transition-all duration-150 hover:brightness-[115%] focus:brightness-[115%]"
									:aria-selected="isUserProjectsOptionSelected"
									:class="isUserProjectsOptionSelected ? 'text-contrast' : 'text-primary'"
									role="option"
									@click="selectUserProjectsMode"
									@keydown.enter.stop
									@keydown.space.stop
								>
									<UserIcon
										class="h-5 w-5 shrink-0 text-primary"
										:class="isUserProjectsOptionSelected ? 'text-contrast' : 'text-primary'"
									/>
									<span class="min-w-0 flex-1 font-semibold leading-tight">
										{{ userProjectsLabel }}
									</span>
									<span class="flex shrink-0 items-center justify-center text-brand">
										<CheckIcon
											v-if="isUserProjectsOptionSelected"
											aria-hidden="true"
											class="size-5"
										/>
									</span>
								</button>
								<button
									v-if="!showProjectPresets || showAllProjectsPreset"
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
									<span class="min-w-0 flex-1 font-semibold leading-tight">
										{{ formatMessage(analyticsMessages.allProjects) }}
									</span>
									<span class="flex shrink-0 items-center justify-center text-brand">
										<CheckIcon
											v-if="isAllProjectsOptionSelected"
											aria-hidden="true"
											class="size-5"
										/>
									</span>
								</button>
							</div>
						</template>
						<template v-if="hasProjectOptions" #bottom>
							<DownloadsThresholdInput
								class="border-0 border-t border-solid border-surface-5 px-6 py-2.5"
								:label="formatMessage(analyticsMessages.projectsAbove)"
								:input-aria-label="formatMessage(analyticsMessages.projectDownloadsThresholdAria)"
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
						<span class="text-base font-medium">
							{{ formatMessage(analyticsMessages.timeframeLabel) }}
						</span>
					</div>
					<div>
						<TimeFramePicker />
					</div>
				</div>
				<div class="flex items-center gap-2">
					<span class="text-base font-medium text-primary">
						{{ formatMessage(analyticsMessages.groupedByLabel) }}
					</span>
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
						{{ formatMessage(analyticsMessages.resetButton) }}
					</button>
				</ButtonStyled>
			</div>

			<div class="flex flex-wrap items-start gap-2">
				<div class="flex items-center gap-2">
					<div class="flex w-32 items-center gap-2 text-primary">
						<BlocksIcon class="size-5" />
						<span class="text-base font-medium">
							{{ formatMessage(analyticsMessages.breakdownLabel) }}
						</span>
					</div>
					<div class="flex flex-col gap-2">
						<div class="flex flex-wrap items-center gap-2">
							<div>
								<MultiSelect
									v-model="selectedBreakdownValue"
									:options="breakdownOptions"
									:max-height="QUERY_BUILDER_DROPDOWN_MAX_HEIGHT"
									:dropdown-width="QUERY_BUILDER_DROPDOWN_MIN_WIDTH"
									:dropdown-min-width="QUERY_BUILDER_DROPDOWN_MIN_WIDTH"
									checkbox-position="right"
									:placeholder="formatMessage(analyticsMessages.none)"
									show-selection-actions
									@open="handleBreakdownSelectOpen"
									@close="handleBreakdownSelectClose"
								>
									<template #input-content="{ isOpen, openDirection }">
										<div class="flex min-h-7 min-w-0 flex-1 items-center gap-1.5 pr-1">
											<span
												class="min-w-0 flex-1 truncate font-semibold text-primary"
												:title="selectedBreakdownLabel"
											>
												{{ selectedBreakdownLabel }}
											</span>
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
									</template>
								</MultiSelect>
							</div>
						</div>
					</div>
				</div>
				<QueryBuilderFilter />
			</div>
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
	ClockIcon,
	FolderOpenIcon,
	LayersIcon,
	UserIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	Combobox,
	type ComboboxOption,
	MultiSelect,
	type MultiSelectItem,
	type MultiSelectOption,
	useVIntl,
} from '@modrinth/ui'

import {
	buildDefaultAnalyticsQueryBuilderState,
	getAnalyticsBreakdownPresetsForProjectSelection,
	MAX_ANALYTICS_BREAKDOWN_PRESETS,
} from '~/components/analytics-dashboard/analytics-route-query'
import {
	type AnalyticsBreakdownPreset,
	type AnalyticsDashboardProject,
	type AnalyticsDashboardStat,
	type AnalyticsGroupByPreset,
	type AnalyticsQueryFilterCategory,
	type AnalyticsSelectedBreakdowns,
	type AnalyticsSelectedFilters,
	getProjectIdsMatchingStatusFilter,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

import {
	analyticsMessages,
	formatAnalyticsBreakdownLabel,
	formatAnalyticsGroupByLabel,
	formatAnalyticsGroupBySelectedLabel,
} from '../analytics-messages.ts'
import DownloadsThresholdInput from './DownloadsThresholdInput.vue'
import {
	getAnalyticsStatsForBreakdown,
	getAnalyticsStatsForFilterCategory,
	getEnabledAnalyticsStatsForState,
} from './query-filter-utils.ts'
import QueryBuilderFilter from './QueryFilter.vue'
import {
	ensureMinimumTimeRange,
	getAnalyticsGroupByPresetMinutes,
	MAX_ANALYTICS_TIME_SLICES,
	useSelectedAnalyticsTimeRange,
} from './timeframe.ts'
import TimeFramePicker from './TimeframePicker.vue'

const QUERY_BUILDER_DROPDOWN_MAX_HEIGHT = 500
const QUERY_BUILDER_DROPDOWN_MIN_WIDTH = '14rem'
const analyticsQueryChipTriggerClass = 'h-10 '
const analyticsQueryAddFilterButtonClass = '!h-10 max-w-full !w-max !px-3.5 flex !gap-2'
const projectOptionCollator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' })
type ProjectSelectionPreset = 'user' | 'all'

const {
	hasProjectContext,
	projectGroups,
	projects,
	dashboardUserProjectIds,
	dashboardOrganizationProjectIds,
	defaultProjectIds,
	isUsingDashboardUserOverride,
	dashboardProjectUserName,
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
	showPreviousPeriod,
	projectStatusById,
	availableProjectDownloadsById,
	queryResetToken,
	refreshAnalyticsQuery,
	setFetchRequest,
} = injectAnalyticsDashboardContext()
const route = useRoute()
const { formatMessage } = useVIntl()
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
const userProjectIds = computed(() =>
	dashboardOrganizationProjectIds.value.length > 0
		? dashboardUserProjectIds.value
		: defaultProjectIds.value,
)
const showProjectPresets = computed(
	() =>
		hasProjectOptions.value &&
		dashboardUserProjectIds.value.length > 0 &&
		dashboardOrganizationProjectIds.value.length > 0,
)
const showAllProjectsPreset = computed(() => dashboardOrganizationProjectIds.value.length > 0)
const noProjectsMessage = computed(() =>
	hasProjectContext.value
		? formatMessage(analyticsMessages.noDataAvailableForAnalytics)
		: formatMessage(analyticsMessages.noProjectsAvailable),
)
const isProjectSelectOpen = ref(false)
const draftProjectSelectionPreset = ref<ProjectSelectionPreset | null>(null)
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

function getProjectSelectionPreset(projectIds: string[]): ProjectSelectionPreset | null {
	if (!showProjectPresets.value) {
		return null
	}

	if (isSameProjectSelection(projectIds, userProjectIds.value)) {
		return 'user'
	}

	if (isSameProjectSelection(projectIds, allProjectIds.value)) {
		return 'all'
	}

	return null
}

function setDraftProjectSelection(projectIds: string[]) {
	const preset = getProjectSelectionPreset(projectIds)
	draftProjectSelectionPreset.value = preset
	if (preset) {
		draftSelectedProjectIds.value = []
		return
	}

	draftSelectedProjectIds.value = isSameProjectSelection(projectIds, allProjectIds.value)
		? []
		: [...projectIds]
}

watch(selectedProjectIds, (nextSelectedProjectIds) => {
	if (isProjectSelectOpen.value) {
		return
	}

	setDraftProjectSelection(nextSelectedProjectIds)
})

watch(draftSelectedProjectIds, (nextSelectedProjectIds) => {
	if (draftProjectSelectionPreset.value && nextSelectedProjectIds.length > 0) {
		draftProjectSelectionPreset.value = null
	}

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
	isBreakdownSelectOpen.value = false
	draftSelectedBreakdowns.value = [...selectedBreakdowns.value]
	clearProjectDownloadsThreshold()
	setDraftProjectSelection(selectedProjectIds.value)
})

const areAllProjectRowsSelected = computed(() => {
	return isSameProjectSelection(draftSelectedProjectIds.value, allProjectIds.value)
})
const isAllProjectsOptionSelected = computed(() =>
	showProjectPresets.value
		? draftProjectSelectionPreset.value === 'all'
		: draftSelectedProjectIds.value.length === 0,
)
const isUserProjectsOptionSelected = computed(() => {
	return showProjectPresets.value && draftProjectSelectionPreset.value === 'user'
})
const userProjectsLabel = computed(() => {
	if (isUsingDashboardUserOverride.value) {
		return formatMessage(analyticsMessages.userProjects, {
			username: dashboardProjectUserName.value,
		})
	}

	return formatMessage(analyticsMessages.yourProjects)
})

const selectedProjectLabel = computed(() => {
	if (!hasProjectOptions.value) {
		return noProjectsMessage.value
	}

	if (isUserProjectsOptionSelected.value) {
		return userProjectsLabel.value
	}

	if (isAllProjectsOptionSelected.value || areAllProjectRowsSelected.value) {
		return formatMessage(analyticsMessages.allProjects)
	}

	if (draftSelectedProjectIds.value.length === 1) {
		const selectedProject = projectOptions.value.find(
			(project) => project.value === draftSelectedProjectIds.value[0],
		)
		return selectedProject?.label ?? formatMessage(analyticsMessages.projectCount, { count: 1 })
	}

	return formatMessage(analyticsMessages.projectCount, {
		count: draftSelectedProjectIds.value.length,
	})
})

const selectedProjectIconUrl = computed(() => {
	if (
		isUserProjectsOptionSelected.value ||
		isAllProjectsOptionSelected.value ||
		areAllProjectRowsSelected.value ||
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
	setDraftProjectSelection(selectedProjectIds.value)
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
	const nextProjectIds =
		draftProjectSelectionPreset.value === 'user'
			? [...userProjectIds.value]
			: draftProjectSelectionPreset.value === 'all'
				? [...allProjectIds.value]
				: normalizeProjectSelection(nextSelectedProjectIds)

	setDraftProjectSelection(nextProjectIds)
	if (!isSameProjectSelection(selectedProjectIds.value, nextProjectIds)) {
		if (isSameProjectSelection(nextProjectIds, allProjectIds.value)) {
			showPreviousPeriod.value = false
		}
		selectedProjectIds.value = nextProjectIds
	}
}

function selectAllProjectsMode() {
	clearProjectDownloadsThreshold()
	if (showProjectPresets.value) {
		draftProjectSelectionPreset.value = 'all'
	} else {
		draftProjectSelectionPreset.value = null
	}
	draftSelectedProjectIds.value = []
}

function selectUserProjectsMode() {
	clearProjectDownloadsThreshold()
	draftProjectSelectionPreset.value = 'user'
	draftSelectedProjectIds.value = []
}

const draftSelectedBreakdowns = ref<AnalyticsSelectedBreakdowns>([...selectedBreakdowns.value])
const isBreakdownSelectOpen = ref(false)

const selectedBreakdownValue = computed<AnalyticsSelectedBreakdowns>({
	get: () => draftSelectedBreakdowns.value,
	set: (nextBreakdowns) => {
		draftSelectedBreakdowns.value = getAnalyticsBreakdownPresetsForProjectSelection(
			nextBreakdowns.slice(0, MAX_ANALYTICS_BREAKDOWN_PRESETS),
			selectedProjectIds.value,
		)
	},
})

watch(selectedBreakdowns, (nextBreakdowns) => {
	if (isBreakdownSelectOpen.value) {
		return
	}
	draftSelectedBreakdowns.value = [...nextBreakdowns]
})

function handleBreakdownSelectOpen() {
	isBreakdownSelectOpen.value = true
	draftSelectedBreakdowns.value = [...selectedBreakdowns.value]
}

function handleBreakdownSelectClose() {
	isBreakdownSelectOpen.value = false
	commitDraftSelectedBreakdowns()
}

function commitDraftSelectedBreakdowns() {
	const nextBreakdowns = [...draftSelectedBreakdowns.value]
	if (!areSelectedBreakdownsEqual(selectedBreakdowns.value, nextBreakdowns)) {
		showPreviousPeriod.value = false
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
		.filter((project) => (availableProjectDownloadsById.value.get(project.id) ?? 0) > threshold)
		.map((project) => project.id)

	draftProjectSelectionPreset.value = null
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
}> = [
	{ value: '1h' },
	{ value: '6h' },
	{ value: 'day' },
	{ value: 'week' },
	{ value: 'month' },
	{ value: 'year' },
]

const selectedBreakdownLabel = computed(() => {
	if (selectedBreakdownValue.value.length === 0) {
		return formatMessage(analyticsMessages.none)
	}

	return selectedBreakdownValue.value
		.map((breakdown) => getBreakdownOptionLabel(breakdown))
		.join(' + ')
})
const selectedGroupByLabel = computed(() => {
	return formatAnalyticsGroupBySelectedLabel(selectedGroupBy.value, formatMessage)
})
const mobileSelectedBreakdownLabel = computed(() => {
	if (selectedBreakdownValue.value.length === 0) {
		return formatMessage(analyticsMessages.noBreakdown)
	}

	return formatMessage(analyticsMessages.breakdownBy, {
		breakdown: selectedBreakdownLabel.value,
	})
})
const breakdownOptions = computed<MultiSelectOption<Exclude<AnalyticsBreakdownPreset, 'none'>>[]>(
	() => {
		const selectedBreakdownSet = new Set(selectedBreakdownValue.value)
		const hasReachedBreakdownLimit =
			selectedBreakdownValue.value.length >= MAX_ANALYTICS_BREAKDOWN_PRESETS
		const options: MultiSelectOption<Exclude<AnalyticsBreakdownPreset, 'none'>>[] = []

		if (selectedProjectIds.value.length > 1) {
			options.push({
				value: 'project',
				label: formatAnalyticsBreakdownLabel('project', formatMessage),
			})
		}

		options.push(
			{
				value: 'country',
				label: formatAnalyticsBreakdownLabel('country', formatMessage),
			},
			{
				value: 'monetization',
				label: formatAnalyticsBreakdownLabel('monetization', formatMessage),
			},
			{
				value: 'user_agent',
				label: formatAnalyticsBreakdownLabel('user_agent', formatMessage),
			},
			{
				value: 'download_reason',
				label: formatAnalyticsBreakdownLabel('download_reason', formatMessage),
			},
			{
				value: 'version_id',
				label: formatAnalyticsBreakdownLabel('version_id', formatMessage),
			},
			{
				value: 'loader',
				label: formatAnalyticsBreakdownLabel('loader', formatMessage),
			},
			{
				value: 'game_version',
				label: formatAnalyticsBreakdownLabel('game_version', formatMessage),
			},
			{
				value: 'dependent_project_download',
				label: formatAnalyticsBreakdownLabel('dependent_project_download', formatMessage),
			},
		)

		return options.map((option) => ({
			...option,
			disabled: hasReachedBreakdownLimit && !selectedBreakdownSet.has(option.value),
		}))
	},
)

function getBreakdownOptionLabel(breakdown: Exclude<AnalyticsBreakdownPreset, 'none'>): string {
	return (
		breakdownOptions.value.find((option) => option.value === breakdown)?.label ??
		formatAnalyticsBreakdownLabel(breakdown, formatMessage)
	)
}

function isRevenueHourlyGroupBy(groupBy: AnalyticsGroupByPreset): boolean {
	return groupBy === '1h' || groupBy === '6h'
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
			label: formatAnalyticsGroupByLabel(option.value, formatMessage),
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
			case 'dependent_project_download':
				if (includesStat(breakdownStats, 'downloads') && includesStat(enabledStats, 'downloads')) {
					downloads.push('dependent_project_id')
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

	return {
		views: unique(views),
		downloads: unique(downloads),
		playtime: unique(playtime),
		revenue: unique(revenue),
	}
}

function getFilterValuesForStat(
	category: AnalyticsQueryFilterCategory,
	stat: AnalyticsDashboardStat,
	enabledStats: readonly AnalyticsDashboardStat[],
	values: string[],
): string[] {
	if (values.length === 0) {
		return []
	}

	const filterStats = getAnalyticsStatsForFilterCategory(category)
	return includesStat(filterStats, stat) && includesStat(enabledStats, stat)
		? sortStrings(values)
		: []
}

function getMonetizationFilterValues(
	stat: AnalyticsDashboardStat,
	enabledStats: readonly AnalyticsDashboardStat[],
	filters: AnalyticsSelectedFilters,
): boolean[] {
	const values = getFilterValuesForStat('monetization', stat, enabledStats, filters.monetization)
	const monetizedValues: boolean[] = []
	if (values.includes('monetized')) {
		monetizedValues.push(true)
	}
	if (values.includes('unmonetized')) {
		monetizedValues.push(false)
	}
	return monetizedValues
}

function getDownloadReasonFilterValues(
	enabledStats: readonly AnalyticsDashboardStat[],
	filters: AnalyticsSelectedFilters,
): Labrinth.Analytics.v3.DownloadReason[] {
	const validReasons = new Set<Labrinth.Analytics.v3.DownloadReason>([
		'standalone',
		'dependency',
		'modpack',
		'update',
	])
	return getFilterValuesForStat(
		'download_reason',
		'downloads',
		enabledStats,
		filters.download_reason,
	).filter((reason): reason is Labrinth.Analytics.v3.DownloadReason =>
		validReasons.has(reason as Labrinth.Analytics.v3.DownloadReason),
	)
}

function buildMetricFilters(
	breakdowns: readonly AnalyticsBreakdownPreset[],
	filters: AnalyticsSelectedFilters,
): {
	views: Labrinth.Analytics.v3.ProjectViewsFilters
	downloads: Labrinth.Analytics.v3.ProjectDownloadsFilters
	playtime: Labrinth.Analytics.v3.ProjectPlaytimeFilters
	revenue: Labrinth.Analytics.v3.ProjectRevenueFilters
} {
	const enabledStats = getEnabledAnalyticsStatsForState(breakdowns, filters)

	return {
		views: {
			country: getFilterValuesForStat('country', 'views', enabledStats, filters.country),
			monetized: getMonetizationFilterValues('views', enabledStats, filters),
		},
		downloads: {
			country: getFilterValuesForStat('country', 'downloads', enabledStats, filters.country),
			monetized: getMonetizationFilterValues('downloads', enabledStats, filters),
			user_agent: getFilterValuesForStat(
				'user_agent',
				'downloads',
				enabledStats,
				filters.user_agent,
			),
			reason: getDownloadReasonFilterValues(enabledStats, filters),
			version_id: getFilterValuesForStat(
				'version_id',
				'downloads',
				enabledStats,
				filters.version_id,
			),
			game_version: getFilterValuesForStat(
				'game_version',
				'downloads',
				enabledStats,
				filters.game_version,
			),
			loader: getFilterValuesForStat('loader_type', 'downloads', enabledStats, filters.loader_type),
			dependent_project_id: getFilterValuesForStat(
				'dependent_project_id',
				'downloads',
				enabledStats,
				filters.dependent_project_id,
			),
		},
		playtime: {
			country: getFilterValuesForStat('country', 'playtime', enabledStats, filters.country),
			version_id: getFilterValuesForStat(
				'version_id',
				'playtime',
				enabledStats,
				filters.version_id,
			),
			game_version: getFilterValuesForStat(
				'game_version',
				'playtime',
				enabledStats,
				filters.game_version,
			),
			loader: getFilterValuesForStat('loader_type', 'playtime', enabledStats, filters.loader_type),
		},
		revenue: {},
	}
}

const fetchRequest = computed<Labrinth.Analytics.v3.FetchRequest>(() => {
	const rawRange = selectedTimeRange.value
	const { start, end } = ensureMinimumTimeRange(rawRange.start, rawRange.end)

	const groupByMs = getAnalyticsGroupByPresetMinutes(selectedGroupBy.value) * 60 * 1000
	const desiredSlices = Math.max(1, Math.floor((end.getTime() - start.getTime()) / groupByMs))
	const resolutionSlices = Math.min(MAX_ANALYTICS_TIME_SLICES, desiredSlices)

	const bucketBy = withBreakdownFields(selectedBreakdowns.value, selectedFilters.value)
	const filterBy = buildMetricFilters(selectedBreakdowns.value, selectedFilters.value)
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
				filter_by: filterBy.views,
			},
			project_downloads: {
				bucket_by: sortStrings(bucketBy.downloads),
				filter_by: filterBy.downloads,
			},
			project_playtime: {
				bucket_by: sortStrings(bucketBy.playtime),
				filter_by: filterBy.playtime,
			},
			project_revenue: {
				bucket_by: sortStrings(bucketBy.revenue),
				filter_by: filterBy.revenue,
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
