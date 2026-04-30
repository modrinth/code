<template>
	<Combobox
		v-model="draftSelectedTimeframe"
		:options="timeframeDropdownOptions"
		:display-value="selectedTimeframeLabel"
		:max-height="TIMEFRAME_DROPDOWN_MAX_HEIGHT"
		:dropdown-min-width="'20rem'"
		@open="handleTimeframeSelectOpen"
		@close="handleTimeframeSelectClose"
		@select="handleTimeframePresetSelect"
	>
		<template #bottom>
			<div class="flex flex-col border-0 border-t border-solid border-surface-5 bg-surface-4">
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
</template>

<script setup lang="ts">
import { Combobox, type ComboboxOption } from '@modrinth/ui'

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
</script>
