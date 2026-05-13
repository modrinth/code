<template>
	<Combobox
		:model-value="highlightedTimeframePreset"
		:options="timeframeDropdownOptions"
		:display-value="selectedTimeframeLabel"
		:max-height="TIMEFRAME_DROPDOWN_MAX_HEIGHT"
		:dropdown-min-width="timeframeDropdownMinWidth"
		:dropdown-class="
			activeTimeframePanel === 'custom_range'
				? 'bg-transparent border-0 -mt-1 pb-2 shadow-none'
				: ''
		"
		@update:model-value="handleTimeframeModelUpdate"
		@open="handleTimeframeSelectOpen"
		@close="handleTimeframeSelectClose"
		@select="handleTimeframePresetSelect"
	>
		<template #bottom>
			<template v-if="activeTimeframePanel === 'custom_range'">
				<CustomRangeTimeframe
					v-model:start-date="draftSelectedCustomTimeframeStartDate"
					v-model:end-date="draftSelectedCustomTimeframeEndDate"
					@cancel="handleCustomRangeCancel"
					@apply="handleCustomRangeApply"
				/>
			</template>
			<div
				v-else
				class="flex flex-col border-0 border-t border-solid border-surface-5 bg-surface-4"
			>
				<CustomTimeframe
					v-model:amount="draftSelectedLastTimeframeAmount"
					v-model:unit="draftSelectedLastTimeframeUnit"
					:active="draftSelectedTimeframeMode === 'last'"
					:unit-options="lastTimeframeUnitOptions"
					@activate="draftSelectedTimeframeMode = 'last'"
					@submit="runCustomTimeframeQuery"
				/>
				<button
					type="button"
					class="flex cursor-pointer items-center border-0 border-t border-solid border-surface-5 bg-transparent px-4 py-3 text-left text-sm font-semibold text-primary transition-colors hover:bg-surface-5"
					@click.stop="switchDraftToCustomDateRange"
				>
					Custom fixed date range...
				</button>
			</div>
		</template>
	</Combobox>
</template>

<script setup lang="ts">
import { Combobox, type ComboboxOption } from '@modrinth/ui'

import {
	type AnalyticsGroupByPreset,
	type AnalyticsLastTimeframeUnit,
	type AnalyticsTimeframeMode,
	type AnalyticsTimeframePreset,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

import CustomRangeTimeframe from './CustomRangeTimeframe.vue'
import CustomTimeframe from './CustomTimeframe.vue'
import {
	ensureMinimumTimeRange,
	getAnalyticsTimeRange,
	getDateInputValue,
	getDefaultAnalyticsGroupByForDurationMinutes,
	getInclusiveEndDateInputValue,
} from './timeframe'

const TIMEFRAME_DROPDOWN_MAX_HEIGHT = 500
const CUSTOM_RANGE_DATE_FORMATTER = new Intl.DateTimeFormat('en-US', {
	month: 'long',
	day: 'numeric',
	year: 'numeric',
})
const CUSTOM_RANGE_MONTH_DAY_FORMATTER = new Intl.DateTimeFormat('en-US', {
	month: 'long',
	day: 'numeric',
})
const CUSTOM_DATE_TIME_RANGE_FORMATTER = new Intl.DateTimeFormat('en-US', {
	month: 'short',
	day: 'numeric',
	year: 'numeric',
	hour: 'numeric',
	minute: '2-digit',
})
const CUSTOM_DATE_TIME_RANGE_MONTH_DAY_FORMATTER = new Intl.DateTimeFormat('en-US', {
	month: 'short',
	day: 'numeric',
	hour: 'numeric',
	minute: '2-digit',
})

const {
	selectedTimeframeMode,
	selectedTimeframe,
	selectedLastTimeframeAmount,
	selectedLastTimeframeUnit,
	selectedCustomTimeframeStartDate,
	selectedCustomTimeframeEndDate,
	selectedGroupBy,
	queryRefreshTimestamp,
	refreshAnalyticsQuery,
} = injectAnalyticsDashboardContext()

const isTimeframeSelectOpen = ref(false)
const activeTimeframePanel = ref<'preset' | 'custom_range'>('preset')
const draftSelectedTimeframeMode = ref(selectedTimeframeMode.value)
const draftSelectedTimeframe = ref(selectedTimeframe.value)
const draftSelectedLastTimeframeAmount = ref(selectedLastTimeframeAmount.value)
const draftSelectedLastTimeframeUnit = ref(selectedLastTimeframeUnit.value)
const draftSelectedCustomTimeframeStartDate = ref(selectedCustomTimeframeStartDate.value)
const draftSelectedCustomTimeframeEndDate = ref(selectedCustomTimeframeEndDate.value)
const draftSelectedGroupBy = ref(selectedGroupBy.value)

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

const defaultGroupByForPreset: Partial<Record<AnalyticsTimeframePreset, AnalyticsGroupByPreset>> = {
	today: '1h',
	yesterday: '1h',
	last_7_days: '6h',
	last_14_days: 'day',
	last_30_days: 'day',
	last_90_days: 'day',
	last_180_days: 'week',
	year_to_date: 'week',
}

const timeframeDropdownOptions = computed<ComboboxOption<AnalyticsTimeframePreset>[]>(() =>
	activeTimeframePanel.value === 'custom_range' ? [] : timeframeOptions,
)
const timeframeDropdownMinWidth = computed(() =>
	activeTimeframePanel.value === 'custom_range' ? '40.5rem' : '20rem',
)

const highlightedTimeframePreset = computed<AnalyticsTimeframePreset | undefined>(() => {
	if (draftSelectedTimeframeMode.value === 'preset') {
		return draftSelectedTimeframe.value
	}

	return undefined
})

const selectedTimeframeLabel = computed(() => {
	const useDraftTimeframeLabel =
		isTimeframeSelectOpen.value && activeTimeframePanel.value !== 'custom_range'

	return getTimeframeLabel(
		useDraftTimeframeLabel ? draftSelectedTimeframeMode.value : selectedTimeframeMode.value,
		useDraftTimeframeLabel ? draftSelectedTimeframe.value : selectedTimeframe.value,
		useDraftTimeframeLabel
			? draftSelectedLastTimeframeAmount.value
			: selectedLastTimeframeAmount.value,
		useDraftTimeframeLabel ? draftSelectedLastTimeframeUnit.value : selectedLastTimeframeUnit.value,
		useDraftTimeframeLabel
			? draftSelectedCustomTimeframeStartDate.value
			: selectedCustomTimeframeStartDate.value,
		useDraftTimeframeLabel
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
		return formatCustomTimeframeRangeLabel(customStartDate, customEndDate)
	}

	if (mode === 'custom_datetime_range') {
		return formatCustomDateTimeRangeLabel(customStartDate, customEndDate)
	}

	return timeframeOptions.find((option) => option.value === preset)?.label ?? 'Select timeframe'
}

function getDraftTimeRange() {
	return getAnalyticsTimeRange({
		mode: draftSelectedTimeframeMode.value,
		preset: draftSelectedTimeframe.value,
		lastAmount: draftSelectedLastTimeframeAmount.value,
		lastUnit: draftSelectedLastTimeframeUnit.value,
		customStartDate: draftSelectedCustomTimeframeStartDate.value,
		customEndDate: draftSelectedCustomTimeframeEndDate.value,
		nowTimestamp: queryRefreshTimestamp.value,
	})
}

function getDateFromInputValue(value: string): Date | undefined {
	const date = new Date(`${value}T00:00:00`)
	if (Number.isNaN(date.getTime()) || getDateInputValue(date) !== value) {
		return undefined
	}

	return date
}

function formatCustomTimeframeRangeLabel(startDateValue: string, endDateValue: string): string {
	const startDate = getDateFromInputValue(startDateValue)
	const endDate = getDateFromInputValue(endDateValue)
	if (!startDate || !endDate) {
		return `${startDateValue} - ${endDateValue}`
	}

	const sameYear = startDate.getFullYear() === endDate.getFullYear()

	if (startDateValue === endDateValue) {
		return CUSTOM_RANGE_DATE_FORMATTER.format(startDate)
	}

	if (sameYear) {
		const startLabel = CUSTOM_RANGE_MONTH_DAY_FORMATTER.format(startDate)
		const endLabel = CUSTOM_RANGE_MONTH_DAY_FORMATTER.format(endDate)
		return `${startLabel} - ${endLabel}, ${startDate.getFullYear()}`
	}

	const startLabel = CUSTOM_RANGE_DATE_FORMATTER.format(startDate)
	const endLabel = CUSTOM_RANGE_DATE_FORMATTER.format(endDate)
	return `${startLabel} - ${endLabel}`
}

function getDateTimeFromInputValue(value: string): Date | undefined {
	const date = new Date(value)
	if (Number.isNaN(date.getTime())) {
		return undefined
	}

	return date
}

function formatCustomDateTimeRangeLabel(startDateValue: string, endDateValue: string): string {
	const startDate = getDateTimeFromInputValue(startDateValue)
	const endDate = getDateTimeFromInputValue(endDateValue)
	if (!startDate || !endDate) {
		return `${startDateValue} - ${endDateValue}`
	}

	if (startDate.getTime() === endDate.getTime()) {
		return CUSTOM_DATE_TIME_RANGE_FORMATTER.format(startDate)
	}

	const sameYear = startDate.getFullYear() === endDate.getFullYear()
	if (sameYear) {
		const startLabel = CUSTOM_DATE_TIME_RANGE_MONTH_DAY_FORMATTER.format(startDate)
		const endLabel = CUSTOM_DATE_TIME_RANGE_MONTH_DAY_FORMATTER.format(endDate)
		return `${startLabel} - ${endLabel}, ${startDate.getFullYear()}`
	}

	const startLabel = CUSTOM_DATE_TIME_RANGE_FORMATTER.format(startDate)
	const endLabel = CUSTOM_DATE_TIME_RANGE_FORMATTER.format(endDate)
	return `${startLabel} - ${endLabel}`
}

function resetTimeframeDraft() {
	draftSelectedTimeframeMode.value = selectedTimeframeMode.value
	draftSelectedTimeframe.value = selectedTimeframe.value
	draftSelectedLastTimeframeAmount.value = selectedLastTimeframeAmount.value
	draftSelectedLastTimeframeUnit.value = selectedLastTimeframeUnit.value
	draftSelectedCustomTimeframeStartDate.value = selectedCustomTimeframeStartDate.value
	draftSelectedCustomTimeframeEndDate.value = selectedCustomTimeframeEndDate.value
	draftSelectedGroupBy.value = selectedGroupBy.value
}

function commitTimeframeDraft() {
	selectedTimeframeMode.value = draftSelectedTimeframeMode.value
	selectedTimeframe.value = draftSelectedTimeframe.value
	selectedLastTimeframeAmount.value = draftSelectedLastTimeframeAmount.value
	selectedLastTimeframeUnit.value = draftSelectedLastTimeframeUnit.value
	selectedCustomTimeframeStartDate.value = draftSelectedCustomTimeframeStartDate.value
	selectedCustomTimeframeEndDate.value = draftSelectedCustomTimeframeEndDate.value
	selectedGroupBy.value = draftSelectedGroupBy.value
}

function handleTimeframeSelectOpen() {
	resetTimeframeDraft()
	activeTimeframePanel.value = 'preset'
	isTimeframeSelectOpen.value = true
}

function handleTimeframeSelectClose() {
	if (activeTimeframePanel.value !== 'custom_range') {
		commitTimeframeDraft()
	}
	isTimeframeSelectOpen.value = false
}

function closeTimeframeSelectDropdown(event: Event) {
	const eventTarget = event.target
	if (!(eventTarget instanceof HTMLElement)) {
		isTimeframeSelectOpen.value = false
		return
	}

	const dropdown = eventTarget.closest('[role="listbox"], [role="menu"]')
	if (!dropdown) {
		isTimeframeSelectOpen.value = false
		return
	}

	dropdown.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
}

async function runCustomTimeframeQuery(event: KeyboardEvent) {
	commitTimeframeDraft()
	closeTimeframeSelectDropdown(event)
	await nextTick()
	await refreshAnalyticsQuery()
}

function handleCustomRangeCancel() {
	resetTimeframeDraft()
	activeTimeframePanel.value = 'preset'
}

async function handleCustomRangeApply(event: MouseEvent) {
	draftSelectedTimeframeMode.value = 'custom_range'
	commitTimeframeDraft()
	closeTimeframeSelectDropdown(event)
	await nextTick()
	await refreshAnalyticsQuery()
}

function handleTimeframeModelUpdate(value: AnalyticsTimeframePreset | undefined) {
	if (value !== undefined) {
		draftSelectedTimeframe.value = value
	}
}

watch(
	[
		selectedTimeframeMode,
		selectedTimeframe,
		selectedLastTimeframeAmount,
		selectedLastTimeframeUnit,
		selectedCustomTimeframeStartDate,
		selectedCustomTimeframeEndDate,
		selectedGroupBy,
	],
	() => {
		if (isTimeframeSelectOpen.value) {
			return
		}

		resetTimeframeDraft()
	},
)

function handleTimeframePresetSelect(option: ComboboxOption<AnalyticsTimeframePreset>) {
	draftSelectedTimeframeMode.value = 'preset'

	const lastTimeframeValue = lastTimeframeValueByPreset[option.value]
	if (lastTimeframeValue) {
		draftSelectedLastTimeframeAmount.value = lastTimeframeValue.amount
		draftSelectedLastTimeframeUnit.value = lastTimeframeValue.unit
	}

	const defaultGroupBy = defaultGroupByForPreset[option.value]
	if (defaultGroupBy) {
		draftSelectedGroupBy.value = defaultGroupBy
	}
}

watch(
	[
		draftSelectedTimeframeMode,
		draftSelectedLastTimeframeAmount,
		draftSelectedLastTimeframeUnit,
		draftSelectedCustomTimeframeStartDate,
		draftSelectedCustomTimeframeEndDate,
	],
	() => {
		if (!isTimeframeSelectOpen.value) {
			return
		}
		if (
			draftSelectedTimeframeMode.value !== 'last' &&
			draftSelectedTimeframeMode.value !== 'custom_range'
		) {
			return
		}

		const range = getDraftTimeRange()
		const { start, end } = ensureMinimumTimeRange(range.start, range.end)
		const durationMinutes = Math.max(1, Math.floor((end.getTime() - start.getTime()) / 60000))
		draftSelectedGroupBy.value = getDefaultAnalyticsGroupByForDurationMinutes(durationMinutes)
	},
	{ flush: 'sync' },
)

function switchDraftToCustomDateRange() {
	const rawRange = getDraftTimeRange()
	draftSelectedCustomTimeframeStartDate.value = getDateInputValue(rawRange.start)
	draftSelectedCustomTimeframeEndDate.value = getInclusiveEndDateInputValue(rawRange.end)
	draftSelectedTimeframeMode.value = 'custom_range'
	activeTimeframePanel.value = 'custom_range'
}
</script>
