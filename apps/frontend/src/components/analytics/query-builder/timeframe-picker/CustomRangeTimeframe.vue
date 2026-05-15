<template>
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
			:max-date="todayInputValue"
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
							Clear
						</button>
					</div>
				</template>
				<template v-else>
					<span class="font-normal text-primary">No date range selected.</span>
				</template>
			</div>

			<div class="flex items-center gap-2">
				<ButtonStyled type="outlined">
					<button type="button" @click="$emit('cancel', $event)">Cancel</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button type="button" @click="$emit('apply', $event)">Apply</button>
				</ButtonStyled>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ButtonStyled, DatePicker } from '@modrinth/ui'

type DatePickerValue = string | Date | null | undefined

defineEmits<{
	cancel: [event: MouseEvent]
	apply: [event: MouseEvent]
}>()

defineProps<{
	minDate: string
}>()

const startDate = defineModel<string>('startDate', { required: true })
const endDate = defineModel<string>('endDate', { required: true })
const pickerRange = ref<DatePickerValue[]>([startDate.value, endDate.value])

const todayInputValue = computed(() => getDateInputValue(new Date()))

const rangeFormatter = new Intl.DateTimeFormat(undefined, {
	month: 'short',
	day: 'numeric',
	year: 'numeric',
})

function formatDateString(value: string): string {
	const parsed = new Date(`${value}T00:00:00`)
	if (Number.isNaN(parsed.getTime())) return value
	return rangeFormatter.format(parsed)
}

function getDateInputValue(date: Date): string {
	const year = date.getFullYear()
	const month = String(date.getMonth() + 1).padStart(2, '0')
	const day = String(date.getDate()).padStart(2, '0')
	return `${year}-${month}-${day}`
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

const selectedDraftDates = computed(() =>
	pickerRange.value
		.map(getDatePickerValueString)
		.filter((value): value is string => Boolean(value)),
)

const rangeLabel = computed(() =>
	selectedDraftDates.value.length === 1 ? 'Selecting' : 'Selected',
)

const formattedRange = computed(() => {
	if (selectedDraftDates.value.length === 1)
		return `${formatDateString(selectedDraftDates.value[0])} –`

	const orderedRange =
		getOrderedRange(pickerRange.value) ?? getOrderedRange([startDate.value, endDate.value])
	if (!orderedRange) return ''

	const [nextStartDate, nextEndDate] = orderedRange
	if (nextStartDate === nextEndDate) return formatDateString(nextStartDate)
	return `${formatDateString(nextStartDate)} – ${formatDateString(nextEndDate)}`
})

function clearRange() {
	startDate.value = ''
	endDate.value = ''
	pickerRange.value = []
}

function syncPickerRangeFromModels() {
	if (
		pickerRange.value.length === 2 &&
		pickerRange.value[0] === startDate.value &&
		pickerRange.value[1] === endDate.value
	) {
		return
	}

	pickerRange.value = [startDate.value, endDate.value]
}

watch([startDate, endDate], syncPickerRangeFromModels)

watch(pickerRange, (nextRange) => {
	const orderedRange = getOrderedRange(nextRange)
	if (!orderedRange) {
		return
	}

	const [nextStartDate, nextEndDate] = orderedRange
	startDate.value = nextStartDate
	endDate.value = nextEndDate
})
</script>
