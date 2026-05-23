<template>
	<Combobox
		:model-value="highlightedTimeframePreset"
		:options="timeframeDropdownOptions"
		:display-value="selectedTimeframeLabel"
		:max-height="maxHeight"
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
		<template #dropdown-footer>
			<template v-if="activeTimeframePanel === 'custom_range'">
				<div
					class="flex flex-col gap-0 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-0 pt-1"
				>
					<DatePicker
						v-model="pickerRange"
						mode="range"
						:show-months="2"
						:clearable="false"
						:default-view-date="todayInputValue"
						view-date-alignment="right"
						:min-date="minDate"
						:max-date="customRangeMaxDate"
						show-today
						calendar-only
						wrapper-class="w-full"
						calendar-class="!border-none"
					/>
					<div class="flex items-center justify-between p-4 pt-1">
						<div class="text-base">
							<template v-if="formattedRange">
								<div class="flex items-center gap-1.5">
									<span class="font-normal text-primary">{{ rangeLabel }}:</span>
									<span class="font-medium text-contrast">{{ formattedRange }}</span>
									<button
										v-if="selectedDraftDates.length !== 1"
										type="button"
										class="ml-1 border-0 bg-transparent p-0 font-normal text-primary underline hover:text-primary"
										@click.stop="clearRange"
									>
										{{ formatMessage(messages.clearRange) }}
									</button>
								</div>
							</template>
							<template v-else>
								<span class="font-normal text-primary">
									{{ formatMessage(messages.emptyRange) }}
								</span>
							</template>
						</div>

						<div class="flex items-center gap-2">
							<ButtonStyled type="outlined">
								<button type="button" @click="handleCustomRangeCancel">
									{{ formatMessage(messages.cancel) }}
								</button>
							</ButtonStyled>
							<ButtonStyled color="brand">
								<button type="button" :disabled="!hasCompleteRange" @click="handleCustomRangeApply">
									{{ formatMessage(messages.apply) }}
								</button>
							</ButtonStyled>
						</div>
					</div>
				</div>
			</template>
			<div
				v-else
				class="flex flex-col border-0 border-t border-solid border-surface-5 bg-surface-4"
			>
				<div
					class="px-3 py-2"
					:class="draftSelectedTimeframeMode === 'last' ? 'bg-highlight-green' : ''"
				>
					<div class="flex items-center gap-2.5 py-0.5 transition-colors">
						<span
							class="shrink-0 text-sm font-semibold"
							:class="draftSelectedTimeframeMode === 'last' ? 'text-green' : 'text-primary'"
						>
							{{ formatMessage(messages.lastTimeframePrefix) }}
						</span>
						<div
							class="flex h-8 shrink-0 items-center overflow-hidden rounded-lg border border-solid border-surface-5 bg-surface-3"
						>
							<button
								type="button"
								class="flex h-8 w-8 cursor-pointer items-center justify-center border-0 border-r border-solid border-surface-5 bg-transparent p-0 text-secondary transition-colors hover:text-contrast"
								:aria-label="formatMessage(messages.decreaseAmount)"
								@click.stop="decrementAmount"
							>
								<MinusIcon class="size-4" />
							</button>
							<input
								v-model="amountInput"
								type="number"
								min="1"
								step="1"
								class="h-8 w-12 border-0 bg-transparent px-1 text-center text-sm font-semibold text-primary outline-none ring-0 focus:outline-none focus-visible:shadow-none"
								:aria-label="formatMessage(messages.timeframeAmount)"
								@focus="activateLastTimeframe"
								@input="handleAmountInput"
								@blur="commitAmountInput"
								@keydown.enter.prevent.stop="submitAmountInput"
							/>
							<button
								type="button"
								class="flex h-8 w-8 cursor-pointer items-center justify-center border-0 border-l border-solid border-surface-5 bg-transparent p-0 text-secondary transition-colors hover:text-contrast"
								:aria-label="formatMessage(messages.increaseAmount)"
								@click.stop="incrementAmount"
							>
								<PlusIcon class="size-4" />
							</button>
						</div>
						<select
							v-model="draftSelectedLastTimeframeUnit"
							class="h-8 rounded-lg border border-solid border-surface-5 bg-surface-3 px-2 text-sm font-semibold text-primary outline-none transition-[box-shadow,color] focus:text-contrast focus:ring-4 focus:ring-brand-shadow"
							:aria-label="formatMessage(messages.timeframeUnit)"
							@change="handleLastTimeframeUnitChange"
						>
							<option
								v-for="option in lastTimeframeUnitOptions"
								:key="option.value"
								:value="option.value"
							>
								{{ option.label }}
							</option>
						</select>
					</div>
				</div>
				<button
					type="button"
					class="flex cursor-pointer items-center border-0 border-t border-solid border-surface-5 bg-transparent px-3 py-3 text-left text-sm font-semibold text-primary transition-colors hover:bg-surface-5"
					@click.stop="switchDraftToCustomDateRange"
				>
					{{ formatMessage(messages.customRange) }}
				</button>
			</div>
		</template>
	</Combobox>
</template>

<script setup lang="ts">
import { MinusIcon, PlusIcon } from '@modrinth/assets'
import { computed, nextTick, ref, watch } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'
import ButtonStyled from './ButtonStyled.vue'
import Combobox, { type ComboboxOption } from './Combobox.vue'
import DatePicker from './DatePicker.vue'

export type TimeFramePreset =
	| 'today'
	| 'yesterday'
	| 'last_7_days'
	| 'last_14_days'
	| 'last_30_days'
	| 'last_90_days'
	| 'last_180_days'
	| 'year_to_date'
	| 'all_time'

export type TimeFrameMode = 'preset' | 'last' | 'custom_range' | 'custom_datetime_range'
export type TimeFrameLastUnit = 'hours' | 'days' | 'weeks' | 'months'

export type TimeFrameLastUnitOption = {
	value: TimeFrameLastUnit
	label: string
}

export type TimeFramePickerSelection = {
	mode: TimeFrameMode
	preset: TimeFramePreset
	lastAmount: number
	lastUnit: TimeFrameLastUnit
	customStartDate: string
	customEndDate: string
}

type DatePickerValue = string | Date | null | undefined
type TimeFramePanel = 'preset' | 'custom_range'
type LastTimeframeValue = {
	amount: number
	unit: TimeFrameLastUnit
}

const TIMEFRAME_DROPDOWN_MAX_HEIGHT = 500
const TIMEFRAME_DROPDOWN_MIN_WIDTH = '20rem'
const CUSTOM_RANGE_DROPDOWN_MIN_WIDTH = '40.5rem'

const DEFAULT_LAST_TIMEFRAME_VALUE_BY_PRESET: Partial<Record<TimeFramePreset, LastTimeframeValue>> =
	{
		today: { amount: 1, unit: 'days' },
		yesterday: { amount: 1, unit: 'days' },
		last_7_days: { amount: 7, unit: 'days' },
		last_14_days: { amount: 14, unit: 'days' },
		last_30_days: { amount: 30, unit: 'days' },
		last_90_days: { amount: 90, unit: 'days' },
		last_180_days: { amount: 180, unit: 'days' },
	}

const messages = defineMessages({
	today: {
		id: 'time-frame-picker.option.today',
		defaultMessage: 'Today',
	},
	yesterday: {
		id: 'time-frame-picker.option.yesterday',
		defaultMessage: 'Yesterday',
	},
	last7Days: {
		id: 'time-frame-picker.option.last-7-days',
		defaultMessage: 'Last 7 days',
	},
	last14Days: {
		id: 'time-frame-picker.option.last-14-days',
		defaultMessage: 'Last 14 days',
	},
	last30Days: {
		id: 'time-frame-picker.option.last-30-days',
		defaultMessage: 'Last 30 days',
	},
	last90Days: {
		id: 'time-frame-picker.option.last-90-days',
		defaultMessage: 'Last 90 days',
	},
	last180Days: {
		id: 'time-frame-picker.option.last-180-days',
		defaultMessage: 'Last 180 days',
	},
	yearToDate: {
		id: 'time-frame-picker.option.year-to-date',
		defaultMessage: 'Year to date',
	},
	allTime: {
		id: 'time-frame-picker.option.all-time',
		defaultMessage: 'All time',
	},
	hours: {
		id: 'time-frame-picker.unit.hours',
		defaultMessage: 'hours',
	},
	days: {
		id: 'time-frame-picker.unit.days',
		defaultMessage: 'days',
	},
	weeks: {
		id: 'time-frame-picker.unit.weeks',
		defaultMessage: 'weeks',
	},
	months: {
		id: 'time-frame-picker.unit.months',
		defaultMessage: 'months',
	},
	lastTimeframe: {
		id: 'time-frame-picker.last-timeframe',
		defaultMessage:
			'In the last {amount} {unit, select, hours {{amount, plural, one {hour} other {hours}}} days {{amount, plural, one {day} other {days}}} weeks {{amount, plural, one {week} other {weeks}}} months {{amount, plural, one {month} other {months}}} other {days}}',
	},
	lastTimeframePrefix: {
		id: 'time-frame-picker.last-timeframe-prefix',
		defaultMessage: 'In the last',
	},
	customRange: {
		id: 'time-frame-picker.custom-range',
		defaultMessage: 'Custom fixed date range...',
	},
	clearRange: {
		id: 'time-frame-picker.clear-range',
		defaultMessage: 'Clear',
	},
	cancel: {
		id: 'time-frame-picker.cancel',
		defaultMessage: 'Cancel',
	},
	apply: {
		id: 'time-frame-picker.apply',
		defaultMessage: 'Apply',
	},
	emptyRange: {
		id: 'time-frame-picker.empty-range',
		defaultMessage: 'No date range selected.',
	},
	selectingRange: {
		id: 'time-frame-picker.selecting-range',
		defaultMessage: 'Selecting',
	},
	selectedRange: {
		id: 'time-frame-picker.selected-range',
		defaultMessage: 'Selected',
	},
	selectTimeframe: {
		id: 'time-frame-picker.select-timeframe',
		defaultMessage: 'Select timeframe',
	},
	decreaseAmount: {
		id: 'time-frame-picker.decrease-amount',
		defaultMessage: 'Decrease timeframe amount',
	},
	increaseAmount: {
		id: 'time-frame-picker.increase-amount',
		defaultMessage: 'Increase timeframe amount',
	},
	timeframeAmount: {
		id: 'time-frame-picker.timeframe-amount',
		defaultMessage: 'Timeframe amount',
	},
	timeframeUnit: {
		id: 'time-frame-picker.timeframe-unit',
		defaultMessage: 'Timeframe unit',
	},
})

const mode = defineModel<TimeFrameMode>('mode', { required: true })
const preset = defineModel<TimeFramePreset>('preset', { required: true })
const lastAmount = defineModel<number>('lastAmount', { required: true })
const lastUnit = defineModel<TimeFrameLastUnit>('lastUnit', { required: true })
const customStartDate = defineModel<string>('customStartDate', { required: true })
const customEndDate = defineModel<string>('customEndDate', { required: true })

const props = withDefaults(
	defineProps<{
		timeframeOptions?: ComboboxOption<TimeFramePreset>[]
		lastTimeframeUnitOptions?: TimeFrameLastUnitOption[]
		lastTimeframeValueByPreset?: Partial<Record<TimeFramePreset, LastTimeframeValue>>
		minDate?: string
		maxDate?: string
		nowTimestamp?: number
		maxHeight?: number
		dropdownMinWidth?: string | number
		customRangeDropdownMinWidth?: string | number
	}>(),
	{
		maxHeight: TIMEFRAME_DROPDOWN_MAX_HEIGHT,
		dropdownMinWidth: TIMEFRAME_DROPDOWN_MIN_WIDTH,
		customRangeDropdownMinWidth: CUSTOM_RANGE_DROPDOWN_MIN_WIDTH,
	},
)

const { formatMessage, locale } = useVIntl()

const emit = defineEmits<{
	open: []
	close: []
	cancel: []
	commit: [selection: TimeFramePickerSelection]
	apply: [selection: TimeFramePickerSelection]
	'draft-change': [selection: TimeFramePickerSelection]
	'preset-select': [option: ComboboxOption<TimeFramePreset>, selection: TimeFramePickerSelection]
}>()

const isTimeframeSelectOpen = ref(false)
const activeTimeframePanel = ref<TimeFramePanel>('preset')
const draftSelectedTimeframeMode = ref<TimeFrameMode>(mode.value)
const draftSelectedTimeframe = ref<TimeFramePreset>(preset.value)
const draftSelectedLastTimeframeAmount = ref(lastAmount.value)
const draftSelectedLastTimeframeUnit = ref<TimeFrameLastUnit>(lastUnit.value)
const draftSelectedCustomTimeframeStartDate = ref(customStartDate.value)
const draftSelectedCustomTimeframeEndDate = ref(customEndDate.value)
const amountInput = ref(String(lastAmount.value))
const pickerRange = ref<DatePickerValue[]>([customStartDate.value, customEndDate.value])

const timeframeOptions = computed<ComboboxOption<TimeFramePreset>[]>(
	() =>
		props.timeframeOptions ?? [
			{ value: 'today', label: formatMessage(messages.today) },
			{ value: 'yesterday', label: formatMessage(messages.yesterday) },
			{ value: 'last_7_days', label: formatMessage(messages.last7Days) },
			{ value: 'last_14_days', label: formatMessage(messages.last14Days) },
			{ value: 'last_30_days', label: formatMessage(messages.last30Days) },
			{ value: 'last_90_days', label: formatMessage(messages.last90Days) },
			{ value: 'last_180_days', label: formatMessage(messages.last180Days) },
			{ value: 'year_to_date', label: formatMessage(messages.yearToDate) },
			{ value: 'all_time', label: formatMessage(messages.allTime) },
		],
)
const lastTimeframeUnitOptions = computed<TimeFrameLastUnitOption[]>(
	() =>
		props.lastTimeframeUnitOptions ?? [
			{ value: 'hours', label: formatMessage(messages.hours) },
			{ value: 'days', label: formatMessage(messages.days) },
			{ value: 'weeks', label: formatMessage(messages.weeks) },
			{ value: 'months', label: formatMessage(messages.months) },
		],
)
const lastTimeframeValueByPreset = computed(
	() => props.lastTimeframeValueByPreset ?? DEFAULT_LAST_TIMEFRAME_VALUE_BY_PRESET,
)
const timeframeDropdownOptions = computed<ComboboxOption<TimeFramePreset>[]>(() =>
	activeTimeframePanel.value === 'custom_range' ? [] : timeframeOptions.value,
)
const timeframeDropdownMinWidth = computed(() =>
	activeTimeframePanel.value === 'custom_range'
		? props.customRangeDropdownMinWidth
		: props.dropdownMinWidth,
)
const highlightedTimeframePreset = computed<TimeFramePreset | undefined>(() =>
	draftSelectedTimeframeMode.value === 'preset' ? draftSelectedTimeframe.value : undefined,
)
const selectedTimeframeLabel = computed(() => {
	const useDraftTimeframeLabel =
		isTimeframeSelectOpen.value && activeTimeframePanel.value !== 'custom_range'

	return getTimeframeLabel(
		useDraftTimeframeLabel ? draftSelectedTimeframeMode.value : mode.value,
		useDraftTimeframeLabel ? draftSelectedTimeframe.value : preset.value,
		useDraftTimeframeLabel ? draftSelectedLastTimeframeAmount.value : lastAmount.value,
		useDraftTimeframeLabel ? draftSelectedLastTimeframeUnit.value : lastUnit.value,
		useDraftTimeframeLabel ? draftSelectedCustomTimeframeStartDate.value : customStartDate.value,
		useDraftTimeframeLabel ? draftSelectedCustomTimeframeEndDate.value : customEndDate.value,
	)
})
const todayInputValue = computed(() => getDateInputValue(new Date()))
const customRangeMaxDate = computed(() => props.maxDate ?? todayInputValue.value)
const selectedDraftDates = computed(() =>
	pickerRange.value
		.map(getDatePickerValueString)
		.filter((value): value is string => Boolean(value)),
)
const rangeLabel = computed(() =>
	formatMessage(
		selectedDraftDates.value.length === 1 ? messages.selectingRange : messages.selectedRange,
	),
)
const hasCompleteRange = computed(() => Boolean(getOrderedRange(pickerRange.value)))
const formattedRange = computed(() => {
	if (selectedDraftDates.value.length === 1) {
		return `${formatDateString(selectedDraftDates.value[0])} -`
	}

	const orderedRange =
		getOrderedRange(pickerRange.value) ??
		getOrderedRange([
			draftSelectedCustomTimeframeStartDate.value,
			draftSelectedCustomTimeframeEndDate.value,
		])
	if (!orderedRange) return ''

	const [nextStartDate, nextEndDate] = orderedRange
	if (nextStartDate === nextEndDate) return formatDateString(nextStartDate)
	return `${formatDateString(nextStartDate)} - ${formatDateString(nextEndDate)}`
})

function getTimeframeLabel(
	selectedMode: TimeFrameMode,
	selectedPreset: TimeFramePreset,
	selectedLastAmount: number,
	selectedLastUnit: TimeFrameLastUnit,
	selectedCustomStartDate: string,
	selectedCustomEndDate: string,
): string {
	if (selectedMode === 'last') {
		return formatMessage(messages.lastTimeframe, {
			amount: selectedLastAmount,
			unit: selectedLastUnit,
		})
	}

	if (selectedMode === 'custom_range') {
		return formatCustomTimeframeRangeLabel(selectedCustomStartDate, selectedCustomEndDate)
	}

	if (selectedMode === 'custom_datetime_range') {
		return formatCustomDateTimeRangeLabel(selectedCustomStartDate, selectedCustomEndDate)
	}

	return (
		timeframeOptions.value.find((option) => option.value === selectedPreset)?.label ??
		formatMessage(messages.selectTimeframe)
	)
}

function getDraftSelection(): TimeFramePickerSelection {
	return {
		mode: draftSelectedTimeframeMode.value,
		preset: draftSelectedTimeframe.value,
		lastAmount: draftSelectedLastTimeframeAmount.value,
		lastUnit: draftSelectedLastTimeframeUnit.value,
		customStartDate: draftSelectedCustomTimeframeStartDate.value,
		customEndDate: draftSelectedCustomTimeframeEndDate.value,
	}
}

function emitDraftChange() {
	emit('draft-change', getDraftSelection())
}

function getDateInputValue(date: Date): string {
	const year = date.getFullYear()
	const month = String(date.getMonth() + 1).padStart(2, '0')
	const day = String(date.getDate()).padStart(2, '0')
	return `${year}-${month}-${day}`
}

function getDateFromInputValue(value: string): Date | undefined {
	const date = new Date(`${value}T00:00:00`)
	if (Number.isNaN(date.getTime()) || getDateInputValue(date) !== value) {
		return undefined
	}

	return date
}

function getDateTimeFromInputValue(value: string): Date | undefined {
	const date = new Date(value)
	if (Number.isNaN(date.getTime())) {
		return undefined
	}

	return date
}

function formatDateString(value: string): string {
	const parsed = new Date(`${value}T00:00:00`)
	if (Number.isNaN(parsed.getTime())) return value
	return formatDate(parsed, {
		month: 'short',
		day: 'numeric',
		year: 'numeric',
	})
}

function isValidDateInputValue(value: string): boolean {
	if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) {
		return false
	}

	const parsedDate = new Date(`${value}T00:00:00`)
	return !Number.isNaN(parsedDate.getTime()) && getDateInputValue(parsedDate) === value
}

function getDatePickerValueString(value: DatePickerValue): string | null {
	if (typeof value === 'string') {
		return isValidDateInputValue(value) ? value : null
	}
	if (value instanceof Date && !Number.isNaN(value.getTime())) {
		return getDateInputValue(value)
	}

	return null
}

function getOrderedRange(values: DatePickerValue[]): [string, string] | null {
	const dates = values
		.map(getDatePickerValueString)
		.filter((value): value is string => Boolean(value))
	if (dates.length < 2) {
		return null
	}

	const firstDate = dates[0]
	const secondDate = dates[1]
	if (!firstDate || !secondDate) {
		return null
	}

	return firstDate <= secondDate ? [firstDate, secondDate] : [secondDate, firstDate]
}

function formatCustomTimeframeRangeLabel(startDateValue: string, endDateValue: string): string {
	const startDate = getDateFromInputValue(startDateValue)
	const endDate = getDateFromInputValue(endDateValue)
	if (!startDate || !endDate) {
		return `${startDateValue} - ${endDateValue}`
	}

	if (startDateValue === endDateValue) {
		return formatDate(startDate, {
			month: 'long',
			day: 'numeric',
			year: 'numeric',
		})
	}

	const sameYear = startDate.getFullYear() === endDate.getFullYear()
	if (sameYear) {
		const startLabel = formatDate(startDate, { month: 'long', day: 'numeric' })
		const endLabel = formatDate(endDate, { month: 'long', day: 'numeric' })
		return `${startLabel} - ${endLabel}, ${startDate.getFullYear()}`
	}

	const startLabel = formatDate(startDate, {
		month: 'long',
		day: 'numeric',
		year: 'numeric',
	})
	const endLabel = formatDate(endDate, {
		month: 'long',
		day: 'numeric',
		year: 'numeric',
	})
	return `${startLabel} - ${endLabel}`
}

function formatCustomDateTimeRangeLabel(startDateValue: string, endDateValue: string): string {
	const startDate = getDateTimeFromInputValue(startDateValue)
	const endDate = getDateTimeFromInputValue(endDateValue)
	if (!startDate || !endDate) {
		return `${startDateValue} - ${endDateValue}`
	}

	if (startDate.getTime() === endDate.getTime()) {
		return formatDate(startDate, {
			month: 'short',
			day: 'numeric',
			year: 'numeric',
			hour: 'numeric',
			minute: '2-digit',
		})
	}

	const sameYear = startDate.getFullYear() === endDate.getFullYear()
	if (sameYear) {
		const startLabel = formatDate(startDate, {
			month: 'short',
			day: 'numeric',
			hour: 'numeric',
			minute: '2-digit',
		})
		const endLabel = formatDate(endDate, {
			month: 'short',
			day: 'numeric',
			hour: 'numeric',
			minute: '2-digit',
		})
		return `${startLabel} - ${endLabel}, ${startDate.getFullYear()}`
	}

	const startLabel = formatDate(startDate, {
		month: 'short',
		day: 'numeric',
		year: 'numeric',
		hour: 'numeric',
		minute: '2-digit',
	})
	const endLabel = formatDate(endDate, {
		month: 'short',
		day: 'numeric',
		year: 'numeric',
		hour: 'numeric',
		minute: '2-digit',
	})
	return `${startLabel} - ${endLabel}`
}

function formatDate(date: Date, options: Intl.DateTimeFormatOptions): string {
	return new Intl.DateTimeFormat(locale.value, options).format(date)
}

function getRoundedNow(timestamp: number): Date {
	const roundedTimestamp = Math.floor(timestamp / 60000) * 60000
	return new Date(roundedTimestamp)
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

function getTimeRangeForLastTimeframe() {
	const end = getRoundedNow(props.nowTimestamp ?? Date.now())
	const amount = Math.max(1, Math.floor(draftSelectedLastTimeframeAmount.value))

	switch (draftSelectedLastTimeframeUnit.value) {
		case 'hours':
			return { start: new Date(end.getTime() - amount * 60 * 60 * 1000), end }
		case 'days':
			return { start: new Date(end.getTime() - amount * 24 * 60 * 60 * 1000), end }
		case 'weeks':
			return { start: new Date(end.getTime() - amount * 7 * 24 * 60 * 60 * 1000), end }
		case 'months':
			return { start: subtractCalendarMonths(end, amount), end }
	}
}

function getDraftTimeRange() {
	if (draftSelectedTimeframeMode.value === 'last') {
		return getTimeRangeForLastTimeframe()
	}

	const startDate =
		draftSelectedTimeframeMode.value === 'custom_datetime_range'
			? getDateTimeFromInputValue(draftSelectedCustomTimeframeStartDate.value)
			: getDateFromInputValue(draftSelectedCustomTimeframeStartDate.value)
	const endDate =
		draftSelectedTimeframeMode.value === 'custom_datetime_range'
			? getDateTimeFromInputValue(draftSelectedCustomTimeframeEndDate.value)
			: getDateFromInputValue(draftSelectedCustomTimeframeEndDate.value)

	if (!startDate || !endDate) {
		return null
	}

	return {
		start: startDate,
		end: draftSelectedTimeframeMode.value === 'custom_range' ? addDays(endDate, 1) : endDate,
	}
}

function resetTimeframeDraft() {
	draftSelectedTimeframeMode.value = mode.value
	draftSelectedTimeframe.value = preset.value
	draftSelectedLastTimeframeAmount.value = lastAmount.value
	draftSelectedLastTimeframeUnit.value = lastUnit.value
	draftSelectedCustomTimeframeStartDate.value = customStartDate.value
	draftSelectedCustomTimeframeEndDate.value = customEndDate.value
	amountInput.value = String(lastAmount.value)
	syncPickerRangeFromDraft()
}

function commitTimeframeDraft() {
	const selection = getDraftSelection()

	mode.value = selection.mode
	preset.value = selection.preset
	lastAmount.value = selection.lastAmount
	lastUnit.value = selection.lastUnit
	customStartDate.value = selection.customStartDate
	customEndDate.value = selection.customEndDate
	emit('commit', selection)
}

function handleTimeframeSelectOpen() {
	resetTimeframeDraft()
	activeTimeframePanel.value = 'preset'
	isTimeframeSelectOpen.value = true
	emit('open')
}

function handleTimeframeSelectClose() {
	if (activeTimeframePanel.value !== 'custom_range') {
		commitTimeframeDraft()
	}
	isTimeframeSelectOpen.value = false
	emit('close')
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

async function applyTimeframeDraft(event: Event) {
	commitTimeframeDraft()
	closeTimeframeSelectDropdown(event)
	await nextTick()
	emit('apply', getDraftSelection())
}

function handleCustomRangeCancel() {
	resetTimeframeDraft()
	activeTimeframePanel.value = 'preset'
	emit('cancel')
	emitDraftChange()
}

async function handleCustomRangeApply(event: MouseEvent) {
	if (!hasCompleteDraftCustomDateRange()) {
		return
	}

	draftSelectedTimeframeMode.value = 'custom_range'
	await applyTimeframeDraft(event)
}

function handleTimeframeModelUpdate(value: TimeFramePreset) {
	draftSelectedTimeframe.value = value
	emitDraftChange()
}

function handleTimeframePresetSelect(option: ComboboxOption<TimeFramePreset>) {
	draftSelectedTimeframeMode.value = 'preset'

	const lastTimeframeValue = lastTimeframeValueByPreset.value[option.value]
	if (lastTimeframeValue) {
		draftSelectedLastTimeframeAmount.value = lastTimeframeValue.amount
		draftSelectedLastTimeframeUnit.value = lastTimeframeValue.unit
		amountInput.value = String(lastTimeframeValue.amount)
	}

	const selection = getDraftSelection()
	emit('preset-select', option, selection)
	emit('draft-change', selection)
}

function hasCompleteDraftCustomDateRange() {
	return Boolean(
		getDateFromInputValue(draftSelectedCustomTimeframeStartDate.value) &&
		getDateFromInputValue(draftSelectedCustomTimeframeEndDate.value),
	)
}

function switchDraftToCustomDateRange() {
	if (draftSelectedTimeframeMode.value === 'preset') {
		draftSelectedCustomTimeframeStartDate.value = ''
		draftSelectedCustomTimeframeEndDate.value = ''
	} else {
		const rawRange = getDraftTimeRange()
		draftSelectedCustomTimeframeStartDate.value = rawRange ? getDateInputValue(rawRange.start) : ''
		draftSelectedCustomTimeframeEndDate.value = rawRange
			? getInclusiveEndDateInputValue(rawRange.end)
			: ''
	}

	draftSelectedTimeframeMode.value = 'custom_range'
	activeTimeframePanel.value = 'custom_range'
	syncPickerRangeFromDraft()
	emitDraftChange()
}

function activateLastTimeframe() {
	draftSelectedTimeframeMode.value = 'last'
	emitDraftChange()
}

function parseAmountInput() {
	const nextAmount = Number(amountInput.value)
	return Number.isFinite(nextAmount) ? Math.max(1, Math.floor(nextAmount)) : null
}

function handleAmountInput() {
	const nextAmount = parseAmountInput()
	if (nextAmount !== null && String(nextAmount) === amountInput.value) {
		draftSelectedLastTimeframeAmount.value = nextAmount
	}

	activateLastTimeframe()
}

function commitAmountInput() {
	const nextAmount = parseAmountInput() ?? 1
	draftSelectedLastTimeframeAmount.value = nextAmount
	amountInput.value = String(nextAmount)
	activateLastTimeframe()
}

function submitAmountInput(event: KeyboardEvent) {
	commitAmountInput()
	void applyTimeframeDraft(event)
}

function incrementAmount() {
	commitAmountInput()
	draftSelectedLastTimeframeAmount.value += 1
	amountInput.value = String(draftSelectedLastTimeframeAmount.value)
	activateLastTimeframe()
}

function decrementAmount() {
	commitAmountInput()
	draftSelectedLastTimeframeAmount.value = Math.max(1, draftSelectedLastTimeframeAmount.value - 1)
	amountInput.value = String(draftSelectedLastTimeframeAmount.value)
	activateLastTimeframe()
}

function handleLastTimeframeUnitChange() {
	activateLastTimeframe()
}

function clearRange() {
	draftSelectedCustomTimeframeStartDate.value = ''
	draftSelectedCustomTimeframeEndDate.value = ''
	pickerRange.value = []
	emitDraftChange()
}

function syncPickerRangeFromDraft() {
	if (
		pickerRange.value.length === 2 &&
		pickerRange.value[0] === draftSelectedCustomTimeframeStartDate.value &&
		pickerRange.value[1] === draftSelectedCustomTimeframeEndDate.value
	) {
		return
	}

	pickerRange.value = [
		draftSelectedCustomTimeframeStartDate.value,
		draftSelectedCustomTimeframeEndDate.value,
	]
}

watch([mode, preset, lastAmount, lastUnit, customStartDate, customEndDate], () => {
	if (isTimeframeSelectOpen.value) {
		return
	}

	resetTimeframeDraft()
})

watch(
	[draftSelectedCustomTimeframeStartDate, draftSelectedCustomTimeframeEndDate],
	syncPickerRangeFromDraft,
)

watch(pickerRange, (nextRange) => {
	const orderedRange = getOrderedRange(nextRange)
	if (!orderedRange) {
		return
	}

	const [nextStartDate, nextEndDate] = orderedRange
	draftSelectedCustomTimeframeStartDate.value = nextStartDate
	draftSelectedCustomTimeframeEndDate.value = nextEndDate
	emitDraftChange()
})
</script>
