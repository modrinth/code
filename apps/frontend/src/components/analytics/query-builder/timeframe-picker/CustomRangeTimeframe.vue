<template>
	<DatePicker
		v-model="pickerRange"
		mode="range"
		:show-months="2"
		:clearable="false"
		:default-view-date="startDate"
		calendar-only
		wrapper-class="w-full"
	/>
</template>

<script setup lang="ts">
import { DatePicker } from '@modrinth/ui'

type DatePickerValue = string | Date | null | undefined

const startDate = defineModel<string>('startDate', { required: true })
const endDate = defineModel<string>('endDate', { required: true })
const pickerRange = ref<DatePickerValue[]>([startDate.value, endDate.value])

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
