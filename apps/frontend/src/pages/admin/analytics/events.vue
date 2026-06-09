<template>
	<ConfirmModal
		ref="deleteEventModal"
		title="Delete analytics event?"
		:description="deleteEventDescription"
		proceed-label="Delete event"
		@proceed="confirmDeleteEvent"
	/>

	<NewModal
		ref="eventModal"
		:header="modalMode === 'create' ? 'New event' : 'Edit event'"
		width="480px"
		max-width="calc(100vw - 2rem)"
		:on-hide="resetForm"
		:close-on-click-outside="false"
	>
		<div class="flex flex-col gap-5" @submit.prevent="saveEvent">
			<div class="flex flex-col gap-2">
				<span class="label__title font-semibold">Title</span>
				<StyledInput
					id="analytics-event-title"
					ref="titleInput"
					v-model="form.title"
					type="text"
					autocomplete="off"
					placeholder="Event title..."
					:maxlength="120"
				/>
			</div>

			<div class="flex flex-col gap-2">
				<div class="flex items-center justify-between">
					<span class="label__title font-semibold">Announcement link (optional)</span>

					<ButtonStyled v-if="committedNormalizedAnnouncementUrl" type="transparent" size="small">
						<a
							:href="committedNormalizedAnnouncementUrl"
							target="_blank"
							rel="noopener noreferrer"
							aria-label="Check announcement link"
							title="Check announcement link"
							class="text-sm"
						>
							<ExternalIcon aria-hidden="true" />
							Open link
						</a>
					</ButtonStyled>
				</div>
				<div class="flex items-center gap-2">
					<StyledInput
						id="analytics-event-link"
						v-model="form.announcementUrl"
						type="url"
						autocomplete="off"
						placeholder="Announcement link..."
						wrapper-class="w-full"
						@change="commitAnnouncementUrl"
					/>
				</div>
			</div>

			<div class="flex flex-col gap-2">
				<div class="flex flex-col gap-1">
					<span class="label__title font-semibold">Start date ({{ EVENT_TIME_ZONE_LABEL }})</span>
				</div>
				<DatePicker
					id="analytics-event-starts"
					v-model="form.startsAt"
					enable-time
					date-format="Y-m-d H:i"
					alt-format="F j, Y at h:i K"
					placeholder="Select start..."
					input-class="w-full"
					wrapper-class="w-full"
					show-today
				/>
			</div>
			<div class="flex flex-col gap-2">
				<span class="label__title font-semibold"
					>End date ({{ EVENT_TIME_ZONE_LABEL }}, optional)</span
				>
				<DatePicker
					id="analytics-event-ends"
					v-model="form.endsAt"
					enable-time
					date-format="Y-m-d H:i"
					alt-format="F j, Y at h:i K"
					placeholder="Select end..."
					input-class="w-full"
					wrapper-class="w-full"
					show-today
				/>
			</div>

			<div class="flex flex-col gap-2">
				<span class="label__title font-semibold">Metric</span>
				<MultiSelect
					v-model="form.metricKinds"
					:options="metricKindOptions"
					:clearable="false"
					:max-tag-rows="2"
					placeholder="Select metrics this applies to"
					include-select-all-option
				/>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="transparent">
					<button @click="eventModal?.hide()">Cancel</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="!canSaveEvent || isSaving" @click="saveEvent">
						<SaveIcon aria-hidden="true" />
						{{ modalMode === 'create' ? 'Create event' : 'Save' }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>

	<div class="normal-page no-sidebar">
		<div class="normal-page__content flex flex-col gap-4">
			<div class="mt-8 flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
				<h1 class="m-0 text-2xl font-extrabold text-contrast">Analytics Events</h1>

				<div class="flex flex-col gap-2 sm:flex-row sm:items-center">
					<StyledInput
						v-model="searchQuery"
						:icon="SearchIcon"
						type="search"
						placeholder="Search..."
						clearable
						wrapper-class="w-full sm:w-72"
					/>
					<ButtonStyled color="brand">
						<button :disabled="isSaving" @click="openCreateModal">
							<PlusIcon aria-hidden="true" />
							New event
						</button>
					</ButtonStyled>
				</div>
			</div>

			<Table
				v-model:sort-column="sortColumn"
				v-model:sort-direction="sortDirection"
				:columns="columns"
				:data="sortedEvents"
				row-key="id"
			>
				<template #cell-title="{ row }">
					<span class="line-clamp-2 font-medium text-primary">{{ row.title }}</span>
				</template>

				<template #cell-announcement="{ row }">
					<a
						v-if="row.announcement_url"
						:href="row.announcement_url"
						target="_blank"
						rel="noopener noreferrer"
						class="inline-flex items-center gap-1 font-medium text-primary hover:text-contrast"
					>
						Open link
						<ExternalIcon class="size-4" aria-hidden="true" />
					</a>
					<span v-else class="text-xs font-medium text-primary">—</span>
				</template>

				<template #cell-date="{ row }">
					<div
						v-if="isEventDateRange(row)"
						class="flex flex-col gap-0.5 text-sm font-medium leading-5 text-primary"
					>
						<span>{{ formatEventDateRangeStart(row) }} -</span>
						<span>{{ formatEventDateRangeEnd(row) }}</span>
					</div>
					<span v-else class="font-medium text-primary">{{ formatEventDateRange(row) }}</span>
				</template>

				<template #cell-metrics="{ row }">
					<div class="flex flex-wrap gap-1">
						<span
							v-for="metric in getMetricKindOptions(row.for_metric_kind)"
							:key="metric.value"
							class="inline-flex items-center rounded-full border border-solid border-surface-5 px-2 py-0.5 text-xs font-medium text-secondary"
						>
							{{ metric.label }}
						</span>
					</div>
				</template>

				<template #cell-actions="{ row }">
					<div class="flex justify-end gap-2">
						<ButtonStyled circular type="outlined" color="red">
							<button
								:aria-label="`Delete ${row.title}`"
								:disabled="isDeletingEvent(row.id)"
								@click="openDeleteEventModal(row)"
							>
								<TrashIcon aria-hidden="true" />
							</button>
						</ButtonStyled>
						<ButtonStyled type="outlined">
							<button :disabled="isSaving || isDeletingEvent(row.id)" @click="openEditModal(row)">
								Edit
								<EditIcon aria-hidden="true" />
							</button>
						</ButtonStyled>
					</div>
				</template>

				<template #empty-state>
					<div class="flex h-64 items-center justify-center text-secondary">
						<div v-if="isFetchingEvents" class="flex items-center gap-2">
							<SpinnerIcon class="size-5 animate-spin" aria-hidden="true" />
							Loading
						</div>
						<template v-else>No results.</template>
					</div>
				</template>
			</Table>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	EditIcon,
	ExternalIcon,
	PlusIcon,
	SaveIcon,
	SearchIcon,
	SpinnerIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	ConfirmModal,
	DatePicker,
	injectModrinthClient,
	injectNotificationManager,
	MultiSelect,
	type MultiSelectOption,
	NewModal,
	type SortDirection,
	StyledInput,
	Table,
	type TableColumn,
} from '@modrinth/ui'
import { isAdmin } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'

definePageMeta({
	middleware: [
		'auth',
		async () => {
			const auth = await useAuth()

			if (!auth.value.user || !isAdmin(auth.value.user)) {
				throw createError({
					fatal: true,
					statusCode: 401,
					statusMessage: 'Unauthorized',
				})
			}
		},
	],
})

type EventColumnKey = 'title' | 'announcement' | 'date' | 'metrics' | 'actions'
type AnalyticsEventMetricKind = Labrinth.Analytics.v3.AnalyticsEventMetricKind

type AnalyticsEventRow = Labrinth.Analytics.v3.AnalyticsEvent & {
	announcement: string
	date: string
	metrics: string
	actions: string
}

type EventForm = {
	title: string
	announcementUrl: string
	startsAt: DatePickerValue
	endsAt: DatePickerValue
	metricKinds: AnalyticsEventMetricKind[]
}

type DatePickerValue = string | Date | null | undefined

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const analyticsEventsQueryKey = ['analytics-events'] as const
const EVENT_TIME_ZONE = 'America/Los_Angeles'
const EVENT_TIME_ZONE_LABEL = 'PST'

const columns: TableColumn<EventColumnKey>[] = [
	{ key: 'date', label: 'Date (PST)', width: '18%', enableSorting: true },
	{ key: 'title', label: 'Title' },
	{ key: 'announcement', label: 'Announcement link', width: '18%' },
	{ key: 'metrics', label: 'Metric', width: '18%' },
	{ key: 'actions', label: 'Actions', width: '15%', align: 'right' },
]

const metricKindOptions: MultiSelectOption<AnalyticsEventMetricKind>[] = [
	{ value: 'views', label: 'Views' },
	{ value: 'downloads', label: 'Downloads' },
	{ value: 'revenue', label: 'Revenue' },
	{ value: 'playtime', label: 'Playtime' },
]
const allMetricKinds = metricKindOptions.map((option) => option.value)

const deleteEventModal = ref<InstanceType<typeof ConfirmModal> | null>(null)
const eventModal = ref<InstanceType<typeof NewModal> | null>(null)
const titleInput = ref<InstanceType<typeof StyledInput> | null>(null)
const searchQuery = ref('')
const sortColumn = ref<EventColumnKey | undefined>('date')
const sortDirection = ref<SortDirection>('desc')
const modalMode = ref<'create' | 'edit'>('create')
const editingEventId = ref<Labrinth.Analytics.v3.AnalyticsEventId | null>(null)
const pendingDeleteEvent = ref<Labrinth.Analytics.v3.AnalyticsEvent | null>(null)
const form = ref<EventForm>(getEmptyForm())
const isSaving = ref(false)
const deletingEventIds = ref(new Set<Labrinth.Analytics.v3.AnalyticsEventId>())
const notifiedEventsErrorMessage = ref<string | null>(null)
const committedAnnouncementUrl = ref('')
let resetFormTimeout: ReturnType<typeof setTimeout> | null = null

const {
	data: analyticsEvents,
	error: eventsError,
	isFetching: isFetchingEvents,
} = useQuery({
	queryKey: analyticsEventsQueryKey,
	queryFn: () => client.labrinth.analytics_v3.getEvents(),
	placeholderData: [],
	refetchOnMount: 'always',
	retry: false,
	staleTime: 0,
})

watch(eventsError, (error) => {
	if (!error) {
		notifiedEventsErrorMessage.value = null
		return
	}

	const message = error.message
	if (notifiedEventsErrorMessage.value === message) {
		return
	}

	notifiedEventsErrorMessage.value = message
	addNotification({
		title: 'Failed to load analytics events',
		text: message,
		type: 'error',
	})
})

onBeforeUnmount(() => {
	clearResetFormTimeout()
})

const trimmedSearchQuery = computed(() => searchQuery.value.trim().toLowerCase())
const normalizedAnnouncementUrl = computed(() => normalizeUrl(form.value.announcementUrl))
const committedNormalizedAnnouncementUrl = computed(() =>
	normalizeUrl(committedAnnouncementUrl.value),
)
const canSaveEvent = computed(
	() =>
		form.value.title.trim().length > 0 &&
		Boolean(getEventFormDateRange()) &&
		form.value.metricKinds.length > 0,
)
const deleteEventDescription = computed(() => {
	if (!pendingDeleteEvent.value) {
		return 'This analytics event will be deleted. This cannot be undone.'
	}

	return `This will delete "${pendingDeleteEvent.value.title}" from analytics events. This cannot be undone.`
})

const eventRows = computed<AnalyticsEventRow[]>(() =>
	(analyticsEvents.value ?? []).map((event) => ({
		...event,
		announcement: '',
		date: '',
		metrics: '',
		actions: '',
	})),
)

const filteredEvents = computed(() => {
	if (!trimmedSearchQuery.value) {
		return eventRows.value
	}

	return eventRows.value.filter((event) => {
		const dateRange = formatEventDateRange(event).toLowerCase()
		return [event.title, event.announcement_url ?? '', dateRange].some((value) =>
			value.toLowerCase().includes(trimmedSearchQuery.value),
		)
	})
})

const sortedEvents = computed(() => {
	const sorted = [...filteredEvents.value]

	if (sortColumn.value === 'date') {
		sorted.sort((left, right) => {
			const direction = sortDirection.value === 'asc' ? 1 : -1
			return (getDateTime(left.starts) - getDateTime(right.starts)) * direction
		})
	}

	return sorted
})

function getEmptyForm(): EventForm {
	return {
		title: '',
		announcementUrl: '',
		startsAt: '',
		endsAt: '',
		metricKinds: [],
	}
}

function openCreateModal() {
	modalMode.value = 'create'
	editingEventId.value = null
	form.value = {
		...getEmptyForm(),
	}
	committedAnnouncementUrl.value = ''
	clearResetFormTimeout()
	eventModal.value?.show()
	void focusTitleInput()
}

function openEditModal(event: Labrinth.Analytics.v3.AnalyticsEvent) {
	modalMode.value = 'edit'
	editingEventId.value = event.id
	form.value = {
		title: event.title,
		announcementUrl: event.announcement_url ?? '',
		startsAt: getDateTimeInputValue(event.starts),
		endsAt: getDateTimeInputValue(event.ends),
		metricKinds: event.for_metric_kind?.length ? [...event.for_metric_kind] : [...allMetricKinds],
	}
	committedAnnouncementUrl.value = event.announcement_url ?? ''
	clearResetFormTimeout()
	eventModal.value?.show()
	void focusTitleInput()
}

function openDeleteEventModal(event: Labrinth.Analytics.v3.AnalyticsEvent) {
	pendingDeleteEvent.value = event
	deleteEventModal.value?.show()
}

async function focusTitleInput() {
	await nextTick()
	setTimeout(() => {
		titleInput.value?.focus()
	}, 75)
}

async function saveEvent() {
	if (!canSaveEvent.value || isSaving.value) {
		return
	}

	isSaving.value = true

	try {
		const payload = buildEventPayload()

		if (modalMode.value === 'edit' && editingEventId.value !== null) {
			await client.labrinth.analytics_v3.editEvent(editingEventId.value, payload)
		} else {
			await client.labrinth.analytics_v3.createEvent(payload)
		}

		await queryClient.invalidateQueries({ queryKey: analyticsEventsQueryKey })
		eventModal.value?.hide()
		addNotification({
			title: modalMode.value === 'edit' ? 'Analytics event updated' : 'Analytics event created',
			type: 'success',
		})
	} catch (error) {
		addNotification({
			title:
				modalMode.value === 'edit'
					? 'Failed to update analytics event'
					: 'Failed to create analytics event',
			text: getErrorMessage(error),
			type: 'error',
		})
	} finally {
		isSaving.value = false
	}
}

async function confirmDeleteEvent() {
	if (!pendingDeleteEvent.value) {
		return
	}

	const eventId = pendingDeleteEvent.value.id
	pendingDeleteEvent.value = null
	await deleteEvent(eventId)
}

async function deleteEvent(eventId: Labrinth.Analytics.v3.AnalyticsEventId) {
	if (isDeletingEvent(eventId)) {
		return
	}

	setDeletingEvent(eventId, true)

	try {
		await client.labrinth.analytics_v3.deleteEvent(eventId)
		await queryClient.invalidateQueries({ queryKey: analyticsEventsQueryKey })
		addNotification({
			title: 'Analytics event deleted',
			type: 'success',
		})
	} catch (error) {
		addNotification({
			title: 'Failed to delete analytics event',
			text: getErrorMessage(error),
			type: 'error',
		})
	} finally {
		setDeletingEvent(eventId, false)
	}
}

function resetForm() {
	clearResetFormTimeout()
	resetFormTimeout = setTimeout(() => {
		form.value = getEmptyForm()
		editingEventId.value = null
		committedAnnouncementUrl.value = ''
		resetFormTimeout = null
	}, 500)
}

function clearResetFormTimeout() {
	if (!resetFormTimeout) {
		return
	}

	clearTimeout(resetFormTimeout)
	resetFormTimeout = null
}

function normalizeUrl(value: string): string | undefined {
	const trimmed = value.trim()
	if (!trimmed) {
		return undefined
	}

	if (/^https?:\/\//i.test(trimmed)) {
		return trimmed
	}

	return `https://${trimmed}`
}

function commitAnnouncementUrl() {
	committedAnnouncementUrl.value = form.value.announcementUrl
}

function buildEventPayload(): Labrinth.Analytics.v3.AnalyticsEventUpsert {
	const selectedRange = getEventFormDateRange()
	if (!selectedRange) {
		throw new Error('Select a valid start and end date')
	}

	const starts = parseDateTimeInputValue(selectedRange[0]).toISOString()
	const ends = parseDateTimeInputValue(selectedRange[1]).toISOString()

	return {
		announcement_url: normalizedAnnouncementUrl.value ?? null,
		for_metric_kind: [...form.value.metricKinds],
		title: form.value.title.trim(),
		ends,
		starts,
	}
}

function getMetricKindOptions(
	metricKinds: AnalyticsEventMetricKind[] | null,
): MultiSelectOption<AnalyticsEventMetricKind>[] {
	const visibleKinds = metricKinds?.length ? metricKinds : allMetricKinds
	return metricKindOptions.filter((option) => visibleKinds.includes(option.value))
}

function formatEventDateRange(event: Labrinth.Analytics.v3.AnalyticsEvent): string {
	const startDate = new Date(event.starts)
	const endDate = new Date(event.ends)
	const startDateValue = getDateInputValueInTimeZone(startDate, EVENT_TIME_ZONE)
	const endDateValue = getDateInputValueInTimeZone(endDate, EVENT_TIME_ZONE)

	if (startDate.getTime() === endDate.getTime()) {
		return formatDateTime(startDate)
	}

	const sameYear = startDateValue.slice(0, 4) === endDateValue.slice(0, 4)
	const sameMonth = sameYear && startDateValue.slice(5, 7) === endDateValue.slice(5, 7)
	const sameDay = startDateValue === endDateValue

	if (sameDay) {
		return `${formatLongDate(startDate)}, ${formatTime(startDate)} - ${formatTime(endDate)}`
	}

	if (sameMonth) {
		return `${formatMonthDayTime(startDate)} - ${formatMonthDayTime(endDate)}, ${endDateValue.slice(0, 4)}`
	}

	if (sameYear) {
		return `${formatMonthDayTime(startDate)} - ${formatLongDateTime(endDate)}`
	}

	return `${formatLongDateTime(startDate)} - ${formatLongDateTime(endDate)}`
}

function isEventDateRange(event: Labrinth.Analytics.v3.AnalyticsEvent): boolean {
	return new Date(event.starts).getTime() !== new Date(event.ends).getTime()
}

function formatEventDateRangeStart(event: Labrinth.Analytics.v3.AnalyticsEvent): string {
	return formatLongDateTime(new Date(event.starts))
}

function formatEventDateRangeEnd(event: Labrinth.Analytics.v3.AnalyticsEvent): string {
	return formatLongDateTime(new Date(event.ends))
}

function getEventFormDateRange(): [string, string] | null {
	const startValue = getDatePickerValueString(form.value.startsAt)
	const endValue = isEmptyDatePickerValue(form.value.endsAt)
		? startValue
		: getDatePickerValueString(form.value.endsAt)

	if (!startValue || !endValue) {
		return null
	}

	const startDate = parseDateTimeInputValue(startValue)
	const endDate = parseDateTimeInputValue(endValue)

	if (startDate.getTime() > endDate.getTime()) {
		return null
	}

	return [startValue, endValue]
}

function isEmptyDatePickerValue(value: DatePickerValue): boolean {
	return value === '' || value === null || value === undefined
}

function getDatePickerValueString(value: DatePickerValue): string | null {
	if (typeof value === 'string') {
		return isValidDateTimeInputValue(value) ? value : null
	}
	if (value instanceof Date && !Number.isNaN(value.getTime())) {
		return formatDateTimeInputValue(value)
	}

	return null
}

function isValidDateTimeInputValue(value: string): boolean {
	if (!/^\d{4}-\d{2}-\d{2} \d{2}:\d{2}$/.test(value)) return false

	const parsedDate = parseDateTimeInputValue(value)
	return (
		!Number.isNaN(parsedDate.getTime()) &&
		formatDateTimeInputValueInTimeZone(parsedDate, EVENT_TIME_ZONE) === value
	)
}

function getDateTimeInputValue(value: string): string {
	const date = new Date(value)
	return Number.isNaN(date.getTime())
		? ''
		: formatDateTimeInputValueInTimeZone(date, EVENT_TIME_ZONE)
}

function formatDateTimeInputValue(date: Date): string {
	return formatDateTimeInputValueInTimeZone(date, EVENT_TIME_ZONE)
}

function formatDateTimeInputValueInTimeZone(date: Date, timeZone: string): string {
	const parts = getTimeZoneDateParts(date, timeZone)
	if (!parts) return ''

	const year = `${parts.year}`.padStart(4, '0')
	const month = `${parts.month}`.padStart(2, '0')
	const day = `${parts.day}`.padStart(2, '0')
	const hours = `${parts.hour}`.padStart(2, '0')
	const minutes = `${parts.minute}`.padStart(2, '0')
	return `${year}-${month}-${day} ${hours}:${minutes}`
}

function getDateInputValueInTimeZone(date: Date, timeZone: string): string {
	const parts = getTimeZoneDateParts(date, timeZone)
	if (!parts) return ''

	const year = `${parts.year}`.padStart(4, '0')
	const month = `${parts.month}`.padStart(2, '0')
	const day = `${parts.day}`.padStart(2, '0')
	return `${year}-${month}-${day}`
}

function parseDateTimeInputValue(value: string): Date {
	return getDateTimeInTimeZone(value, EVENT_TIME_ZONE)
}

function getDateTimeInTimeZone(value: string, timeZone: string): Date {
	const match = value.match(/^(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2})$/)
	if (!match) return new Date(Number.NaN)

	const [, yearValue, monthValue, dayValue, hourValue, minuteValue] = match
	const year = Number(yearValue)
	const month = Number(monthValue)
	const day = Number(dayValue)
	const hour = Number(hourValue)
	const minute = Number(minuteValue)
	const utcGuess = Date.UTC(year, month - 1, day, hour, minute)
	let offset = getTimeZoneOffsetMs(new Date(utcGuess), timeZone)
	offset = getTimeZoneOffsetMs(new Date(utcGuess - offset), timeZone)

	return new Date(utcGuess - offset)
}

function getTimeZoneOffsetMs(date: Date, timeZone: string): number {
	const parts = getTimeZoneDateParts(date, timeZone)
	if (!parts) return 0

	const zonedDateAsUtc = Date.UTC(
		parts.year,
		parts.month - 1,
		parts.day,
		parts.hour,
		parts.minute,
		parts.second,
	)
	return zonedDateAsUtc - date.getTime()
}

function getTimeZoneDateParts(date: Date, timeZone: string) {
	if (Number.isNaN(date.getTime())) return null

	const parts = new Intl.DateTimeFormat('en-US', {
		timeZone,
		year: 'numeric',
		month: '2-digit',
		day: '2-digit',
		hour: '2-digit',
		minute: '2-digit',
		second: '2-digit',
		hourCycle: 'h23',
	}).formatToParts(date)

	const valueByType = Object.fromEntries(parts.map((part) => [part.type, part.value]))
	return {
		year: Number(valueByType.year),
		month: Number(valueByType.month),
		day: Number(valueByType.day),
		hour: Number(valueByType.hour),
		minute: Number(valueByType.minute),
		second: Number(valueByType.second),
	}
}

function formatDateTime(date: Date): string {
	return new Intl.DateTimeFormat(undefined, {
		month: 'short',
		day: 'numeric',
		year: 'numeric',
		hour: 'numeric',
		minute: '2-digit',
		timeZone: EVENT_TIME_ZONE,
	}).format(date)
}

function formatLongDateTime(date: Date): string {
	return `${formatLongDate(date)}, ${formatTime(date)}`
}

function formatMonthDayTime(date: Date): string {
	return `${formatMonthDay(date)}, ${formatTime(date)}`
}

function formatLongDate(date: Date): string {
	return new Intl.DateTimeFormat(undefined, {
		month: 'short',
		day: 'numeric',
		year: 'numeric',
		timeZone: EVENT_TIME_ZONE,
	}).format(date)
}

function formatMonthDay(date: Date): string {
	return new Intl.DateTimeFormat(undefined, {
		month: 'short',
		day: 'numeric',
		timeZone: EVENT_TIME_ZONE,
	}).format(date)
}

function formatTime(date: Date): string {
	return new Intl.DateTimeFormat(undefined, {
		hour: 'numeric',
		minute: '2-digit',
		timeZone: EVENT_TIME_ZONE,
	}).format(date)
}

function getDateTime(value: string): number {
	return Date.parse(value)
}

function isDeletingEvent(eventId: Labrinth.Analytics.v3.AnalyticsEventId): boolean {
	return deletingEventIds.value.has(eventId)
}

function setDeletingEvent(eventId: Labrinth.Analytics.v3.AnalyticsEventId, deleting: boolean) {
	const nextIds = new Set(deletingEventIds.value)

	if (deleting) {
		nextIds.add(eventId)
	} else {
		nextIds.delete(eventId)
	}

	deletingEventIds.value = nextIds
}

function getErrorMessage(error: unknown): string {
	return error instanceof Error ? error.message : String(error)
}
</script>
