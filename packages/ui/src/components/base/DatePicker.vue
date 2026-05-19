<template>
	<div
		ref="wrapperRef"
		class="modrinth-date-picker relative inline-flex items-center"
		:class="[
			wrapperClass,
			calendarOnly ? 'calendar-only' : '',
			disabled ? 'cursor-not-allowed opacity-50' : '',
			showToday ? 'show-today' : '',
			props.mode === 'range' && selectedDates.length < 2 ? 'can-select-range' : '',
			props.mode === 'range' && selectedDates.length === 1 ? 'is-selecting-range' : '',
			props.mode === 'range' && selectedDates.length === 2 ? 'can-drag-range' : '',
			rangeDragState ? 'is-dragging-range' : '',
			rangeEndpointMoveState ? 'is-moving-range-end' : '',
		]"
	>
		<CalendarIcon
			v-if="showIcon && !calendarOnly"
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
			:tabindex="calendarOnly ? -1 : undefined"
			:aria-hidden="calendarOnly ? 'true' : undefined"
			type="text"
		/>
	</div>
</template>

<script setup lang="ts">
import 'flatpickr/dist/flatpickr.css'

import { CalendarIcon } from '@modrinth/assets'
import chevronLeftIcon from '@modrinth/assets/icons/chevron-left.svg?raw'
import chevronRightIcon from '@modrinth/assets/icons/chevron-right.svg?raw'
import flatpickr from 'flatpickr'
import type { Instance } from 'flatpickr/dist/types/instance'
import type { Options } from 'flatpickr/dist/types/options'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

type DatePickerValue = string | Date | null | undefined
type RangeEdge = 'start' | 'end'
type ViewDateAlignment = 'left' | 'right'
type CalendarViewMonth = { month: number; year: number }
type RangeDayElement = HTMLElement & { dateObj?: Date }
type RangeDragState = {
	edge: RangeEdge
	anchorDate: Date
	draggedEndpointDate: Date
	pointerId: number
	lastDateTime: number
	hasMoved: boolean
}
type RangeEndpointMoveState = {
	edge: RangeEdge
	anchorDate: Date
	endpointDate: Date
	lastPreviewDateTime: number
}

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
		showMonths?: number
		minDate?: string | Date
		maxDate?: string | Date
		/**
		 * The date the calendar opens to when no value is selected. Does not set
		 * the value — only controls which month/year is shown when the picker
		 * first opens (and on subsequent opens while the value is empty).
		 */
		defaultViewDate?: string | Date
		/**
		 * Controls which visible calendar contains the opening view date when
		 * multiple months are shown. In range mode, a complete range opens to
		 * the end date.
		 */
		viewDateAlignment?: ViewDateAlignment
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
		calendarOnly?: boolean
		/**
		 * When true (single mode only), navigating between months/years preserves the
		 * originally selected day number. If the target month has fewer days, the day
		 * is clamped to the last valid day, and a `clamp` event is emitted with the
		 * intended and resolved days. Navigating back to a month that supports the
		 * original day snaps the selection back.
		 */
		preserveDay?: boolean
		wrapperClass?: string
		inputClass?: string
		calendarClass?: string
	}>(),
	{
		disabled: false,
		readonly: false,
		enableTime: false,
		mode: 'single',
		showMonths: 1,
		time24hr: false,
		clearable: true,
		showIcon: true,
		showToday: false,
		calendarOnly: false,
		preserveDay: false,
		viewDateAlignment: 'left',
	},
)

const emit = defineEmits<{
	change: [value: DatePickerValue | DatePickerValue[]]
	clear: []
	clamp: [intendedDay: number, resolvedDay: number]
}>()

const inputRef = ref<HTMLInputElement>()
const wrapperRef = ref<HTMLElement>()
const picker = ref<Instance>()
const isSyncingFromModel = ref(false)
const intendedViewMonth = ref<CalendarViewMonth | null>(null)
const preserveViewOnNextModelSync = ref(false)
const intendedDay = ref<number | null>(null)
const isPreservingDay = ref(false)
const rangeDragState = ref<RangeDragState | null>(null)
const rangeEndpointMoveState = ref<RangeEndpointMoveState | null>(null)
const suppressNextRangeClick = ref(false)
let rangeClickSuppressionTimeout: number | null = null
let monthSelectSyncFrame: number | null = null
let calendarPortal: HTMLElement | null = null
const calendarBaseClass = 'modrinth-date-picker-calendar'
const calendarStateClasses = [
	'calendar-only',
	'show-today',
	'can-select-range',
	'is-selecting-range',
	'can-drag-range',
	'is-dragging-range',
	'is-moving-range-end',
]

function getCalendarStateClasses() {
	return [
		props.calendarOnly ? 'calendar-only' : '',
		props.showToday ? 'show-today' : '',
		props.mode === 'range' && selectedDates.value.length < 2 ? 'can-select-range' : '',
		props.mode === 'range' && selectedDates.value.length === 1 ? 'is-selecting-range' : '',
		props.mode === 'range' && selectedDates.value.length === 2 ? 'can-drag-range' : '',
		rangeDragState.value ? 'is-dragging-range' : '',
		rangeEndpointMoveState.value ? 'is-moving-range-end' : '',
	].filter(Boolean)
}

function syncCalendarStateClasses(instance?: Instance) {
	const container = (instance ?? picker.value)?.calendarContainer
	const targets = [container, calendarPortal].filter((target): target is HTMLElement =>
		Boolean(target),
	)
	if (targets.length === 0) return

	for (const target of targets) {
		for (const cls of calendarStateClasses) {
			target.classList.remove(cls)
		}

		for (const cls of getCalendarStateClasses()) {
			target.classList.add(cls)
		}
	}
}

function ensureCalendarPortal() {
	if (props.calendarOnly || typeof document === 'undefined') return undefined

	if (!calendarPortal) {
		calendarPortal = document.createElement('div')
		calendarPortal.classList.add('modrinth-date-picker', 'modrinth-date-picker-portal')

		for (const attr of wrapperRef.value?.getAttributeNames() ?? []) {
			if (attr.startsWith('data-v-')) {
				calendarPortal.setAttribute(attr, '')
			}
		}

		document.body.appendChild(calendarPortal)
	}

	syncCalendarStateClasses()
	return calendarPortal
}

function destroyCalendarPortal() {
	calendarPortal?.remove()
	calendarPortal = null
}

function getDayFromDateStr(dateStr: string): number | null {
	const parts = dateStr.split('-')
	if (parts.length < 3) return null
	const day = Number.parseInt(parts[2], 10)
	return Number.isInteger(day) ? day : null
}

function daysInMonth(month: number, year: number): number {
	return new Date(year, month + 1, 0).getDate()
}

function applyPreserveDay(instance: Instance) {
	if (!props.preserveDay) return
	if (props.mode !== 'single') return
	if (intendedDay.value === null) return

	const selected = instance.selectedDates[0]
	if (!selected) return

	if (
		selected.getMonth() === instance.currentMonth &&
		selected.getFullYear() === instance.currentYear
	) {
		return
	}

	const maxDay = daysInMonth(instance.currentMonth, instance.currentYear)
	const resolvedDay = Math.min(intendedDay.value, maxDay)
	const newDate = new Date(instance.currentYear, instance.currentMonth, resolvedDay)

	emit('clamp', intendedDay.value, resolvedDay)

	isPreservingDay.value = true
	instance.setDate(newDate, true)
	nextTick(() => {
		isPreservingDay.value = false
	})
}

let appliedCalendarClasses: string[] = []

function parseClassString(value: string | undefined): string[] {
	if (!value) return []
	return value.split(/\s+/).filter(Boolean)
}

function syncCalendarClasses(instance?: Instance) {
	const container = (instance ?? picker.value)?.calendarContainer
	if (!container) return

	container.classList.add(calendarBaseClass)

	for (const cls of appliedCalendarClasses) {
		container.classList.remove(cls)
	}

	const nextClasses = parseClassString(props.calendarClass)
	for (const cls of nextClasses) {
		container.classList.add(cls)
	}

	appliedCalendarClasses = nextClasses
	syncCalendarStateClasses(instance)
}

function syncHeaderControlState(instance: Instance) {
	const prevDisabled = instance.prevMonthNav.classList.contains('flatpickr-disabled')
	const nextDisabled = instance.nextMonthNav.classList.contains('flatpickr-disabled')

	instance.prevMonthNav.setAttribute('aria-disabled', String(prevDisabled))
	instance.nextMonthNav.setAttribute('aria-disabled', String(nextDisabled))

	if (instance.monthsDropdownContainer) {
		const monthSelectDisabled = instance.monthsDropdownContainer.options.length <= 1
		instance.monthsDropdownContainer.disabled = monthSelectDisabled
		instance.monthsDropdownContainer.setAttribute('aria-disabled', String(monthSelectDisabled))
	}
}

function getDisplayedMonthDate(instance: Instance, index: number) {
	return new Date(instance.currentYear, instance.currentMonth + index, 1)
}

function getMonthName(instance: Instance, month: number) {
	const monthNames = instance.config.shorthandCurrentMonth
		? instance.l10n.months.shorthand
		: instance.l10n.months.longhand
	return monthNames[month] ?? ''
}

function syncMultiMonthSelects(instance = picker.value) {
	if (!instance) return

	for (const select of instance.monthNav.querySelectorAll('.modrinth-monthDropdown-months')) {
		select.remove()
	}

	for (const monthElement of instance.monthNav.querySelectorAll('.modrinth-hidden-cur-month')) {
		monthElement.classList.remove('modrinth-hidden-cur-month')
	}

	const showMultiMonthSelects =
		resolvedShowMonths.value >= 1 && !instance.isMobile && !instance.config.noCalendar
	instance.calendarContainer.classList.toggle('has-multi-month-selects', showMultiMonthSelects)
	if (!showMultiMonthSelects) return

	instance.monthElements.forEach((monthElement, index) => {
		const currentMonth = monthElement.closest('.flatpickr-current-month')
		if (!currentMonth) return

		const displayedMonthDate = getDisplayedMonthDate(instance, index)
		const displayedYear = displayedMonthDate.getFullYear()
		const displayedMonth = displayedMonthDate.getMonth()
		const monthSelect = document.createElement('select')
		monthSelect.className = 'flatpickr-monthDropdown-months modrinth-monthDropdown-months'
		monthSelect.setAttribute('aria-label', instance.l10n.monthAriaLabel)
		monthSelect.tabIndex = -1

		for (let month = 0; month < 12; month++) {
			const option = document.createElement('option')
			option.className = 'flatpickr-monthDropdown-month'
			option.value = String(month)
			option.textContent = getMonthName(instance, month)
			option.selected = month === displayedMonth
			monthSelect.appendChild(option)
		}

		monthSelect.addEventListener('change', () => {
			const selectedMonth = Number.parseInt(monthSelect.value, 10)
			const selectedYear = Number.parseInt(
				instance.yearElements[index]?.value ?? String(displayedYear),
				10,
			)
			if (!Number.isFinite(selectedMonth) || !Number.isFinite(selectedYear)) return

			instance.jumpToDate(new Date(selectedYear, selectedMonth - index, 1), true)
			syncHeaderControlState(instance)
			syncMissingRangeEndState()
			syncRangeEndpointMoveState(instance)
			syncMultiMonthSelects(instance)
		})

		monthElement.classList.add('modrinth-hidden-cur-month')
		currentMonth.insertBefore(monthSelect, monthElement)
	})
}

function changeYearInput(instance: Instance, input: HTMLInputElement, delta: number) {
	const index = instance.yearElements.indexOf(input)
	if (index < 0) return

	const displayedMonthDate = getDisplayedMonthDate(instance, index)
	const displayedYear = displayedMonthDate.getFullYear()
	const selectedYear = Number.parseInt(input.value, 10)
	const nextYear = (Number.isFinite(selectedYear) ? selectedYear : displayedYear) + delta

	instance.jumpToDate(new Date(nextYear, displayedMonthDate.getMonth() - index, 1), true)
	syncHeaderControlState(instance)
	syncMissingRangeEndState()
	syncRangeEndpointMoveState(instance)
	syncMultiMonthSelects(instance)
}

function queueMultiMonthSelectSync(instance: Instance) {
	if (monthSelectSyncFrame !== null) window.cancelAnimationFrame(monthSelectSyncFrame)

	monthSelectSyncFrame = window.requestAnimationFrame(() => {
		monthSelectSyncFrame = null
		syncMultiMonthSelects(instance)
	})
}

function parseViewDate(value: DatePickerValue, instance: Instance): Date | null {
	if (!value) return null
	if (value instanceof Date) return Number.isNaN(value.getTime()) ? null : value

	const parsedDate =
		instance.parseDate(value, resolvedDateFormat.value) ?? instance.parseDate(value)
	if (parsedDate && !Number.isNaN(parsedDate.getTime())) return parsedDate

	const nativeDate = new Date(value)
	return Number.isNaN(nativeDate.getTime()) ? null : nativeDate
}

function getAlignedViewDate(date: Date) {
	if (props.viewDateAlignment !== 'right') return date

	const calendarOffset = resolvedShowMonths.value - 1
	if (calendarOffset <= 0) return date

	return new Date(date.getFullYear(), date.getMonth() - calendarOffset, 1)
}

function getOpeningViewDate() {
	if (props.mode === 'range' && selectedDates.value.length >= 2) {
		return selectedDates.value[selectedDates.value.length - 1] ?? null
	}

	return selectedDates.value[0] ?? props.defaultViewDate ?? null
}

function hasCompleteRange(instance: Instance) {
	return props.mode === 'range' && instance.selectedDates.length === 2
}

function hasCompleteModelRange() {
	return props.mode === 'range' && selectedDates.value.length === 2
}

function getCompleteRangeInputValue(instance: Instance, format: string) {
	if (!hasCompleteRange(instance)) return null

	const rangeValues = instance.selectedDates.map((date) => instance.formatDate(date, format))
	if (rangeValues[0] === rangeValues[1]) return rangeValues[0] ?? null

	return rangeValues.join(instance.l10n.rangeSeparator)
}

function syncInputDisplayValue(instance = picker.value) {
	if (!instance) return

	const inputValue = getCompleteRangeInputValue(instance, resolvedDateFormat.value)
	if (inputValue) {
		instance.input.value = inputValue
	}

	const altInputValue = getCompleteRangeInputValue(instance, resolvedAltFormat.value)
	if (instance.altInput && altInputValue) {
		instance.altInput.value = altInputValue
	}
}

function syncCalendarView(instance: Instance) {
	const viewDate = parseViewDate(getOpeningViewDate(), instance)
	if (!viewDate) return

	instance.jumpToDate(getAlignedViewDate(viewDate))
}

function getRangeDayElement(target: EventTarget | null): RangeDayElement | null {
	if (!(target instanceof Element)) return null
	return target.closest<RangeDayElement>('.flatpickr-day')
}

function isSelectableDay(dayElem: RangeDayElement) {
	return (
		Boolean(dayElem.dateObj) &&
		!dayElem.classList.contains('flatpickr-disabled') &&
		!dayElem.classList.contains('hidden')
	)
}

function getRangeEdge(dayElem: RangeDayElement): RangeEdge | null {
	if (props.mode !== 'range') return null
	if (picker.value?.selectedDates.length !== 2) return null
	if (!isSelectableDay(dayElem)) return null
	if (dayElem.classList.contains('startRange') && !dayElem.classList.contains('endRange')) {
		return 'start'
	}
	if (dayElem.classList.contains('endRange') && !dayElem.classList.contains('startRange')) {
		return 'end'
	}
	return null
}

function dateTime(date: Date) {
	return date.getTime()
}

function dateOnlyTime(date: Date) {
	return new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime()
}

function jumpToViewMonth(instance: Instance, viewMonth: CalendarViewMonth) {
	if (instance.currentMonth === viewMonth.month && instance.currentYear === viewMonth.year) return

	instance.jumpToDate(new Date(viewMonth.year, viewMonth.month, 1))
}

function withEndpointTime(date: Date, endpoint: Date) {
	return new Date(
		date.getFullYear(),
		date.getMonth(),
		date.getDate(),
		endpoint.getHours(),
		endpoint.getMinutes(),
		endpoint.getSeconds(),
		endpoint.getMilliseconds(),
	)
}

function resolveRangeEndpointMove(targetDate: Date, endpointDate: Date, anchorDate: Date) {
	const movedDate = withEndpointTime(targetDate, endpointDate)
	const [nextStartDate, nextEndDate] =
		dateOnlyTime(movedDate) <= dateOnlyTime(anchorDate)
			? [movedDate, anchorDate]
			: [anchorDate, movedDate]

	return {
		nextStartDate,
		nextEndDate,
		nextEndpointDate: movedDate,
	}
}

function withPreservedCalendarView(instance: Instance, callback: () => void) {
	const currentViewMonth = {
		month: instance.currentMonth,
		year: instance.currentYear,
	}

	callback()
	jumpToViewMonth(instance, currentViewMonth)
}

function syncRangeEndpointMoveState(instance = picker.value) {
	if (!instance) return

	for (const dayElem of instance.calendarContainer.querySelectorAll('.is-moving-range-end')) {
		dayElem.classList.remove('is-moving-range-end')
	}

	const state = rangeEndpointMoveState.value
	if (!state || props.mode !== 'range' || instance.selectedDates.length !== 2) return

	const selector =
		state.edge === 'start'
			? '.flatpickr-day.startRange:not(.endRange)'
			: '.flatpickr-day.endRange:not(.startRange)'
	instance.calendarContainer.querySelector(selector)?.classList.add('is-moving-range-end')
}

function setRangeEndpointMoveState(state: RangeEndpointMoveState | null) {
	rangeEndpointMoveState.value = state
	syncCalendarStateClasses()
	syncRangeEndpointMoveState()
}

function selectRangeEndpointForMove(edge: RangeEdge, startDate: Date, endDate: Date) {
	setRangeEndpointMoveState({
		edge,
		anchorDate: new Date(edge === 'start' ? endDate : startDate),
		endpointDate: new Date(edge === 'start' ? startDate : endDate),
		lastPreviewDateTime: dateTime(edge === 'start' ? startDate : endDate),
	})
}

function cancelRangeEndpointMovePreview() {
	if (!rangeEndpointMoveState.value) return

	setRangeEndpointMoveState(null)
	syncPickerFromModel()
}

function previewSelectedRangeEndpoint(event: MouseEvent) {
	const state = rangeEndpointMoveState.value
	const instance = picker.value
	if (!state || !instance) return

	const dayElem = getRangeDayElement(event.target)
	if (!dayElem || !instance.calendarContainer.contains(dayElem) || !isSelectableDay(dayElem)) return

	const { nextStartDate, nextEndDate, nextEndpointDate } = resolveRangeEndpointMove(
		dayElem.dateObj!,
		state.endpointDate,
		state.anchorDate,
	)
	const nextPreviewDateTime = dateTime(nextEndpointDate)
	if (nextPreviewDateTime === state.lastPreviewDateTime) return

	state.lastPreviewDateTime = nextPreviewDateTime
	withPreservedCalendarView(instance, () => {
		instance.setDate([nextStartDate, nextEndDate], false)
	})
	syncHeaderControlState(instance)
	syncMissingRangeEndState()
	syncRangeEndpointMoveState(instance)
}

function moveSelectedRangeEndpoint(dayElem: RangeDayElement) {
	const state = rangeEndpointMoveState.value
	const instance = picker.value
	if (!state || !instance || !dayElem.dateObj) return false

	const { nextStartDate, nextEndDate, nextEndpointDate } = resolveRangeEndpointMove(
		dayElem.dateObj,
		state.endpointDate,
		state.anchorDate,
	)

	setRangeEndpointMoveState(null)

	if (dateTime(nextEndpointDate) === dateTime(state.endpointDate)) return true

	intendedViewMonth.value = {
		month: instance.currentMonth,
		year: instance.currentYear,
	}

	instance.setDate([nextStartDate, nextEndDate], true)
	syncHeaderControlState(instance)

	return true
}

function clearRangeClickSuppressionTimeout() {
	if (rangeClickSuppressionTimeout === null) return

	window.clearTimeout(rangeClickSuppressionTimeout)
	rangeClickSuppressionTimeout = null
}

function clearRangeClickSuppressionSoon() {
	clearRangeClickSuppressionTimeout()
	rangeClickSuppressionTimeout = window.setTimeout(() => {
		suppressNextRangeClick.value = false
		rangeClickSuppressionTimeout = null
	}, 50)
}

function startRangeDrag(event: PointerEvent) {
	if (event.button !== 0) return
	if (rangeEndpointMoveState.value) return

	const instance = picker.value
	const dayElem = getRangeDayElement(event.target)
	if (!instance || !dayElem) return

	const edge = getRangeEdge(dayElem)
	if (!edge) return

	const [startDate, endDate] = instance.selectedDates
	if (!startDate || !endDate) return

	const draggedEndpointDate = new Date(edge === 'start' ? startDate : endDate)
	rangeDragState.value = {
		edge,
		anchorDate: new Date(edge === 'start' ? endDate : startDate),
		draggedEndpointDate,
		pointerId: event.pointerId,
		lastDateTime: dateTime(draggedEndpointDate),
		hasMoved: false,
	}
	syncCalendarStateClasses(instance)
	clearRangeClickSuppressionTimeout()
	suppressNextRangeClick.value = true

	document.addEventListener('pointermove', updateRangeDrag, true)
	document.addEventListener('pointerup', stopRangeDrag, true)
	document.addEventListener('pointercancel', stopRangeDrag, true)

	event.preventDefault()
	event.stopImmediatePropagation()
}

function updateRangeDrag(event: PointerEvent) {
	const state = rangeDragState.value
	const instance = picker.value
	if (!state || !instance || event.pointerId !== state.pointerId) return

	const target = document.elementFromPoint(event.clientX, event.clientY)
	const dayElem = getRangeDayElement(target)
	if (!dayElem || !instance.calendarContainer.contains(dayElem) || !isSelectableDay(dayElem)) return

	const { nextStartDate, nextEndDate, nextEndpointDate } = resolveRangeEndpointMove(
		dayElem.dateObj!,
		state.draggedEndpointDate,
		state.anchorDate,
	)

	const nextDraggedTime = dateTime(nextEndpointDate)
	if (nextDraggedTime === state.lastDateTime) return

	intendedViewMonth.value = {
		month: instance.currentMonth,
		year: instance.currentYear,
	}

	state.lastDateTime = nextDraggedTime
	state.hasMoved = true
	instance.setDate([nextStartDate, nextEndDate], true)
	syncHeaderControlState(instance)

	event.preventDefault()
	event.stopImmediatePropagation()
}

function stopRangeDrag(event: PointerEvent) {
	const state = rangeDragState.value
	if (!state || event.pointerId !== state.pointerId) return

	const instance = picker.value
	rangeDragState.value = null
	syncCalendarStateClasses(instance)
	document.removeEventListener('pointermove', updateRangeDrag, true)
	document.removeEventListener('pointerup', stopRangeDrag, true)
	document.removeEventListener('pointercancel', stopRangeDrag, true)
	clearRangeClickSuppressionSoon()
	if (state.hasMoved) {
		setRangeEndpointMoveState(null)
	} else {
		selectRangeEndpointForMove(
			state.edge,
			state.edge === 'start' ? state.draggedEndpointDate : state.anchorDate,
			state.edge === 'end' ? state.draggedEndpointDate : state.anchorDate,
		)
	}

	if (
		document.activeElement instanceof HTMLElement &&
		instance?.calendarContainer.contains(document.activeElement)
	) {
		document.activeElement.blur()
	}

	event.preventDefault()
	event.stopImmediatePropagation()
}

function stopRangeEndpointMouseEvent(event: MouseEvent) {
	if (
		rangeDragState.value ||
		suppressNextRangeClick.value ||
		(rangeEndpointMoveState.value && getRangeDayElement(event.target))
	) {
		event.preventDefault()
		event.stopImmediatePropagation()
	}
}

function suppressRangeDragClick(event: MouseEvent) {
	if (!suppressNextRangeClick.value) return

	clearRangeClickSuppressionTimeout()
	suppressNextRangeClick.value = false
	event.preventDefault()
	event.stopImmediatePropagation()
}

function handleRangeEndpointMoveClick(event: MouseEvent) {
	if (suppressNextRangeClick.value) {
		suppressRangeDragClick(event)
		return
	}

	const instance = picker.value
	const dayElem = getRangeDayElement(event.target)
	if (!instance || !dayElem || props.mode !== 'range') return

	const moveState = rangeEndpointMoveState.value
	if (moveState) {
		if (!isSelectableDay(dayElem)) return

		moveSelectedRangeEndpoint(dayElem)
		event.preventDefault()
		event.stopImmediatePropagation()
		return
	}

	const edge = getRangeEdge(dayElem)
	if (!edge) return

	const [startDate, endDate] = instance.selectedDates
	if (!startDate || !endDate) return

	selectRangeEndpointForMove(edge, startDate, endDate)
	event.preventDefault()
	event.stopImmediatePropagation()
}

function hasRangeEnd(instance: Instance) {
	return (
		instance.selectedDates.length === 2 ||
		Boolean(instance.calendarContainer.querySelector('.flatpickr-day.endRange:not(.startRange)'))
	)
}

function shouldSuppressMissingRangeEndBackground(instance?: Instance) {
	if (props.mode !== 'range' || !instance) return false
	if (instance.selectedDates.length !== 1) return false

	const hasRangeStart = Boolean(
		instance.calendarContainer.querySelector('.flatpickr-day.startRange:not(.endRange)'),
	)
	const hasRangePreview = Boolean(
		instance.calendarContainer.querySelector('.flatpickr-day.inRange'),
	)

	return hasRangeStart && !hasRangeEnd(instance) && !hasRangePreview
}

function syncMissingRangeEndState() {
	const instance = picker.value

	instance?.calendarContainer.classList.toggle(
		'is-missing-range-end',
		shouldSuppressMissingRangeEndBackground(instance),
	)
}

function isTimeInput(target: EventTarget | null): target is HTMLInputElement {
	return (
		target instanceof HTMLInputElement &&
		target.matches('input.flatpickr-hour, input.flatpickr-minute, input.flatpickr-second')
	)
}

function getTimeInputFromArrow(target: EventTarget | null): HTMLInputElement | null {
	if (!(target instanceof Element)) return null
	if (!target.classList.contains('arrowUp') && !target.classList.contains('arrowDown')) return null

	return target.closest('.numInputWrapper')?.querySelector<HTMLInputElement>('input') ?? null
}

function getTimeInputDigits(value: string) {
	return value.replace(/\D/g, '').slice(0, 2)
}

function sanitizeTimeInputValue(input: HTMLInputElement) {
	const nextValue = getTimeInputDigits(input.value)
	if (input.value === nextValue) return

	const cursorPosition = input.selectionStart ?? nextValue.length
	const removedBeforeCursor =
		input.value.slice(0, cursorPosition).length -
		getTimeInputDigits(input.value.slice(0, cursorPosition)).length
	const nextCursorPosition = Math.max(0, cursorPosition - removedBeforeCursor)

	input.value = nextValue
	input.setSelectionRange(nextCursorPosition, nextCursorPosition)
}

function normalizeTimeInputValue(input: HTMLInputElement) {
	sanitizeTimeInputValue(input)
	if (input.value.length === 2) return

	const minValue = Number.parseInt(input.min, 10)
	const maxValue = Number.parseInt(input.max, 10)
	const parsedValue = Number.parseInt(input.value, 10)
	const fallbackValue = Number.isFinite(minValue) ? minValue : 0
	let nextValue = Number.isFinite(parsedValue) ? parsedValue : fallbackValue

	if (Number.isFinite(minValue)) nextValue = Math.max(nextValue, minValue)
	if (Number.isFinite(maxValue)) nextValue = Math.min(nextValue, maxValue)

	input.value = String(nextValue).padStart(2, '0')
}

function normalizeTimeInputForArrowIncrement(event: MouseEvent) {
	const input = getTimeInputFromArrow(event.target)
	if (input && isTimeInput(input)) normalizeTimeInputValue(input)
}

function normalizeTimeInputForKeyboardIncrement(event: KeyboardEvent) {
	if (event.key !== 'ArrowUp' && event.key !== 'ArrowDown') return
	if (isTimeInput(event.target)) normalizeTimeInputValue(event.target)
}

function preventNonNumericTimeInput(event: InputEvent) {
	if (!isTimeInput(event.target)) return
	if (event.data && /\D/.test(event.data)) event.preventDefault()
}

function preventNonNumericTimeKeydown(event: KeyboardEvent) {
	if (!isTimeInput(event.target)) return
	if (event.metaKey || event.ctrlKey || event.altKey) return
	if (event.key.length === 1 && /\D/.test(event.key)) event.preventDefault()
}

function sanitizeNumericTimeInput(event: Event) {
	if (isTimeInput(event.target)) sanitizeTimeInputValue(event.target)
}

function syncTimeInputTypes(instance: Instance) {
	const timeInputs = [instance.hourElement, instance.minuteElement, instance.secondElement].filter(
		(input): input is HTMLInputElement => Boolean(input),
	)

	for (const input of timeInputs) {
		input.type = 'text'
		input.inputMode = 'numeric'
		input.pattern = '[0-9]*'
		normalizeTimeInputValue(input)
	}
}

const resolvedDateFormat = computed(
	() => props.dateFormat ?? (props.enableTime ? 'Y-m-d H:i' : 'Y-m-d'),
)
const resolvedAltFormat = computed(
	() => props.altFormat ?? (props.enableTime ? 'F j, Y at h:i K' : 'F j, Y'),
)
const resolvedShowMonths = computed(() =>
	Number.isFinite(props.showMonths) ? Math.max(1, Math.floor(props.showMonths)) : 1,
)

const inputClasses = computed(() => [
	props.calendarOnly
		? 'sr-only pointer-events-none absolute h-0 w-0 opacity-0'
		: 'w-full text-primary placeholder:text-secondary focus:text-contrast font-medium transition-[shadow,color] appearance-none shadow-none focus:ring-4 focus:ring-brand-shadow !outline-0',
	!props.calendarOnly && props.showIcon ? 'pl-10' : '',
	!props.calendarOnly && !props.showIcon ? 'pl-3' : '',
	!props.calendarOnly
		? 'pr-3 h-9 py-2 text-base outline-none bg-surface-4 border-none rounded-xl'
		: '',
	props.disabled && !props.calendarOnly ? 'cursor-not-allowed' : '',
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
		props.showMonths,
		props.defaultViewDate,
		props.viewDateAlignment,
		props.dateFormat,
		props.altFormat,
		props.time24hr,
		props.calendarOnly,
	],
	() => {
		if (!picker.value) return

		picker.value.set({
			...flatpickrOptions(),
			onChange: picker.value.config.onChange,
			onClose: picker.value.config.onClose,
			onReady: picker.value.config.onReady,
			onMonthChange: picker.value.config.onMonthChange,
			onYearChange: picker.value.config.onYearChange,
			onOpen: picker.value.config.onOpen,
		})

		if (!isPreservingDay.value && props.mode === 'single') {
			const value = model.value
			if (typeof value === 'string' && value) {
				const day = getDayFromDateStr(value)
				if (day !== null) intendedDay.value = day
			} else {
				intendedDay.value = null
			}
		}

		syncAltInputState()
		syncTimeInputTypes(picker.value)
		if (props.mode !== 'range' || selectedDates.value.length !== 2) cancelRangeEndpointMovePreview()
		syncPickerFromModel()
		syncCalendarClasses()
	},
	{ deep: true },
)

watch(
	() => props.calendarClass,
	() => syncCalendarClasses(),
)

watch(
	() => [
		props.showToday,
		props.calendarOnly,
		props.mode,
		selectedDates.value.length,
		Boolean(rangeDragState.value),
		Boolean(rangeEndpointMoveState.value),
	],
	() => syncCalendarStateClasses(),
)

onMounted(async () => {
	await nextTick()
	if (!inputRef.value) return

	picker.value = flatpickr(inputRef.value, {
		...flatpickrOptions(),
		onReady: (_selectedDates, _dateStr, instance) => {
			syncCalendarView(instance)

			instance.calendarContainer.addEventListener('pointerdown', startRangeDrag, true)
			instance.calendarContainer.addEventListener('mousedown', stopRangeEndpointMouseEvent, true)
			instance.calendarContainer.addEventListener('mouseup', stopRangeEndpointMouseEvent, true)
			instance.calendarContainer.addEventListener('click', handleRangeEndpointMoveClick, true)
			instance.calendarContainer.addEventListener('mouseover', previewSelectedRangeEndpoint, true)
			instance.calendarContainer.addEventListener('mouseover', syncMissingRangeEndState)
			instance.calendarContainer.addEventListener('mouseleave', syncMissingRangeEndState)
			instance.calendarContainer.addEventListener(
				'click',
				normalizeTimeInputForArrowIncrement,
				true,
			)
			instance.calendarContainer.addEventListener(
				'keydown',
				normalizeTimeInputForKeyboardIncrement,
				true,
			)
			instance.calendarContainer.addEventListener('beforeinput', preventNonNumericTimeInput, true)
			instance.calendarContainer.addEventListener('keydown', preventNonNumericTimeKeydown, true)
			instance.calendarContainer.addEventListener('input', sanitizeNumericTimeInput, true)

			instance.calendarContainer.addEventListener('mousedown', (event) => {
				if (props.mode !== 'range') return
				const target = event.target as HTMLElement | null
				const dayElem = target?.closest<RangeDayElement>('.flatpickr-day')
				if (!dayElem) return

				if (resolvedShowMonths.value > 1) {
					if (!isSelectableDay(dayElem)) return
				} else if (
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

			instance.calendarContainer.addEventListener(
				'keydown',
				(event) => {
					const target = event.target as HTMLElement | null
					if (!target?.matches('input.cur-year')) return
					if (event.key === 'ArrowUp' || event.key === 'ArrowDown') {
						event.preventDefault()
						event.stopImmediatePropagation()
						changeYearInput(instance, target as HTMLInputElement, event.key === 'ArrowUp' ? 1 : -1)
						return
					}
					if (
						event.key === 'ArrowLeft' ||
						event.key === 'ArrowRight' ||
						event.key === 'Home' ||
						event.key === 'End'
					) {
						event.stopPropagation()
					}
				},
				true,
			)
			instance.calendarContainer.addEventListener('keyup', (event) => {
				const target = event.target as HTMLElement | null
				if (!target?.matches('input.cur-year') || event.key !== 'Enter') return

				window.requestAnimationFrame(() => target.blur())
			})

			syncTimeInputTypes(instance)
			syncHeaderControlState(instance)
			syncCalendarClasses(instance)
			syncCalendarStateClasses(instance)
			syncRangeEndpointMoveState(instance)
			syncMultiMonthSelects(instance)
		},
		onChange: (_selectedDates, dateStr, instance) => {
			if (isSyncingFromModel.value) return

			const nextValue =
				props.mode === 'single'
					? dateStr || null
					: _selectedDates.map((date) => instance.formatDate(date, resolvedDateFormat.value))
			if (intendedViewMonth.value !== null) {
				preserveViewOnNextModelSync.value = true
			}
			model.value = nextValue
			emit('change', nextValue)

			if (intendedViewMonth.value !== null) {
				jumpToViewMonth(instance, intendedViewMonth.value)
			}

			syncHeaderControlState(instance)
			syncMissingRangeEndState()
			syncCalendarStateClasses(instance)
			syncRangeEndpointMoveState(instance)
			syncInputDisplayValue(instance)
		},
		onClose: (_selectedDates, dateStr, instance) => {
			cancelRangeEndpointMovePreview()
			if (hasCompleteModelRange()) {
				syncPickerFromModel()
				return
			}

			syncInputDisplayValue(instance)
			if (!props.clearable || dateStr || hasCompleteRange(instance)) return

			const nextValue = props.mode === 'single' ? null : []
			model.value = nextValue
			instance.clear(false)
			emit('clear')
			emit('change', nextValue)
		},
		onMonthChange: (_selectedDates, _dateStr, instance) => {
			applyPreserveDay(instance)
			syncHeaderControlState(instance)
			syncMissingRangeEndState()
			syncCalendarStateClasses(instance)
			syncRangeEndpointMoveState(instance)
			queueMultiMonthSelectSync(instance)
		},
		onYearChange: (_selectedDates, _dateStr, instance) => {
			applyPreserveDay(instance)
			syncHeaderControlState(instance)
			syncMissingRangeEndState()
			syncCalendarStateClasses(instance)
			syncRangeEndpointMoveState(instance)
			queueMultiMonthSelectSync(instance)
		},
		onOpen: (_selectedDates, _dateStr, instance) => {
			syncCalendarView(instance)
			syncTimeInputTypes(instance)
			syncHeaderControlState(instance)
			syncMissingRangeEndState()
			syncCalendarStateClasses(instance)
			syncRangeEndpointMoveState(instance)
			syncMultiMonthSelects(instance)
		},
	})

	if (props.mode === 'single' && typeof model.value === 'string' && model.value) {
		const day = getDayFromDateStr(model.value)
		if (day !== null) intendedDay.value = day
	}

	syncAltInputState()
	syncPickerFromModel()
	syncHeaderControlState(picker.value)
	syncRangeEndpointMoveState(picker.value)
	syncMultiMonthSelects(picker.value)
})

onBeforeUnmount(() => {
	clearRangeClickSuppressionTimeout()
	if (monthSelectSyncFrame !== null) window.cancelAnimationFrame(monthSelectSyncFrame)
	document.removeEventListener('pointermove', updateRangeDrag, true)
	document.removeEventListener('pointerup', stopRangeDrag, true)
	document.removeEventListener('pointercancel', stopRangeDrag, true)
	picker.value?.destroy()
	destroyCalendarPortal()
})

function flatpickrOptions(): Options {
	return {
		allowInput: !props.calendarOnly,
		altInput: !props.calendarOnly,
		altInputClass: props.calendarOnly ? undefined : inputClasses.value.filter(Boolean).join(' '),
		altFormat: resolvedAltFormat.value,
		appendTo: ensureCalendarPortal(),
		closeOnSelect: false,
		dateFormat: resolvedDateFormat.value,
		disableMobile: true,
		enableTime: props.enableTime,
		inline: props.calendarOnly,
		maxDate: props.maxDate,
		minDate: props.minDate,
		mode: props.mode,
		noCalendar: false,
		nextArrow: chevronRightIcon,
		prevArrow: chevronLeftIcon,
		showMonths: resolvedShowMonths.value,
		static: false,
		time_24hr: props.time24hr,
	}
}

function syncPickerFromModel() {
	if (!picker.value) return

	const preservedViewMonth =
		intendedViewMonth.value ??
		(preserveViewOnNextModelSync.value
			? {
					month: picker.value.currentMonth,
					year: picker.value.currentYear,
				}
			: null)

	isSyncingFromModel.value = true
	picker.value.setDate(selectedDates.value, false, resolvedDateFormat.value)
	syncTimeInputTypes(picker.value)

	if (preservedViewMonth !== null) {
		jumpToViewMonth(picker.value, preservedViewMonth)
		intendedViewMonth.value = null
		preserveViewOnNextModelSync.value = false
	} else {
		syncCalendarView(picker.value)
	}

	isSyncingFromModel.value = false
	syncHeaderControlState(picker.value)
	syncInputDisplayValue()
	syncMissingRangeEndState()
	syncCalendarStateClasses()
	syncRangeEndpointMoveState()
	syncMultiMonthSelects()
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
		setRangeEndpointMoveState(null)
		syncMissingRangeEndState()
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
	@apply mt-2 rounded-2xl border border-solid border-surface-5 bg-surface-3 shadow-none p-3 text-primary select-none;
	box-sizing: content-box;
}

.modrinth-date-picker.calendar-only {
	@apply block;
}

.modrinth-date-picker.calendar-only :deep(.flatpickr-wrapper) {
	@apply block w-full;
}

.modrinth-date-picker.calendar-only :deep(.flatpickr-calendar) {
	@apply m-0;
	box-sizing: content-box;
}

.modrinth-date-picker :deep(.dayContainer) {
	@apply max-w-[307.875px] min-w-[307.875px] w-[307.875px];
}

.modrinth-date-picker :deep(.flatpickr-calendar.multiMonth .flatpickr-rContainer),
.modrinth-date-picker :deep(.flatpickr-calendar.multiMonth .flatpickr-days) {
	@apply max-w-none;
}

.modrinth-date-picker :deep(.flatpickr-calendar.multiMonth .flatpickr-days) {
	overflow: visible;
}

.modrinth-date-picker :deep(.flatpickr-calendar.multiMonth .dayContainer + .dayContainer) {
	box-shadow: none;
}

.modrinth-date-picker :deep(.flatpickr-calendar::before),
.modrinth-date-picker :deep(.flatpickr-calendar::after) {
	display: none;
}

.modrinth-date-picker :deep(.flatpickr-months),
.modrinth-date-picker :deep(.flatpickr-month) {
	@apply items-center h-10 flex mb-1 overflow-visible;
}

.modrinth-date-picker :deep(.flatpickr-month),
.modrinth-date-picker :deep(.flatpickr-current-month),
.modrinth-date-picker :deep(.flatpickr-weekdays),
.modrinth-date-picker :deep(.flatpickr-weekdaycontainer) {
	@apply text-contrast shadow-none;
}

.modrinth-date-picker :deep(.flatpickr-current-month) {
	@apply flex h-full items-center justify-center gap-2 py-0 text-base font-semibold;
}

.modrinth-date-picker :deep(.flatpickr-current-month input.cur-year),
.modrinth-date-picker :deep(.flatpickr-current-month .flatpickr-monthDropdown-months) {
	@apply rounded-xl bg-surface-4 py-1 font-semibold text-contrast hover:bg-surface-5 min-h-10;
}

.modrinth-date-picker :deep(.flatpickr-current-month .flatpickr-monthDropdown-months) {
	@apply min-w-28 pl-3 pr-8;
	appearance: none;
	background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%23ffffff' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
	background-position: calc(100% - 8px) calc(50%);
	background-repeat: no-repeat;
	background-size: 16px 16px;
}
.modrinth-date-picker :deep(.flatpickr-current-month .modrinth-hidden-cur-month) {
	display: none;
}
.modrinth-date-picker :deep(.flatpickr-current-month .flatpickr-monthDropdown-months:disabled) {
	@apply cursor-not-allowed opacity-40 hover:bg-surface-4;
}
.modrinth-date-picker :deep(.flatpickr-current-month .numInputWrapper:has(input.cur-year)) {
	@apply w-[76px];
}
.modrinth-date-picker :deep(.flatpickr-current-month input.cur-year) {
	@apply min-w-[76px] px-2 text-center;
}
.modrinth-date-picker
	:deep(.flatpickr-current-month .numInputWrapper:has(input.cur-year:disabled)) {
	@apply cursor-not-allowed;
	background: none;
}
.modrinth-date-picker :deep(.flatpickr-current-month input.cur-year:disabled) {
	@apply opacity-40 bg-surface-4 hover:bg-surface-4;
}
.modrinth-date-picker
	:deep(.flatpickr-current-month .numInputWrapper:has(input.cur-year:disabled) span) {
	@apply pointer-events-none opacity-0;
}

.modrinth-date-picker :deep(.flatpickr-current-month input.cur-year:focus),
.modrinth-date-picker :deep(.flatpickr-current-month .flatpickr-monthDropdown-months:focus) {
	@apply outline-none ring-4 ring-brand-shadow;
}

.modrinth-date-picker :deep(.numInputWrapper span.arrowUp) {
	@apply border-0 rounded-tr-xl;
}
.modrinth-date-picker :deep(.numInputWrapper span.arrowDown) {
	@apply border-0 rounded-br-xl;
}

.modrinth-date-picker :deep(.numInputWrapper span.arrowUp::after) {
	border-bottom-color: var(--color-text-secondary);
}

.modrinth-date-picker :deep(.numInputWrapper span.arrowDown::after) {
	border-top-color: var(--color-text-secondary);
}

.modrinth-date-picker :deep(.flatpickr-prev-month),
.modrinth-date-picker :deep(.flatpickr-next-month) {
	@apply top-2.5 mx-3.5 flex h-10 w-10 items-center justify-center rounded-full p-0 text-secondary hover:bg-surface-4 hover:text-contrast;
}

.modrinth-date-picker :deep(.flatpickr-prev-month.flatpickr-disabled),
.modrinth-date-picker :deep(.flatpickr-next-month.flatpickr-disabled) {
	@apply cursor-not-allowed opacity-40 hover:bg-transparent hover:text-secondary;
}

.modrinth-date-picker :deep(.flatpickr-prev-month svg),
.modrinth-date-picker :deep(.flatpickr-next-month svg) {
	@apply h-5 w-5 stroke-current text-secondary;
	fill: none;
	stroke-width: 3;
}

.modrinth-date-picker :deep(.flatpickr-prev-month:hover svg),
.modrinth-date-picker :deep(.flatpickr-next-month:hover svg) {
	fill: none;
}

.modrinth-date-picker :deep(.flatpickr-weekday) {
	@apply text-xs font-semibold text-secondary;
}

.modrinth-date-picker :deep(.flatpickr-day) {
	@apply relative z-0 m-0 max-w-none rounded-full border border-solid border-transparent text-primary hover:bg-surface-4 hover:text-contrast font-semibold aspect-square h-auto;
}
.modrinth-date-picker
	:deep(
		.flatpickr-day:focus:not(:focus-visible):not(.selected):not(.startRange):not(.endRange):not(
				.inRange
			)
	) {
	@apply border-transparent bg-transparent text-primary outline-none;
}
.modrinth-date-picker
	:deep(.flatpickr-day:focus-visible:not(.selected):not(.startRange):not(.endRange):not(.inRange)) {
	@apply border-transparent bg-surface-4 text-contrast outline-none;
}
.modrinth-date-picker :deep(.flatpickr-day.flatpickr-disabled) {
	@apply hover:bg-transparent;
}

.modrinth-date-picker :deep(.flatpickr-day::before) {
	content: '';
	position: absolute;
	inset: 0 0;
	z-index: -1;
	background: transparent;
}

.modrinth-date-picker :deep(.flatpickr-day.today) {
	@apply border-transparent;
}

.modrinth-date-picker.show-today :deep(.flatpickr-day.today::after) {
	content: '';
	position: absolute;
	bottom: 6px;
	left: 50%;
	width: 4px;
	height: 4px;
	border-radius: 9999px;
	transform: translateX(-50%);
	@apply bg-brand;
}

.modrinth-date-picker.show-today :deep(.flatpickr-day.today.selected::after),
.modrinth-date-picker.show-today :deep(.flatpickr-day.today.startRange::after),
.modrinth-date-picker.show-today :deep(.flatpickr-day.today.endRange::after) {
	@apply bg-brand-inverted;
}

.modrinth-date-picker :deep(.flatpickr-day.selected),
.modrinth-date-picker :deep(.flatpickr-day.startRange),
.modrinth-date-picker :deep(.flatpickr-day.endRange) {
	@apply z-[1] border-brand bg-brand text-brand-inverted !shadow-none hover:border-brand hover:bg-brand hover:text-brand-inverted hover:shadow-none;
}

.modrinth-date-picker :deep(.flatpickr-day.inRange) {
	@apply rounded-none border-transparent bg-transparent text-brand shadow-none hover:rounded-none hover:border-transparent hover:bg-transparent;
}

.modrinth-date-picker :deep(.flatpickr-calendar.multiMonth .flatpickr-day.inRange) {
	box-shadow: none !important;
}

.modrinth-date-picker :deep(.flatpickr-day.inRange::before),
.modrinth-date-picker :deep(.flatpickr-day.startRange:not(.endRange)::before),
.modrinth-date-picker :deep(.flatpickr-day.endRange:not(.startRange)::before) {
	background: var(--color-brand-highlight);
}

.modrinth-date-picker :deep(.flatpickr-day.inRange::before) {
	left: -1px;
	right: -1px;
}

.modrinth-date-picker :deep(.flatpickr-day.inRange:nth-child(7n + 1)::before) {
	@apply rounded-l-xl;
	left: 0;
}

.modrinth-date-picker :deep(.flatpickr-day.inRange:nth-child(7n)::before) {
	@apply rounded-r-xl;
	right: 0;
}

.modrinth-date-picker :deep(.flatpickr-day.startRange:not(.endRange)) {
	@apply rounded-full border-brand;
}

.modrinth-date-picker :deep(.flatpickr-day.startRange:not(.endRange)::before) {
	@apply rounded-l-xl;
	left: 50%;
	right: -1px;
}

.modrinth-date-picker
	:deep(.flatpickr-calendar.is-missing-range-end .flatpickr-day.startRange:not(.endRange)::before),
.modrinth-date-picker :deep(.flatpickr-calendar.is-missing-range-end .flatpickr-day:hover::before) {
	background: transparent;
}

.modrinth-date-picker :deep(.flatpickr-day.startRange:nth-child(7n):not(.endRange)::before) {
	@apply rounded-r-xl;
	right: 0;
}

.modrinth-date-picker :deep(.flatpickr-day.endRange:not(.startRange)) {
	@apply rounded-full border-brand;
}

.modrinth-date-picker :deep(.flatpickr-day.endRange:not(.startRange)::before) {
	@apply rounded-r-xl;
	left: -1px;
	right: 50%;
}

.modrinth-date-picker :deep(.flatpickr-day.endRange:nth-child(7n + 1):not(.startRange)::before) {
	@apply rounded-l-xl;
	left: 0;
}

.modrinth-date-picker :deep(.flatpickr-calendar.multiMonth .flatpickr-day.hidden::before) {
	display: none;
}

.modrinth-date-picker
	:deep(.flatpickr-calendar.multiMonth .flatpickr-day.hidden + .flatpickr-day.inRange::before),
.modrinth-date-picker
	:deep(
		.flatpickr-calendar.multiMonth
			.flatpickr-day.hidden
			+ .flatpickr-day.endRange:not(.startRange)::before
	) {
	@apply rounded-l-xl;
	left: 0;
}

.modrinth-date-picker
	:deep(.flatpickr-calendar.multiMonth .flatpickr-day.inRange:has(+ .hidden)::before),
.modrinth-date-picker
	:deep(
		.flatpickr-calendar.multiMonth .flatpickr-day.startRange:not(.endRange):has(+ .hidden)::before
	) {
	@apply rounded-r-xl;
	right: 0;
}

.modrinth-date-picker.can-select-range :deep(.flatpickr-day:not(.flatpickr-disabled):not(.hidden)) {
	cursor: pointer;
}

.modrinth-date-picker.is-selecting-range
	:deep(.flatpickr-day:not(.flatpickr-disabled):not(.hidden)) {
	cursor: grabbing;
}

.modrinth-date-picker.can-drag-range
	:deep(.flatpickr-day.startRange:not(.endRange):not(.flatpickr-disabled)),
.modrinth-date-picker.can-drag-range
	:deep(.flatpickr-day.endRange:not(.startRange):not(.flatpickr-disabled)) {
	cursor: grab;
}

.modrinth-date-picker.is-moving-range-end
	:deep(.flatpickr-day:not(.flatpickr-disabled):not(.hidden)) {
	cursor: grabbing !important;
}

.modrinth-date-picker.is-dragging-range :deep(.flatpickr-day) {
	cursor: grabbing !important;
}

.modrinth-date-picker.is-dragging-range
	:deep(.flatpickr-day:not(.selected):not(.startRange):not(.endRange):not(.inRange):hover),
.modrinth-date-picker.is-dragging-range
	:deep(.flatpickr-day:not(.selected):not(.startRange):not(.endRange):not(.inRange):focus) {
	@apply border-transparent bg-transparent text-primary;
}

.modrinth-date-picker
	:deep(.flatpickr-day.prevMonthDay:not(.inRange):not(.startRange):not(.endRange)),
.modrinth-date-picker
	:deep(.flatpickr-day.nextMonthDay:not(.inRange):not(.startRange):not(.endRange)),
.modrinth-date-picker :deep(.flatpickr-day.flatpickr-disabled) {
	@apply text-secondary opacity-40;
}

.modrinth-date-picker :deep(.flatpickr-time) {
	@apply mt-2 flex h-11 max-h-none items-center  gap-2 border-0 border-t border-solid border-surface-5 px-1 pt-2 leading-none;
}

.modrinth-date-picker :deep(.flatpickr-time .numInputWrapper) {
	@apply h-full flex-1 rounded-xl bg-surface-4;
}

.modrinth-date-picker :deep(.flatpickr-time input),
.modrinth-date-picker :deep(.flatpickr-time .flatpickr-am-pm) {
	@apply h-full rounded-xl bg-transparent px-2 text-center font-semibold text-primary hover:bg-surface-5 focus:bg-surface-5;
}

.modrinth-date-picker :deep(.flatpickr-time .flatpickr-time-separator) {
	@apply flex h-full items-center justify-center text-secondary text-center;
}

.modrinth-date-picker :deep(.flatpickr-time .flatpickr-am-pm) {
	@apply flex w-14 items-center justify-center bg-surface-4 px-3;
}
</style>
