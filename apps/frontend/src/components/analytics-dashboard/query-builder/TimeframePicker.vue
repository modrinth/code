<template>
	<BaseTimeFramePicker
		v-model:mode="selectedTimeframeMode"
		v-model:preset="selectedTimeframe"
		v-model:last-amount="selectedLastTimeframeAmount"
		v-model:last-unit="selectedLastTimeframeUnit"
		v-model:custom-start-date="selectedCustomTimeframeStartDate"
		v-model:custom-end-date="selectedCustomTimeframeEndDate"
		:min-date="ANALYTICS_START_DATE_INPUT_VALUE"
		:now-timestamp="queryRefreshTimestamp"
		:trigger-class="triggerClass"
		@open="handleTimeframeOpen"
		@commit="handleTimeframeCommit"
		@apply="handleTimeframeApply"
		@draft-change="handleTimeframeDraftChange"
		@preset-select="handleTimeframePresetSelect"
	>
		<template #prefix>
			<slot name="prefix"></slot>
		</template>
	</BaseTimeFramePicker>
</template>

<script setup lang="ts">
import {
	type ComboboxOption,
	TimeFramePicker as BaseTimeFramePicker,
	type TimeFramePickerSelection,
	type TimeFramePreset,
} from '@modrinth/ui'

import {
	ANALYTICS_START_DATE_INPUT_VALUE,
	type AnalyticsGroupByPreset,
	type AnalyticsTimeframePreset,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

import {
	ensureMinimumTimeRange,
	getAnalyticsTimeRange,
	getDateInputValue,
	getDefaultAnalyticsGroupByForDurationMinutes,
} from './timeframe'

const {
	selectedTimeframeMode,
	selectedTimeframe,
	selectedLastTimeframeAmount,
	selectedLastTimeframeUnit,
	selectedCustomTimeframeStartDate,
	selectedCustomTimeframeEndDate,
	selectedGroupBy,
	queryRefreshTimestamp,
	analyticsAllTimeStartDate,
	refreshAnalyticsQuery,
} = injectAnalyticsDashboardContext()

defineProps<{
	triggerClass?: string
}>()

const draftSelectedGroupBy = ref(selectedGroupBy.value)

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

function handleTimeframeOpen() {
	draftSelectedGroupBy.value = selectedGroupBy.value
}

function handleTimeframeCommit() {
	selectedGroupBy.value = draftSelectedGroupBy.value
}

async function handleTimeframeApply() {
	await refreshAnalyticsQuery()
}

function handleTimeframePresetSelect(option: ComboboxOption<TimeFramePreset>) {
	const defaultGroupBy = defaultGroupByForPreset[option.value as AnalyticsTimeframePreset]
	if (defaultGroupBy) {
		draftSelectedGroupBy.value = defaultGroupBy
	}
}

function handleTimeframeDraftChange(selection: TimeFramePickerSelection) {
	if (selection.mode !== 'last' && selection.mode !== 'custom_range') {
		return
	}
	if (selection.mode === 'custom_range' && !hasCompleteCustomDateRange(selection)) {
		return
	}

	const range = getAnalyticsTimeRange({
		mode: selection.mode,
		preset: selection.preset,
		lastAmount: selection.lastAmount,
		lastUnit: selection.lastUnit,
		customStartDate: selection.customStartDate,
		customEndDate: selection.customEndDate,
		nowTimestamp: queryRefreshTimestamp.value,
		allTimeStartDate: analyticsAllTimeStartDate.value,
	})
	const { start, end } = ensureMinimumTimeRange(range.start, range.end)
	const durationMinutes = Math.max(1, Math.floor((end.getTime() - start.getTime()) / 60000))
	draftSelectedGroupBy.value = getDefaultAnalyticsGroupByForDurationMinutes(durationMinutes)
}

function hasCompleteCustomDateRange(selection: TimeFramePickerSelection) {
	return Boolean(
		getDateFromInputValue(selection.customStartDate) &&
		getDateFromInputValue(selection.customEndDate),
	)
}

function getDateFromInputValue(value: string): Date | undefined {
	const date = new Date(`${value}T00:00:00`)
	if (Number.isNaN(date.getTime()) || getDateInputValue(date) !== value) {
		return undefined
	}

	return date
}
</script>
