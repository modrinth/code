import type { Labrinth } from '@modrinth/api-client'
import { injectModrinthClient, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, type ComputedRef } from 'vue'

import type { AnalyticsDashboardContextValue } from '~/providers/analytics/analytics'

import { analyticsProjectEventMessages, type FormatMessage } from '../../analytics-messages.ts'
import {
	PROJECT_EVENT_DATE_FORMATTER,
	PROJECT_VERSION_UPLOAD_DEDUPE_WINDOW_MS,
	VISIBLE_PROJECT_STATUS_CHANGE_EVENT_STATUS_SET,
	type VisibleProjectStatusChangeEventStatus,
} from '../analytics-chart-constants.ts'
import type { AnalyticsChartRangeBounds } from '../analytics-chart-types.ts'
import type { AnalyticsChartEvent } from './AnalyticsChartEvents.vue'

const analyticsEventsQueryKey = ['analytics-events'] as const

export function useAnalyticsChartEvents(
	context: Pick<
		AnalyticsDashboardContextValue,
		| 'activeStat'
		| 'showChartEvents'
		| 'showProjectEvents'
		| 'displayedProjectEvents'
		| 'hasCompletedAnalyticsLoading'
	>,
	chartRangeBounds: ComputedRef<AnalyticsChartRangeBounds | null>,
	selectedProjectNameById: ComputedRef<Map<string, string>>,
	selectedProjectEventIdSet: ComputedRef<Set<string>>,
	visibleProjectEventIdSet: ComputedRef<Set<string>>,
) {
	const client = injectModrinthClient()
	const { formatMessage } = useVIntl()
	const { data: analyticsEvents } = useQuery({
		queryKey: analyticsEventsQueryKey,
		queryFn: () => client.labrinth.analytics_v3.getEvents(),
		enabled: computed(() => context.hasCompletedAnalyticsLoading.value),
		placeholderData: [],
		refetchOnMount: 'always',
		retry: false,
	})

	const localAnalyticsChartEvents = computed(() => analyticsEvents.value ?? [])
	const hasChartEvents = computed(() =>
		localAnalyticsChartEvents.value.some(isTimelineEventVisibleInCurrentGraph),
	)
	const visibleModrinthChartEvents = computed<AnalyticsChartEvent[]>(() =>
		context.showChartEvents.value
			? localAnalyticsChartEvents.value.map((event) => ({
					...event,
					markerIcon: 'info' as const,
					groupKey: 'modrinth',
				}))
			: [],
	)
	const localProjectChartEvents = computed<AnalyticsChartEvent[]>(() =>
		dedupeProjectVersionUploadEvents(
			context.displayedProjectEvents.value.filter(
				(event) =>
					selectedProjectEventIdSet.value.has(event.project_id) && shouldShowProjectEvent(event),
			),
		).map((event) => ({
			title: getProjectEventTitle(event, formatMessage),
			starts: event.timestamp,
			ends: event.timestamp,
			projectId: event.project_id,
			projectName: selectedProjectNameById.value.get(event.project_id),
			subtitle: formatProjectEventDate(event.timestamp),
			markerIcon: 'flag' as const,
			groupKey: 'project',
		})),
	)
	const hasProjectEvents = computed(() =>
		localProjectChartEvents.value.some(
			(event) =>
				isProjectChartEventVisibleForLegend(event) && isTimelineEventVisibleInCurrentGraph(event),
		),
	)
	const visibleProjectChartEvents = computed(() =>
		context.showProjectEvents.value
			? localProjectChartEvents.value.filter(isProjectChartEventVisibleForLegend)
			: [],
	)
	const visibleTimelineEvents = computed(() => [
		...visibleModrinthChartEvents.value,
		...visibleProjectChartEvents.value,
	])
	const hasVisibleTimelineEvents = computed(
		() => visibleModrinthChartEvents.value.length > 0 || visibleProjectChartEvents.value.length > 0,
	)

	function isTimelineEventVisibleInCurrentGraph(event: AnalyticsChartEvent) {
		const rangeBounds = chartRangeBounds.value
		if (!rangeBounds) return false
		if (!doesTimelineEventMatchActiveStat(event)) return false

		const eventStartMs = new Date(event.starts).getTime()
		const eventEndMs = new Date(event.ends).getTime()
		if (!Number.isFinite(eventStartMs) || !Number.isFinite(eventEndMs)) return false
		if (eventEndMs < eventStartMs) return false

		return eventEndMs >= rangeBounds.start.getTime() && eventStartMs <= rangeBounds.end.getTime()
	}

	function doesTimelineEventMatchActiveStat(event: AnalyticsChartEvent) {
		if (!event.for_metric_kind?.length) return true
		return event.for_metric_kind.some((metricKind) => metricKind === context.activeStat.value)
	}

	function isProjectChartEventVisibleForLegend(event: AnalyticsChartEvent) {
		return !event.projectId || visibleProjectEventIdSet.value.has(event.projectId)
	}

	return {
		localAnalyticsChartEvents,
		hasChartEvents,
		visibleModrinthChartEvents,
		localProjectChartEvents,
		hasProjectEvents,
		visibleProjectChartEvents,
		visibleTimelineEvents,
		hasVisibleTimelineEvents,
		isTimelineEventVisibleInCurrentGraph,
		isProjectChartEventVisibleForLegend,
	}
}

function getProjectEventTitle(
	event: Labrinth.Analytics.v3.ProjectAnalyticsEvent,
	formatMessage: FormatMessage,
) {
	if (event.kind === 'version_uploaded') {
		const versionNumber = event.version_number.trim()
		return versionNumber
			? formatMessage(analyticsProjectEventMessages.versionReleased, { version: versionNumber })
			: formatMessage(analyticsProjectEventMessages.versionUploaded)
	}

	if (isVisibleProjectStatusChangeEventStatus(event.status_to)) {
		return getProjectStatusEventTitle(event.status_to, formatMessage)
	}

	return formatMessage(analyticsProjectEventMessages.projectStatusChanged)
}

function getProjectStatusEventTitle(
	status: VisibleProjectStatusChangeEventStatus,
	formatMessage: FormatMessage,
) {
	switch (status) {
		case 'approved':
			return formatMessage(analyticsProjectEventMessages.projectApproved)
		case 'unlisted':
			return formatMessage(analyticsProjectEventMessages.projectUnlisted)
		case 'private':
			return formatMessage(analyticsProjectEventMessages.projectPrivate)
	}
}

function shouldShowProjectEvent(event: Labrinth.Analytics.v3.ProjectAnalyticsEvent) {
	if (event.kind !== 'status_changed') {
		return true
	}

	return isVisibleProjectStatusChangeEventStatus(event.status_to)
}

function isVisibleProjectStatusChangeEventStatus(
	status: Labrinth.Projects.v2.ProjectStatus,
): status is VisibleProjectStatusChangeEventStatus {
	return VISIBLE_PROJECT_STATUS_CHANGE_EVENT_STATUS_SET.has(status)
}

function dedupeProjectVersionUploadEvents(events: Labrinth.Analytics.v3.ProjectAnalyticsEvent[]) {
	const keptEvents: Labrinth.Analytics.v3.ProjectAnalyticsEvent[] = []
	const keptVersionUploadEventsByKey = new Map<
		string,
		Labrinth.Analytics.v3.ProjectAnalyticsEvent[]
	>()

	for (const event of events) {
		const key = getProjectVersionUploadDedupeKey(event)
		if (!key) {
			keptEvents.push(event)
			continue
		}

		const matchingEvents = keptVersionUploadEventsByKey.get(key) ?? []
		if (
			matchingEvents.some((matchingEvent) =>
				areProjectEventsWithinDedupeWindow(event, matchingEvent),
			)
		) {
			continue
		}

		keptEvents.push(event)
		matchingEvents.push(event)
		keptVersionUploadEventsByKey.set(key, matchingEvents)
	}

	return keptEvents
}

function getProjectVersionUploadDedupeKey(event: Labrinth.Analytics.v3.ProjectAnalyticsEvent) {
	if (event.kind !== 'version_uploaded') return null

	const versionNumber = event.version_number.trim()
	if (versionNumber.length === 0) return null

	return `${event.project_id}:${versionNumber}`
}

function areProjectEventsWithinDedupeWindow(
	left: Labrinth.Analytics.v3.ProjectAnalyticsEvent,
	right: Labrinth.Analytics.v3.ProjectAnalyticsEvent,
) {
	const leftTimestamp = new Date(left.timestamp).getTime()
	const rightTimestamp = new Date(right.timestamp).getTime()
	if (!Number.isFinite(leftTimestamp) || !Number.isFinite(rightTimestamp)) return false

	return Math.abs(leftTimestamp - rightTimestamp) <= PROJECT_VERSION_UPLOAD_DEDUPE_WINDOW_MS
}

function formatProjectEventDate(timestamp: string) {
	const date = new Date(timestamp)
	if (Number.isNaN(date.getTime())) return timestamp
	return PROJECT_EVENT_DATE_FORMATTER.format(date)
}
