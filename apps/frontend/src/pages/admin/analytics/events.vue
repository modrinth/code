<template>
	<NewModal
		ref="eventModal"
		:header="modalMode === 'create' ? 'New event' : 'Edit event'"
		width="480px"
		max-width="calc(100vw - 2rem)"
		:on-hide="resetForm"
		:close-on-click-outside="false"
	>
		<form class="flex flex-col gap-5" @submit.prevent="saveEvent">
			<label for="analytics-event-title" class="flex flex-col gap-2">
				<span class="label__title font-semibold">Title</span>
				<StyledInput
					id="analytics-event-title"
					v-model="form.title"
					type="text"
					autocomplete="off"
					placeholder="Event title..."
					:maxlength="120"
				/>
			</label>

			<label for="analytics-event-link" class="flex flex-col gap-2">
				<div class="flex items-center justify-between">
					<span class="label__title font-semibold">Announcement link</span>

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
						placeholder="Annoucement link..."
						wrapper-class="w-full"
						@change="commitAnnouncementUrl"
					/>
				</div>
			</label>

			<label for="analytics-event-date" class="flex flex-col gap-2">
				<span class="label__title font-semibold">Date range</span>
				<DatePicker
					id="analytics-event-date"
					v-model="form.dateRange"
					mode="range"
					:show-months="2"
					date-format="Y-m-d"
					alt-format="F j, Y"
					placeholder="Select date range"
					input-class="w-full"
					wrapper-class="w-full"
					view-date-alignment="right"
				/>
			</label>

			<div class="flex flex-col gap-2">
				<span class="label__title font-semibold">Metric</span>
				<MultiSelect
					v-model="form.metricKinds"
					:options="metricKindOptions"
					:clearable="false"
					:max-tag-rows="2"
					placeholder="Select metrics"
					trigger-class="border border-solid border-surface-5 bg-surface-1"
					include-select-all-option
				/>
			</div>
		</form>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="transparent">
					<button @click="eventModal?.hide()">Cancel</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="!canSaveEvent || isSaving" @click="saveEvent">
						{{ modalMode === 'create' ? 'Create event' : 'Save' }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>

	<div class="normal-page no-sidebar">
		<div class="normal-page__content flex flex-col gap-6">
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
					<span v-else class="font-medium text-primary">N/A</span>
				</template>

				<template #cell-date="{ row }">
					<span class="font-medium text-primary">{{ formatEventDateRange(row) }}</span>
				</template>

				<template #cell-metrics="{ row }">
					<div class="flex flex-wrap gap-1">
						<span
							v-for="metric in getMetricKindOptions(row.for_metric_kind)"
							:key="metric.value"
							class="inline-flex items-center rounded-full border border-solid border-surface-5 bg-surface-3 px-2 py-0.5 text-xs font-semibold text-secondary"
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
								@click="deleteEvent(row.id)"
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
						{{ isLoadingEvents ? 'Loading analytics events...' : 'No results.' }}
					</div>
				</template>
			</Table>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { EditIcon, ExternalIcon, PlusIcon, SearchIcon, TrashIcon } from '@modrinth/assets'
import {
	ButtonStyled,
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
import { useQuery } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

definePageMeta({
	middleware: ['auth', 'staff'],
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
	dateRange: DatePickerValue[]
	metricKinds: AnalyticsEventMetricKind[]
}

type DatePickerValue = string | Date | null | undefined

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()

const columns: TableColumn<EventColumnKey>[] = [
	{ key: 'date', label: 'Date', width: '18%', enableSorting: true },
	{ key: 'title', label: 'Title', width: '30%' },
	{ key: 'announcement', label: 'Announcement link', width: '20%' },
	{ key: 'metrics', label: 'Metric', width: '17%' },
	{ key: 'actions', label: 'Actions', width: '15%', align: 'right' },
]

const metricKindOptions: MultiSelectOption<AnalyticsEventMetricKind>[] = [
	{ value: 'view', label: 'Views' },
	{ value: 'downloads', label: 'Downloads' },
	{ value: 'revenue', label: 'Revenue' },
	{ value: 'playtime', label: 'Playtime' },
]
const allMetricKinds = metricKindOptions.map((option) => option.value)

const eventModal = ref<InstanceType<typeof NewModal> | null>(null)
const searchQuery = ref('')
const sortColumn = ref<EventColumnKey | undefined>('date')
const sortDirection = ref<SortDirection>('desc')
const modalMode = ref<'create' | 'edit'>('create')
const editingEventId = ref<Labrinth.Analytics.v3.AnalyticsEventId | null>(null)
const form = ref<EventForm>(getEmptyForm())
const isSaving = ref(false)
const deletingEventIds = ref(new Set<Labrinth.Analytics.v3.AnalyticsEventId>())
const notifiedEventsErrorMessage = ref<string | null>(null)
const committedAnnouncementUrl = ref('')

const {
	data: analyticsEvents,
	error: eventsError,
	isLoading: isLoadingEvents,
	refetch: refetchEvents,
} = useQuery({
	queryKey: ['analytics-events'],
	queryFn: () => client.labrinth.analytics_v3.getEvents(),
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
		dateRange: [],
		metricKinds: [...allMetricKinds],
	}
}

function openCreateModal() {
	modalMode.value = 'create'
	editingEventId.value = null
	form.value = {
		...getEmptyForm(),
		dateRange: getDefaultDateRange(),
	}
	committedAnnouncementUrl.value = ''
	eventModal.value?.show()
}

function openEditModal(event: Labrinth.Analytics.v3.AnalyticsEvent) {
	modalMode.value = 'edit'
	editingEventId.value = event.id
	form.value = {
		title: event.title,
		announcementUrl: event.announcement_url ?? '',
		dateRange: [getDateInputValue(event.starts), getDateInputValue(event.ends)],
		metricKinds: event.for_metric_kind?.length ? [...event.for_metric_kind] : [...allMetricKinds],
	}
	committedAnnouncementUrl.value = event.announcement_url ?? ''
	eventModal.value?.show()
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

		await refetchEvents()
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

async function deleteEvent(eventId: Labrinth.Analytics.v3.AnalyticsEventId) {
	if (isDeletingEvent(eventId)) {
		return
	}

	setDeletingEvent(eventId, true)

	try {
		await client.labrinth.analytics_v3.deleteEvent(eventId)
		await refetchEvents()
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
	form.value = getEmptyForm()
	editingEventId.value = null
	committedAnnouncementUrl.value = ''
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
		throw new Error('Select a date range')
	}

	const existingEvent =
		editingEventId.value === null
			? null
			: analyticsEvents.value?.find((event) => event.id === editingEventId.value)
	const [startDate, endDate] = selectedRange
	const starts = buildDateTime(startDate, existingEvent?.starts)
	const ends = buildDateTime(endDate, existingEvent?.ends)

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
	const startDate = parseDate(getDateInputValue(event.starts))
	const endDate = parseDate(getDateInputValue(event.ends))

	if (getDateInputValue(event.starts) === getDateInputValue(event.ends)) {
		return formatLongDate(startDate)
	}

	const sameYear = startDate.getFullYear() === endDate.getFullYear()
	const sameMonth = sameYear && startDate.getMonth() === endDate.getMonth()

	if (sameMonth) {
		return `${formatMonthDay(startDate)} - ${formatMonthDay(endDate)}, ${endDate.getFullYear()}`
	}

	if (sameYear) {
		return `${formatMonthDay(startDate)} - ${formatLongDate(endDate)}`
	}

	return `${formatLongDate(startDate)} - ${formatLongDate(endDate)}`
}

function buildDateTime(dateValue: string, sourceDateTime?: string): string {
	const [year, month, day] = dateValue.split('-').map(Number)
	const sourceDate = sourceDateTime ? new Date(sourceDateTime) : null
	const date =
		sourceDate && !Number.isNaN(sourceDate.getTime())
			? sourceDate
			: new Date(Date.UTC(year, month - 1, day))

	date.setUTCFullYear(year, month - 1, day)
	return date.toISOString()
}

function getDateInputValue(value: string): string {
	return value.split('T')[0]
}

function getDefaultDateRange(): DatePickerValue[] {
	const today = getTodayDateInputValue()
	return [today, today]
}

function getTodayDateInputValue(): string {
	const date = new Date()
	const year = date.getFullYear()
	const month = `${date.getMonth() + 1}`.padStart(2, '0')
	const day = `${date.getDate()}`.padStart(2, '0')
	return `${year}-${month}-${day}`
}

function getEventFormDateRange(): [string, string] | null {
	const dates = form.value.dateRange
		.map(getDatePickerValueString)
		.filter((value): value is string => Boolean(value))
	if (dates.length === 0) {
		return null
	}

	const startDate = dates[0]
	const endDate = dates[1] ?? dates[0]
	return startDate <= endDate ? [startDate, endDate] : [endDate, startDate]
}

function getDatePickerValueString(value: DatePickerValue): string | null {
	if (typeof value === 'string') {
		return isValidDateInputValue(value) ? value : null
	}
	if (value instanceof Date && !Number.isNaN(value.getTime())) {
		return formatDateInputValue(value)
	}

	return null
}

function isValidDateInputValue(value: string): boolean {
	if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) {
		return false
	}

	const parsedDate = parseDate(value)
	return !Number.isNaN(parsedDate.getTime()) && formatDateInputValue(parsedDate) === value
}

function formatDateInputValue(date: Date): string {
	const year = date.getFullYear()
	const month = `${date.getMonth() + 1}`.padStart(2, '0')
	const day = `${date.getDate()}`.padStart(2, '0')
	return `${year}-${month}-${day}`
}

function formatLongDate(date: Date): string {
	return new Intl.DateTimeFormat(undefined, {
		month: 'short',
		day: 'numeric',
		year: 'numeric',
	}).format(date)
}

function formatMonthDay(date: Date): string {
	return new Intl.DateTimeFormat(undefined, {
		month: 'short',
		day: 'numeric',
	}).format(date)
}

function parseDate(value: string): Date {
	const [year, month, day] = value.split('-').map(Number)
	return new Date(year, month - 1, day)
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
