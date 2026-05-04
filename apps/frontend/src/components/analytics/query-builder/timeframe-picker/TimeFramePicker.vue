<template>
	<Combobox
		:key="timeframeSelectKey"
		:model-value="highlightedTimeframePreset"
		:options="timeframeDropdownOptions"
		:display-value="selectedTimeframeLabel"
		:max-height="TIMEFRAME_DROPDOWN_MAX_HEIGHT"
		:dropdown-min-width="'20rem'"
		@update:model-value="handleTimeframeModelUpdate"
		@open="handleTimeframeSelectOpen"
		@close="handleTimeframeSelectClose"
		@select="handleTimeframePresetSelect"
	>
		<template #bottom>
			<div class="flex flex-col border-0 border-t border-solid border-surface-5 bg-surface-4">
				<template v-if="activeTimeframePanel === 'custom_range'">
					<CustomRangeTimeframe
						v-model:start-date="draftSelectedCustomTimeframeStartDate"
						v-model:end-date="draftSelectedCustomTimeframeEndDate"
					/>
					<div
						class="flex items-center justify-between gap-3 border-0 border-t border-solid border-surface-5 px-4 py-3"
					>
						<div class="min-w-0 truncate text-sm text-secondary">
							Selected period:
							<span class="font-semibold text-primary">{{ draftCustomTimeframeRangeLabel }}</span>
						</div>
						<div class="flex shrink-0 items-center gap-2">
							<ButtonStyled type="outlined">
								<button type="button" @click.stop="cancelCustomDateRange">Cancel</button>
							</ButtonStyled>
							<ButtonStyled color="brand">
								<button type="button" @click.stop="applyCustomDateRange">Apply</button>
							</ButtonStyled>
						</div>
					</div>
				</template>
				<template v-else>
					<CustomTimeframe
						v-model:amount="draftSelectedLastTimeframeAmount"
						v-model:unit="draftSelectedLastTimeframeUnit"
						:active="draftSelectedTimeframeMode === 'last'"
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
</template>

<script setup lang="ts">
import { ButtonStyled, Combobox, type ComboboxOption } from '@modrinth/ui'

import {
	type AnalyticsLastTimeframeUnit,
	type AnalyticsTimeframeMode,
	type AnalyticsTimeframePreset,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

import CustomRangeTimeframe from './CustomRangeTimeframe.vue'
import CustomTimeframe from './CustomTimeframe.vue'
import {
	getAnalyticsTimeRange,
	getDateInputValue,
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

const {
	selectedTimeframeMode,
	selectedTimeframe,
	selectedLastTimeframeAmount,
	selectedLastTimeframeUnit,
	selectedCustomTimeframeStartDate,
	selectedCustomTimeframeEndDate,
	queryRefreshTimestamp,
} = injectAnalyticsDashboardContext()

const isTimeframeSelectOpen = ref(false)
const timeframeSelectKey = ref(0)
const activeTimeframePanel = ref<'preset' | 'custom_range'>('preset')
const shouldCommitTimeframeDraftOnClose = ref(true)
const draftSelectedTimeframeMode = ref(selectedTimeframeMode.value)
const draftSelectedTimeframe = ref(selectedTimeframe.value)
const draftSelectedLastTimeframeAmount = ref(selectedLastTimeframeAmount.value)
const draftSelectedLastTimeframeUnit = ref(selectedLastTimeframeUnit.value)
const draftSelectedCustomTimeframeStartDate = ref(selectedCustomTimeframeStartDate.value)
const draftSelectedCustomTimeframeEndDate = ref(selectedCustomTimeframeEndDate.value)

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
	activeTimeframePanel.value === 'custom_range' ? [] : timeframeOptions,
)

const highlightedTimeframePreset = computed<AnalyticsTimeframePreset | undefined>(() => {
	if (draftSelectedTimeframeMode.value === 'preset') {
		return draftSelectedTimeframe.value
	}

	return undefined
})

const selectedTimeframeLabel = computed(() => {
	const useDraftTimeframeLabel =
		isTimeframeSelectOpen.value && shouldCommitTimeframeDraftOnClose.value

	return getTimeframeLabel(
		useDraftTimeframeLabel ? draftSelectedTimeframeMode.value : selectedTimeframeMode.value,
		useDraftTimeframeLabel ? draftSelectedTimeframe.value : selectedTimeframe.value,
		useDraftTimeframeLabel
			? draftSelectedLastTimeframeAmount.value
			: selectedLastTimeframeAmount.value,
		useDraftTimeframeLabel
			? draftSelectedLastTimeframeUnit.value
			: selectedLastTimeframeUnit.value,
		useDraftTimeframeLabel
			? draftSelectedCustomTimeframeStartDate.value
			: selectedCustomTimeframeStartDate.value,
		useDraftTimeframeLabel
			? draftSelectedCustomTimeframeEndDate.value
			: selectedCustomTimeframeEndDate.value,
	)
})

const draftCustomTimeframeRangeLabel = computed(() =>
	formatCustomTimeframeRangeLabel(
		draftSelectedCustomTimeframeStartDate.value,
		draftSelectedCustomTimeframeEndDate.value,
	),
)

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
	activeTimeframePanel.value = 'preset'
	shouldCommitTimeframeDraftOnClose.value = true
	isTimeframeSelectOpen.value = true
}

function handleTimeframeSelectClose() {
	if (shouldCommitTimeframeDraftOnClose.value) {
		commitTimeframeDraft()
	} else {
		resetTimeframeDraft()
	}

	isTimeframeSelectOpen.value = false
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
	activeTimeframePanel.value = 'custom_range'
	shouldCommitTimeframeDraftOnClose.value = false
}

function cancelCustomDateRange() {
	resetTimeframeDraft()
	activeTimeframePanel.value = 'preset'
	shouldCommitTimeframeDraftOnClose.value = true
}

function applyCustomDateRange() {
	commitTimeframeDraft()
	activeTimeframePanel.value = 'preset'
	shouldCommitTimeframeDraftOnClose.value = true
	isTimeframeSelectOpen.value = false
	timeframeSelectKey.value++
}
</script>
