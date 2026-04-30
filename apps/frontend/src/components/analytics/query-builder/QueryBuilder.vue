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
						<div
							class="flex items-center gap-3 border-0 border-t border-solid border-surface-5 px-3 py-2.5"
						>
							<span class="shrink-0 text-sm font-semibold text-primary">Projects above</span>
							<input
								v-model="projectDownloadsThresholdInput"
								type="text"
								inputmode="numeric"
								placeholder="0"
								class="h-8 w-20 rounded-lg border border-solid border-surface-5 bg-surface-3 px-2 text-center text-sm font-semibold text-primary outline-none transition-[box-shadow,color] focus:text-contrast focus:ring-4 focus:ring-brand-shadow"
								aria-label="Project downloads threshold"
								@blur="formatProjectDownloadsThresholdInput"
							/>
							<span class="shrink-0 text-sm font-semibold text-primary">downloads</span>
						</div>
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
					<Combobox
						v-model="draftSelectedTimeframe"
						:options="timeframeDropdownOptions"
						:display-value="selectedTimeframeLabel"
						:max-height="QUERY_BUILDER_DROPDOWN_MAX_HEIGHT"
						:dropdown-min-width="'20rem'"
						@open="handleTimeframeSelectOpen"
						@close="handleTimeframeSelectClose"
						@select="handleTimeframePresetSelect"
					>
						<template #bottom>
							<div
								class="flex flex-col border-0 border-t border-solid border-surface-5 bg-surface-4"
							>
								<template v-if="draftSelectedTimeframeMode === 'custom_range'">
									<CustomRangeTimeframe
										v-model:start-date="draftSelectedCustomTimeframeStartDate"
										v-model:end-date="draftSelectedCustomTimeframeEndDate"
									/>
									<button
										type="button"
										class="flex cursor-pointer items-center border-0 border-t border-solid border-surface-5 bg-transparent px-4 py-3 text-left text-sm font-semibold text-primary transition-colors hover:bg-surface-5"
										@click.stop="draftSelectedTimeframeMode = 'preset'"
									>
										Preset timeframes...
									</button>
								</template>
								<template v-else>
									<CustomTimeframe
										v-model:amount="draftSelectedLastTimeframeAmount"
										v-model:unit="draftSelectedLastTimeframeUnit"
										:unit-options="lastTimeframeUnitOptions"
										@activate="draftSelectedTimeframeMode = 'last'"
									/>
									<button
										type="button"
										class="flex cursor-pointer items-center border-0 border-t border-solid border-surface-5 bg-transparent px-4 py-3 text-left text-sm font-semibold text-primary transition-colors hover:bg-surface-5"
										@click.stop="switchDraftToCustomDateRange"
									>
										Custom fixed date range...
									</button>
								</template>
							</div>
						</template>
					</Combobox>
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
import { BlocksIcon, CalendarIcon, ChevronLeftIcon, FolderOpenIcon, XIcon } from '@modrinth/assets'
import { Combobox, type ComboboxOption, MultiSelect, type MultiSelectOption } from '@modrinth/ui'

import {
	type AnalyticsBreakdownPreset,
	type AnalyticsGroupByPreset,
	type AnalyticsLastTimeframeUnit,
	type AnalyticsSelectedFilters,
	type AnalyticsTimeframeMode,
	type AnalyticsTimeframePreset,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

import CustomRangeTimeframe from './CustomRangeTimeframe.vue'
import CustomTimeframe from './CustomTimeframe.vue'
import QueryBuilderFilter from './QueryFilter.vue'

const MIN_RANGE_MS = 60 * 60 * 1000
const MAX_TIME_SLICES = 1024
const QUERY_BUILDER_DROPDOWN_MAX_HEIGHT = 500
const QUERY_BUILDER_DROPDOWN_MIN_WIDTH = '12rem'
const TIME_RANGE_ROUNDING_MS = 60 * 1000

const {
	projects,
	selectedProjectIds,
	selectedTimeframeMode,
	selectedTimeframe,
	selectedLastTimeframeAmount,
	selectedLastTimeframeUnit,
	selectedCustomTimeframeStartDate,
	selectedCustomTimeframeEndDate,
	selectedGroupBy,
	selectedBreakdown,
	selectedFilters,
	queryRefreshTimestamp,
	setFetchRequest,
} = injectAnalyticsDashboardContext()

const projectOptions = computed<MultiSelectOption<string>[]>(() =>
	projects.value.map((project) => ({
		value: project.id,
		label: project.name,
	})),
)

const allProjectIds = computed(() => projectOptions.value.map((project) => project.value))
const isProjectSelectOpen = ref(false)
const draftSelectedProjectIds = ref<string[]>([...selectedProjectIds.value])
const isTimeframeSelectOpen = ref(false)
const draftSelectedTimeframeMode = ref(selectedTimeframeMode.value)
const draftSelectedTimeframe = ref(selectedTimeframe.value)
const draftSelectedLastTimeframeAmount = ref(selectedLastTimeframeAmount.value)
const draftSelectedLastTimeframeUnit = ref(selectedLastTimeframeUnit.value)
const draftSelectedCustomTimeframeStartDate = ref(selectedCustomTimeframeStartDate.value)
const draftSelectedCustomTimeframeEndDate = ref(selectedCustomTimeframeEndDate.value)
const projectDownloadsThresholdInput = ref('')

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

const showProjectRow = computed(() => projects.value.length > 1)

function parseProjectDownloadsThreshold(value: string): number | null {
	const normalizedValue = value.trim().toLowerCase().replace(/,/g, '')
	if (!normalizedValue) {
		return null
	}

	const match = normalizedValue.match(/^(\d+(?:\.\d+)?)([kmb])?$/)
	if (!match) {
		return null
	}

	const amount = Number.parseFloat(match[1])
	if (!Number.isFinite(amount)) {
		return null
	}

	const multiplierBySuffix: Record<string, number> = {
		k: 1_000,
		m: 1_000_000,
		b: 1_000_000_000,
	}

	const multiplier = match[2] ? multiplierBySuffix[match[2]] : 1
	return Math.max(0, Math.floor(amount * multiplier))
}

function formatCompactNumber(value: number): string {
	const formatWithSuffix = (divisor: number, suffix: string) => {
		const dividedValue = value / divisor
		const fractionDigits = Number.isInteger(dividedValue) ? 0 : 1
		return `${dividedValue.toFixed(fractionDigits).replace(/\.0$/, '')}${suffix}`
	}

	if (value >= 1_000_000_000) return formatWithSuffix(1_000_000_000, 'B')
	if (value >= 1_000_000) return formatWithSuffix(1_000_000, 'M')
	if (value >= 1_000) return formatWithSuffix(1_000, 'k')
	return String(value)
}

function applyProjectDownloadsThreshold() {
	const threshold = parseProjectDownloadsThreshold(projectDownloadsThresholdInput.value)
	if (threshold === null) {
		return
	}

	draftSelectedProjectIds.value = projects.value
		.filter((project) => project.downloads >= threshold)
		.map((project) => project.id)
}

function formatProjectDownloadsThresholdInput() {
	const threshold = parseProjectDownloadsThreshold(projectDownloadsThresholdInput.value)
	if (threshold === null) {
		return
	}

	projectDownloadsThresholdInput.value = formatCompactNumber(threshold)
}

watch(
	projectDownloadsThresholdInput,
	() => {
		applyProjectDownloadsThreshold()
	}
)

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

const lastTimeframeUnitOptions: Array<{
	value: AnalyticsLastTimeframeUnit
	label: string
	singularLabel: string
}> = [
	{ value: 'hours', label: 'hours', singularLabel: 'hour' },
	{ value: 'days', label: 'days', singularLabel: 'day' },
	{ value: 'weeks', label: 'weeks', singularLabel: 'week' },
	{ value: 'months', label: 'months', singularLabel: 'month' },
]

const lastTimeframeValueByPreset: Partial<
	Record<
		AnalyticsTimeframePreset,
		{
			amount: number
			unit: AnalyticsLastTimeframeUnit
		}
	>
> = {
	today: { amount: 1, unit: 'days' },
	yesterday: { amount: 1, unit: 'days' },
	last_7_days: { amount: 7, unit: 'days' },
	last_14_days: { amount: 14, unit: 'days' },
	last_30_days: { amount: 30, unit: 'days' },
	last_90_days: { amount: 90, unit: 'days' },
	last_180_days: { amount: 180, unit: 'days' },
}

const timeframeDropdownOptions = computed<ComboboxOption<AnalyticsTimeframePreset>[]>(() =>
	draftSelectedTimeframeMode.value === 'custom_range' ? [] : timeframeOptions,
)

const selectedTimeframeLabel = computed(() => {
	return getTimeframeLabel(
		isTimeframeSelectOpen.value ? draftSelectedTimeframeMode.value : selectedTimeframeMode.value,
		isTimeframeSelectOpen.value ? draftSelectedTimeframe.value : selectedTimeframe.value,
		isTimeframeSelectOpen.value
			? draftSelectedLastTimeframeAmount.value
			: selectedLastTimeframeAmount.value,
		isTimeframeSelectOpen.value
			? draftSelectedLastTimeframeUnit.value
			: selectedLastTimeframeUnit.value,
		isTimeframeSelectOpen.value
			? draftSelectedCustomTimeframeStartDate.value
			: selectedCustomTimeframeStartDate.value,
		isTimeframeSelectOpen.value
			? draftSelectedCustomTimeframeEndDate.value
			: selectedCustomTimeframeEndDate.value,
	)
})

function getTimeframeLabel(
	mode: AnalyticsTimeframeMode,
	preset: AnalyticsTimeframePreset,
	lastAmount: number,
	lastUnit: AnalyticsLastTimeframeUnit,
	customStartDate: string,
	customEndDate: string,
): string {
	if (mode === 'last') {
		const unit = lastTimeframeUnitOptions.find((option) => option.value === lastUnit)
		const unitLabel = lastAmount === 1 ? unit?.singularLabel : unit?.label
		return `In the last ${lastAmount} ${unitLabel ?? lastUnit}`
	}

	if (mode === 'custom_range') {
		return `${customStartDate} to ${customEndDate}`
	}

	return timeframeOptions.find((option) => option.value === preset)?.label ?? 'Select timeframe'
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

function startOfDay(date: Date): Date {
	const nextDate = new Date(date)
	nextDate.setHours(0, 0, 0, 0)
	return nextDate
}

function getRoundedNow(timestamp: number): Date {
	const roundedTimestamp = Math.floor(timestamp / TIME_RANGE_ROUNDING_MS) * TIME_RANGE_ROUNDING_MS
	return new Date(roundedTimestamp)
}

function getDateInputValue(date: Date): string {
	const year = date.getFullYear()
	const month = String(date.getMonth() + 1).padStart(2, '0')
	const day = String(date.getDate()).padStart(2, '0')
	return `${year}-${month}-${day}`
}

function parseDateInputValue(value: string): Date {
	const parsedDate = new Date(`${value}T00:00:00`)
	return Number.isNaN(parsedDate.getTime()) ? startOfDay(new Date()) : parsedDate
}

function addDays(date: Date, days: number): Date {
	const nextDate = new Date(date)
	nextDate.setDate(nextDate.getDate() + days)
	return nextDate
}

function isStartOfDay(date: Date): boolean {
	return (
		date.getHours() === 0 &&
		date.getMinutes() === 0 &&
		date.getSeconds() === 0 &&
		date.getMilliseconds() === 0
	)
}

function getInclusiveEndDateInputValue(end: Date): string {
	return getDateInputValue(isStartOfDay(end) ? addDays(end, -1) : end)
}

function subtractCalendarMonths(date: Date, months: number): Date {
	const nextDate = new Date(date)
	const day = nextDate.getDate()
	nextDate.setDate(1)
	nextDate.setMonth(nextDate.getMonth() - months)
	const daysInMonth = new Date(nextDate.getFullYear(), nextDate.getMonth() + 1, 0).getDate()
	nextDate.setDate(Math.min(day, daysInMonth))
	return nextDate
}

function getTimeRangeForPreset(
	preset: AnalyticsTimeframePreset,
	nowTimestamp: number,
): { start: Date; end: Date } {
	const now = getRoundedNow(nowTimestamp)
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

function getCurrentTimeRangeForPreset(preset: AnalyticsTimeframePreset): {
	start: Date
	end: Date
} {
	return getTimeRangeForPreset(preset, queryRefreshTimestamp.value)
}

function getTimeRangeForLastTimeframe(
	amountValue: number,
	unit: AnalyticsLastTimeframeUnit,
): { start: Date; end: Date } {
	const end = getRoundedNow(queryRefreshTimestamp.value)
	const amount = Math.max(1, Math.floor(amountValue))

	switch (unit) {
		case 'hours':
			return { start: new Date(end.getTime() - amount * 60 * 60 * 1000), end }
		case 'days':
			return { start: new Date(end.getTime() - amount * 24 * 60 * 60 * 1000), end }
		case 'weeks':
			return { start: new Date(end.getTime() - amount * 7 * 24 * 60 * 60 * 1000), end }
		case 'months':
			return { start: subtractCalendarMonths(end, amount), end }
		default:
			return { start: new Date(end.getTime() - 24 * 60 * 60 * 1000), end }
	}
}

function getCurrentTimeRangeForLastTimeframe(): { start: Date; end: Date } {
	return getTimeRangeForLastTimeframe(
		selectedLastTimeframeAmount.value,
		selectedLastTimeframeUnit.value,
	)
}

function getTimeRangeForCustomDateRange(
	startDate: string,
	endDate: string,
): { start: Date; end: Date } {
	const start = parseDateInputValue(startDate)
	const inclusiveEnd = parseDateInputValue(endDate)
	return {
		start,
		end: addDays(inclusiveEnd, 1),
	}
}

function getCurrentTimeRangeForCustomDateRange(): { start: Date; end: Date } {
	return getTimeRangeForCustomDateRange(
		selectedCustomTimeframeStartDate.value,
		selectedCustomTimeframeEndDate.value,
	)
}

function getCurrentTimeRange(): { start: Date; end: Date } {
	switch (selectedTimeframeMode.value) {
		case 'last':
			return getCurrentTimeRangeForLastTimeframe()
		case 'custom_range':
			return getCurrentTimeRangeForCustomDateRange()
		case 'preset':
		default:
			return getCurrentTimeRangeForPreset(selectedTimeframe.value)
	}
}

function getDraftTimeRange(): { start: Date; end: Date } {
	switch (draftSelectedTimeframeMode.value) {
		case 'last':
			return getTimeRangeForLastTimeframe(
				draftSelectedLastTimeframeAmount.value,
				draftSelectedLastTimeframeUnit.value,
			)
		case 'custom_range':
			return getTimeRangeForCustomDateRange(
				draftSelectedCustomTimeframeStartDate.value,
				draftSelectedCustomTimeframeEndDate.value,
			)
		case 'preset':
		default:
			return getCurrentTimeRangeForPreset(draftSelectedTimeframe.value)
	}
}

function resetTimeframeDraft() {
	draftSelectedTimeframeMode.value = selectedTimeframeMode.value
	draftSelectedTimeframe.value = selectedTimeframe.value
	draftSelectedLastTimeframeAmount.value = selectedLastTimeframeAmount.value
	draftSelectedLastTimeframeUnit.value = selectedLastTimeframeUnit.value
	draftSelectedCustomTimeframeStartDate.value = selectedCustomTimeframeStartDate.value
	draftSelectedCustomTimeframeEndDate.value = selectedCustomTimeframeEndDate.value
}

function commitTimeframeDraft() {
	selectedTimeframeMode.value = draftSelectedTimeframeMode.value
	selectedTimeframe.value = draftSelectedTimeframe.value
	selectedLastTimeframeAmount.value = draftSelectedLastTimeframeAmount.value
	selectedLastTimeframeUnit.value = draftSelectedLastTimeframeUnit.value
	selectedCustomTimeframeStartDate.value = draftSelectedCustomTimeframeStartDate.value
	selectedCustomTimeframeEndDate.value = draftSelectedCustomTimeframeEndDate.value
}

function handleTimeframeSelectOpen() {
	resetTimeframeDraft()
	isTimeframeSelectOpen.value = true
}

function handleTimeframeSelectClose() {
	commitTimeframeDraft()
	isTimeframeSelectOpen.value = false
}

watch(
	[
		selectedTimeframeMode,
		selectedTimeframe,
		selectedLastTimeframeAmount,
		selectedLastTimeframeUnit,
		selectedCustomTimeframeStartDate,
		selectedCustomTimeframeEndDate,
	],
	() => {
		if (isTimeframeSelectOpen.value) {
			return
		}

		resetTimeframeDraft()
	},
)

function handleTimeframePresetSelect(option: ComboboxOption<AnalyticsTimeframePreset>) {
	const lastTimeframeValue = lastTimeframeValueByPreset[option.value]
	if (lastTimeframeValue) {
		draftSelectedLastTimeframeAmount.value = lastTimeframeValue.amount
		draftSelectedLastTimeframeUnit.value = lastTimeframeValue.unit
	}

	draftSelectedTimeframeMode.value = 'preset'
}

function switchDraftToCustomDateRange() {
	const rawRange = getDraftTimeRange()
	draftSelectedCustomTimeframeStartDate.value = getDateInputValue(rawRange.start)
	draftSelectedCustomTimeframeEndDate.value = getInclusiveEndDateInputValue(rawRange.end)
	draftSelectedTimeframeMode.value = 'custom_range'
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
	const rawRange = getCurrentTimeRange()
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
		case 'version_id':
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

	if (filters.version_id.length > 0) {
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
	const rawRange = getCurrentTimeRange()
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
