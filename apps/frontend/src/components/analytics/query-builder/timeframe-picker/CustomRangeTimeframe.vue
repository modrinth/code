<template>
	<div class="flex flex-col gap-2 p-3">
		<label class="flex items-center justify-between gap-3 text-sm font-semibold text-primary">
			<span>Start</span>
			<input
				v-model="startDate"
				type="date"
				class="h-8 rounded-lg border border-solid border-surface-5 bg-surface-3 px-2 text-sm font-semibold text-primary outline-none transition-[box-shadow,color] focus:text-contrast focus:ring-4 focus:ring-brand-shadow"
				@change="handleStartDateChange"
			/>
		</label>
		<label class="flex items-center justify-between gap-3 text-sm font-semibold text-primary">
			<span>End</span>
			<input
				v-model="endDate"
				type="date"
				class="h-8 rounded-lg border border-solid border-surface-5 bg-surface-3 px-2 text-sm font-semibold text-primary outline-none transition-[box-shadow,color] focus:text-contrast focus:ring-4 focus:ring-brand-shadow"
				@change="handleEndDateChange"
			/>
		</label>
	</div>
</template>

<script setup lang="ts">
const startDate = defineModel<string>('startDate', { required: true })
const endDate = defineModel<string>('endDate', { required: true })

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

function handleStartDateChange() {
	if (!isValidDateInputValue(startDate.value)) {
		startDate.value = isValidDateInputValue(endDate.value)
			? endDate.value
			: getDateInputValue(new Date())
	}
	if (!isValidDateInputValue(endDate.value)) {
		endDate.value = startDate.value
	}
	if (startDate.value > endDate.value) {
		endDate.value = startDate.value
	}
}

function handleEndDateChange() {
	if (!isValidDateInputValue(endDate.value)) {
		endDate.value = isValidDateInputValue(startDate.value)
			? startDate.value
			: getDateInputValue(new Date())
	}
	if (!isValidDateInputValue(startDate.value)) {
		startDate.value = endDate.value
	}
	if (endDate.value < startDate.value) {
		startDate.value = endDate.value
	}
}
</script>
