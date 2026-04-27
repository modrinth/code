<template>
	<div
		class="modrinth-date-picker relative inline-flex items-center"
		:class="[
			wrapperClass,
			disabled ? 'cursor-not-allowed opacity-50' : '',
			showToday ? 'show-today' : '',
		]"
	>
		<CalendarIcon
			v-if="showIcon"
			class="pointer-events-none absolute left-3 z-[1] h-5 w-5 text-secondary opacity-60 transition-colors"
			aria-hidden="true"
		/>
		<input
			:id="id"
			ref="inputRef"
			:name="name"
			:placeholder="placeholder"
			:disabled="disabled"
			:readonly="readonly"
			:autocomplete="autocomplete"
			:class="inputClasses"
			type="text"
		/>
	</div>
</template>

<script setup lang="ts">
import { CalendarIcon } from '@modrinth/assets'
import flatpickr from 'flatpickr'
import 'flatpickr/dist/flatpickr.css'
import type { Instance } from 'flatpickr/dist/types/instance'
import type { Options } from 'flatpickr/dist/types/options'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

type DatePickerValue = string | Date | null | undefined

const model = defineModel<DatePickerValue | DatePickerValue[]>()

const props = withDefaults(
	defineProps<{
		id?: string
		name?: string
		placeholder?: string
		autocomplete?: string
		disabled?: boolean
		readonly?: boolean
		enableTime?: boolean
		mode?: 'single' | 'multiple' | 'range'
		minDate?: string | Date
		maxDate?: string | Date
		/**
		 * The stored value format emitted by v-model. See https://flatpickr.js.org/formatting/
		 *
		 * Examples:
		 * - `Y-m-d` renders as `2026-04-27`
		 * - `Y-m-d H:i` renders as `2026-04-27 14:30`
		 */
		dateFormat?: string
		/**
		 * The human-friendly format shown in the visible input. See https://flatpickr.js.org/formatting/
		 *
		 * Examples:
		 * - `F j, Y` renders as `April 27, 2026`
		 * - `F j, Y at h:i K` renders as `April 27, 2026 at 02:30 PM`
		 */
		altFormat?: string
		time24hr?: boolean
		clearable?: boolean
		showIcon?: boolean
		showToday?: boolean
		wrapperClass?: string
		inputClass?: string
	}>(),
	{
		disabled: false,
		readonly: false,
		enableTime: false,
		mode: 'single',
		time24hr: false,
		clearable: true,
		showIcon: true,
		showToday: false,
	},
)

const emit = defineEmits<{
	change: [value: DatePickerValue | DatePickerValue[]]
	clear: []
}>()

const inputRef = ref<HTMLInputElement>()
const picker = ref<Instance>()
const isSyncingFromModel = ref(false)
const intendedViewMonth = ref<{ month: number; year: number } | null>(null)

const resolvedDateFormat = computed(
	() => props.dateFormat ?? (props.enableTime ? 'Y-m-d H:i' : 'Y-m-d'),
)
const resolvedAltFormat = computed(
	() => props.altFormat ?? (props.enableTime ? 'F j, Y at h:i K' : 'F j, Y'),
)

const inputClasses = computed(() => [
	'w-full text-primary placeholder:text-secondary focus:text-contrast font-medium transition-[shadow,color] appearance-none shadow-none focus:ring-4 focus:ring-brand-shadow !outline-0',
	props.showIcon ? 'pl-10' : 'pl-3',
	'pr-3 h-9 py-2 text-base outline-none bg-surface-4 border-none rounded-xl',
	props.disabled ? 'cursor-not-allowed' : '',
	props.inputClass,
])

const selectedDates = computed(() => {
	const value = model.value
	if (Array.isArray(value)) {
		return value.filter(Boolean) as Array<string | Date>
	}

	return value ? [value] : []
})

watch(
	() => [
		model.value,
		props.disabled,
		props.readonly,
		props.minDate,
		props.maxDate,
		props.enableTime,
		props.mode,
		props.dateFormat,
		props.altFormat,
		props.time24hr,
	],
	() => {
		if (!picker.value) return

		picker.value.set({
			...flatpickrOptions(),
			onChange: picker.value.config.onChange,
			onClose: picker.value.config.onClose,
			onReady: picker.value.config.onReady,
		})

		syncAltInputState()
		syncPickerFromModel()
	},
	{ deep: true },
)

onMounted(async () => {
	await nextTick()
	if (!inputRef.value) return

	picker.value = flatpickr(inputRef.value, {
		...flatpickrOptions(),
		onReady: (_selectedDates, _dateStr, instance) => {
			instance.calendarContainer.addEventListener('mousedown', (event) => {
				if (props.mode !== 'range') return
				const target = event.target as HTMLElement | null
				const dayElem = target?.closest('.flatpickr-day')
				if (!dayElem) return
				if (
					!dayElem.classList.contains('prevMonthDay') &&
					!dayElem.classList.contains('nextMonthDay')
				) {
					return
				}
				intendedViewMonth.value = {
					month: instance.currentMonth,
					year: instance.currentYear,
				}
			})
		},
		onChange: (_selectedDates, dateStr, instance) => {
			if (isSyncingFromModel.value) return

			const nextValue =
				props.mode === 'single'
					? dateStr || null
					: _selectedDates.map((date) => instance.formatDate(date, resolvedDateFormat.value))
			model.value = nextValue
			emit('change', nextValue)

			if (intendedViewMonth.value !== null) {
				const monthDelta =
					intendedViewMonth.value.month -
					instance.currentMonth +
					(intendedViewMonth.value.year - instance.currentYear) * 12
				if (monthDelta !== 0) instance.changeMonth(monthDelta)
			}
		},
		onClose: (_selectedDates, dateStr, instance) => {
			if (!props.clearable || dateStr) return

			const nextValue = props.mode === 'single' ? null : []
			model.value = nextValue
			instance.clear(false)
			emit('clear')
			emit('change', nextValue)
		},
	})

	syncAltInputState()
	syncPickerFromModel()
})

onBeforeUnmount(() => {
	picker.value?.destroy()
})

function flatpickrOptions(): Options {
	return {
		allowInput: true,
		altInput: true,
		altInputClass: inputClasses.value.filter(Boolean).join(' '),
		altFormat: resolvedAltFormat.value,
		appendTo: inputRef.value?.parentElement ?? undefined,
		closeOnSelect: false,
		dateFormat: resolvedDateFormat.value,
		disableMobile: true,
		enableTime: props.enableTime,
		maxDate: props.maxDate,
		minDate: props.minDate,
		mode: props.mode,
		noCalendar: false,
		static: true,
		time_24hr: props.time24hr,
	}
}

function syncPickerFromModel() {
	if (!picker.value) return

	isSyncingFromModel.value = true
	picker.value.setDate(selectedDates.value, false, resolvedDateFormat.value)

	if (intendedViewMonth.value !== null) {
		const monthDelta =
			intendedViewMonth.value.month -
			picker.value.currentMonth +
			(intendedViewMonth.value.year - picker.value.currentYear) * 12
		if (monthDelta !== 0) picker.value.changeMonth(monthDelta)
		intendedViewMonth.value = null
	}

	isSyncingFromModel.value = false
}

function syncAltInputState() {
	if (!picker.value?.altInput) return

	picker.value.altInput.disabled = props.disabled
	picker.value.altInput.readOnly = props.readonly
}

defineExpose({
	focus: () => picker.value?.altInput?.focus() ?? inputRef.value?.focus(),
	open: () => picker.value?.open(),
	close: () => picker.value?.close(),
	clear: () => {
		const nextValue = props.mode === 'single' ? null : []
		model.value = nextValue
		picker.value?.clear(false)
		emit('clear')
		emit('change', nextValue)
	},
})
</script>

<style scoped>
.modrinth-date-picker :deep(.flatpickr-wrapper) {
	@apply w-full;
}

.modrinth-date-picker :deep(.flatpickr-calendar) {
	@apply mt-2 overflow-hidden rounded-[14px] border border-solid border-surface-5 bg-surface-3 p-2 text-primary shadow-2xl;
	box-sizing: content-box;
}

.modrinth-date-picker :deep(.flatpickr-innerContainer),
.modrinth-date-picker :deep(.flatpickr-rContainer),
.modrinth-date-picker :deep(.flatpickr-days),
.modrinth-date-picker :deep(.dayContainer) {
	@apply max-w-[307.875px] min-w-[307.875px] w-[307.875px];
}

.modrinth-date-picker :deep(.flatpickr-calendar::before),
.modrinth-date-picker :deep(.flatpickr-calendar::after) {
	display: none;
}

.modrinth-date-picker :deep(.flatpickr-months),
.modrinth-date-picker :deep(.flatpickr-month) {
	@apply items-center h-10 flex;
}

.modrinth-date-picker :deep(.flatpickr-month),
.modrinth-date-picker :deep(.flatpickr-current-month),
.modrinth-date-picker :deep(.flatpickr-weekdays),
.modrinth-date-picker :deep(.flatpickr-weekdaycontainer) {
	@apply bg-surface-3 text-contrast;
}

.modrinth-date-picker :deep(.flatpickr-current-month) {
	@apply flex h-full items-center justify-center gap-2 py-0 text-base font-semibold;
}

.modrinth-date-picker :deep(.flatpickr-current-month input.cur-year),
.modrinth-date-picker :deep(.flatpickr-current-month .flatpickr-monthDropdown-months) {
	@apply rounded-lg bg-surface-4 px-2 py-1 font-semibold text-contrast hover:bg-surface-5 min-h-8;
}

.modrinth-date-picker :deep(.flatpickr-current-month .flatpickr-monthDropdown-months) {
	@apply pr-4;
}

.modrinth-date-picker :deep(.flatpickr-current-month input.cur-year:focus),
.modrinth-date-picker :deep(.flatpickr-current-month .flatpickr-monthDropdown-months:focus) {
	@apply outline-none ring-4 ring-brand-shadow;
}

.modrinth-date-picker :deep(.flatpickr-prev-month),
.modrinth-date-picker :deep(.flatpickr-next-month) {
	@apply top-3 mx-3 flex h-8 w-8 items-center justify-center rounded-lg p-0 text-secondary hover:bg-surface-4 hover:text-contrast;
}

.modrinth-date-picker :deep(.flatpickr-prev-month svg),
.modrinth-date-picker :deep(.flatpickr-next-month svg) {
	@apply h-4 w-4 fill-current;
}

.modrinth-date-picker :deep(.flatpickr-weekday) {
	@apply text-xs font-semibold text-secondary;
}

.modrinth-date-picker :deep(.flatpickr-day) {
	@apply m-0 max-w-none rounded-lg border border-solid border-transparent text-primary hover:border-surface-5 hover:bg-surface-4 hover:text-contrast;
}

.modrinth-date-picker :deep(.flatpickr-day.today) {
	@apply border-transparent;
}

.modrinth-date-picker.show-today :deep(.flatpickr-day.today) {
	@apply border-brand text-contrast;
}

.modrinth-date-picker :deep(.flatpickr-day.selected),
.modrinth-date-picker :deep(.flatpickr-day.startRange),
.modrinth-date-picker :deep(.flatpickr-day.endRange) {
	@apply border-brand bg-brand text-brand-inverted !shadow-none hover:border-brand hover:bg-brand hover:text-brand-inverted hover:shadow-none;
}

.modrinth-date-picker :deep(.flatpickr-day.inRange) {
	@apply rounded-none border-x-0 border-y-surface-3 bg-brand-highlight text-contrast shadow-none hover:rounded-none hover:bg-brand-highlight;
}

.modrinth-date-picker :deep(.flatpickr-day.startRange:not(.endRange)) {
	@apply rounded-r-none border-r-0 border-y-surface-3;
}

.modrinth-date-picker :deep(.flatpickr-day.endRange:not(.startRange)) {
	@apply rounded-l-none border-l-0 border-y-surface-3;
}

.modrinth-date-picker
	:deep(.flatpickr-day.prevMonthDay:not(.inRange):not(.startRange):not(.endRange)),
.modrinth-date-picker
	:deep(.flatpickr-day.nextMonthDay:not(.inRange):not(.startRange):not(.endRange)),
.modrinth-date-picker :deep(.flatpickr-day.flatpickr-disabled) {
	@apply text-secondary opacity-40;
}

.modrinth-date-picker :deep(.flatpickr-time) {
	@apply mt-2 flex h-11 max-h-none items-center gap-2 border-0 border-t border-solid border-surface-5 px-1 pt-2 leading-none;
}

.modrinth-date-picker :deep(.flatpickr-time .numInputWrapper) {
	@apply h-full flex-1 rounded-lg bg-surface-4;
}

.modrinth-date-picker :deep(.flatpickr-time input),
.modrinth-date-picker :deep(.flatpickr-time .flatpickr-am-pm) {
	@apply h-full rounded-lg bg-transparent px-2 text-center font-semibold text-primary hover:bg-surface-5 focus:bg-surface-5;
}

.modrinth-date-picker :deep(.flatpickr-time .flatpickr-time-separator) {
	@apply flex h-full items-center text-secondary;
}

.modrinth-date-picker :deep(.flatpickr-time .flatpickr-am-pm) {
	@apply flex w-14 items-center justify-center bg-surface-4 px-3;
}

.modrinth-date-picker :deep(.flatpickr-time .numInputWrapper span.arrowUp::after) {
	border-bottom-color: var(--color-text-tertiary);
}

.modrinth-date-picker :deep(.flatpickr-time .numInputWrapper span.arrowDown::after) {
	border-top-color: var(--color-text-tertiary);
}
</style>
