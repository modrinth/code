import type { AbstractModrinthClient, Archon } from '@modrinth/api-client'
import { useInfiniteQuery, useQueryClient } from '@tanstack/vue-query'
import type { ComputedRef } from 'vue'
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import type {
	DropdownFilterBarCategory,
	DropdownFilterBarItem,
	DropdownFilterBarOption,
} from '#ui/components/base/DropdownFilterBar.vue'
import type {
	TimeFrameLastUnit,
	TimeFrameMode,
	TimeFramePreset,
} from '#ui/components/base/TimeFramePicker.vue'
import type { ServerAuditLogEntry } from '#ui/components/servers/access'
import { parseAuditEvent } from '#ui/components/servers/access/events'
import { useVIntl } from '#ui/composables/i18n'
import type { AbstractWebNotificationManager } from '#ui/providers/web-notifications'

import {
	actionLogActionGroups,
	type ActionLogFilterActionName,
	compareFilterOptions,
	getActionLogEntryId,
	getAuditLogTimeframeRange,
	isActionLogActionName,
	SERVER_SCOPED_ACTION_LOG_WORLD_FILTER,
	SUPPORT_ACTION_LOG_USER_FILTER,
} from './audit-log-utils'
import { accessMessages, actionLogActionMessages } from './messages'

type AuditLogFilterKey = 'users' | 'worlds' | 'actions'

type UseAccessAuditLogOptions = {
	client: AbstractModrinthClient
	serverId: string
	serverFull: ComputedRef<Archon.Servers.v1.ServerFull | null>
	showAuditLogInstances: ComputedRef<boolean>
	addNotification: AbstractWebNotificationManager['addNotification']
}

const ACTION_LOG_PAGE_SIZE = 200
const ACTION_LOG_FILTER_OVERLAY_MS = 750

export function useAccessAuditLog({
	client,
	serverId,
	serverFull,
	showAuditLogInstances,
	addNotification,
}: UseAccessAuditLogOptions) {
	const { formatMessage } = useVIntl()
	const queryClient = useQueryClient()
	const auditLogFilters = ref<Record<string, string[]>>({
		users: [],
		worlds: [],
		actions: [],
	})
	const auditLogTimeframeMode = ref<TimeFrameMode>('preset')
	const auditLogTimeframePreset = ref<TimeFramePreset>('last_7_days')
	const auditLogTimeframeLastAmount = ref(30)
	const auditLogTimeframeLastUnit = ref<TimeFrameLastUnit>('days')
	const auditLogTimeframeCustomStartDate = ref('')
	const auditLogTimeframeCustomEndDate = ref('')
	const auditLogSortDirection = ref<Archon.Actions.v1.SortOrder>('desc')

	const worldOptions = computed(
		() => serverFull.value?.worlds.map((world) => ({ id: world.id, name: world.name })) ?? [],
	)
	const isAuditLogWorldFilterVisible = computed(
		() => showAuditLogInstances.value && worldOptions.value.length > 0,
	)

	const worldById = computed(
		() => new Map(worldOptions.value.map((world) => [world.id, world] as const)),
	)

	const backupById = computed(() => {
		const backups = new Map<string, Archon.Backups.v1.Backup>()
		for (const world of serverFull.value?.worlds ?? []) {
			for (const backup of world.backups ?? []) {
				backups.set(backup.id, backup)
			}
		}
		return backups
	})

	const actionLogDateFilter = computed(() => {
		const range = getAuditLogTimeframeRange({
			mode: auditLogTimeframeMode.value,
			preset: auditLogTimeframePreset.value,
			lastAmount: auditLogTimeframeLastAmount.value,
			lastUnit: auditLogTimeframeLastUnit.value,
			customStartDate: auditLogTimeframeCustomStartDate.value,
			customEndDate: auditLogTimeframeCustomEndDate.value,
		})

		return {
			min_datetime: range?.start.toISOString(),
			max_datetime: range?.end.toISOString(),
		}
	})

	const actionLogEndpointFilter = computed<Archon.Actions.v1.ActionLogFilter | undefined>(() => {
		const users = selectedAuditLogFilterValues('users')
		const worlds = isAuditLogWorldFilterVisible.value ? selectedAuditLogWorldFilterValues() : []
		const actions = selectedAuditLogFilterValues('actions').filter(isActionLogActionName)
		const filter: Archon.Actions.v1.ActionLogFilter = {}

		if (users.length > 0) filter.users = users
		if (worlds.length > 0) filter.worlds = worlds
		if (actions.length > 0) filter.actions = actions

		return Object.keys(filter).length > 0 ? filter : undefined
	})
	const actionLogBaseQueryKey = ['servers', 'action-log', 'v1', 'infinite', serverId] as const
	const actionLogQueryKey = computed(() => {
		const filter = actionLogEndpointFilter.value
		const dateFilter = actionLogDateFilter.value

		return [
			...actionLogBaseQueryKey,
			filter ?? null,
			dateFilter.min_datetime ?? null,
			dateFilter.max_datetime ?? null,
			auditLogSortDirection.value,
		]
	})
	const actionLogQuery = useInfiniteQuery({
		queryKey: actionLogQueryKey,
		queryFn: ({ pageParam = 0 }) => {
			const offset = typeof pageParam === 'number' ? pageParam : 0
			return client.archon.actions_v1.list(serverId, {
				limit: ACTION_LOG_PAGE_SIZE,
				offset,
				order: auditLogSortDirection.value,
				filter: actionLogEndpointFilter.value,
				...actionLogDateFilter.value,
			})
		},
		getNextPageParam: (lastPage) =>
			typeof lastPage.next_offset === 'number' ? lastPage.next_offset : undefined,
		initialPageParam: 0,
		placeholderData: (previousData) => previousData,
		staleTime: 30_000,
	})
	const actionLogFilterSignature = computed(() =>
		JSON.stringify([
			actionLogEndpointFilter.value ?? null,
			actionLogDateFilter.value.min_datetime ?? null,
			actionLogDateFilter.value.max_datetime ?? null,
		]),
	)
	const isActionLogFilterTransitioning = ref(false)
	const isActionLogSortTransitioning = ref(false)
	let actionLogFilterTransitionTimeout: ReturnType<typeof setTimeout> | null = null
	let actionLogSortTransitionTimeout: ReturnType<typeof setTimeout> | null = null

	watch(actionLogFilterSignature, (_signature, previousSignature) => {
		if (previousSignature === undefined) return
		startActionLogFilterTransition()
	})

	watch(
		auditLogSortDirection,
		(_direction, previousDirection) => {
			if (previousDirection === undefined) return
			startActionLogSortTransition()
		},
		{ flush: 'sync' },
	)

	watch(
		() => actionLogQuery.isFetching.value,
		(isFetching) => {
			if (!isFetching && isActionLogSortTransitioning.value) {
				finishActionLogSortTransition()
			}
		},
		{ flush: 'post' },
	)

	onBeforeUnmount(() => {
		if (actionLogFilterTransitionTimeout) {
			clearTimeout(actionLogFilterTransitionTimeout)
		}
		if (actionLogSortTransitionTimeout) {
			clearTimeout(actionLogSortTransitionTimeout)
		}
	})

	const auditEntries = computed<ServerAuditLogEntry[]>(() => {
		const pages = actionLogQuery.data.value?.pages ?? []
		const entryIdCounts = new Map<string, number>()

		return pages.flatMap((actionLog) =>
			actionLog.data.map((entry) => {
				const entryId = getActionLogEntryId(entry)
				const entryIdCount = entryIdCounts.get(entryId) ?? 0
				entryIdCounts.set(entryId, entryIdCount + 1)

				return apiActionLogEntryToAuditEntry(
					entry,
					actionLog,
					entryIdCount === 0 ? entryId : `${entryId}-${entryIdCount}`,
				)
			}),
		)
	})
	const hasShownActionLogLoadError = ref(false)
	const hasMoreActionLogEntries = computed(
		() => !actionLogQuery.isPlaceholderData.value && actionLogQuery.hasNextPage.value,
	)
	const isLoadingMoreActionLogEntries = computed(() => actionLogQuery.isFetchingNextPage.value)
	const isActionLogFiltering = computed(() => isActionLogFilterTransitioning.value)
	const initialAuditLogUserFilterOptions = ref<DropdownFilterBarOption[]>([])

	watch(
		() => actionLogQuery.data.value?.pages ?? [],
		(pages) => {
			if (actionLogEndpointFilter.value) return

			initialAuditLogUserFilterOptions.value = mergeAuditLogUserFilterOptions(
				initialAuditLogUserFilterOptions.value,
				extractAuditLogUserFilterOptions(pages),
			)
		},
		{ immediate: true },
	)

	const auditLogUserFilterOptions = computed<DropdownFilterBarOption[]>(() => {
		if (initialAuditLogUserFilterOptions.value.length > 0) {
			return initialAuditLogUserFilterOptions.value
		}

		return extractAuditLogUserFilterOptions(actionLogQuery.data.value?.pages ?? [])
	})

	const auditLogWorldFilterOptions = computed<DropdownFilterBarOption[]>(() => [
		{
			value: SERVER_SCOPED_ACTION_LOG_WORLD_FILTER,
			label: formatMessage(accessMessages.serverScopedInstance),
			searchTerms: [
				SERVER_SCOPED_ACTION_LOG_WORLD_FILTER,
				formatMessage(accessMessages.serverScopedInstance),
			],
		},
		...worldOptions.value.map((world) => ({
			value: world.id,
			label: world.name,
			searchTerms: [world.id, world.name],
		})),
	])

	const auditLogActionFilterOptions = computed<DropdownFilterBarItem[]>(() =>
		actionLogActionGroups.flatMap((group) => [
			{
				type: 'section-header' as const,
				key: group.key,
				label: formatMessage(group.label),
				icon: group.icon,
			},
			...group.actions.map((action) => ({
				value: action,
				label: formatActionLogAction(action),
				searchTerms: [action, action.replaceAll('_', ' ')],
			})),
		]),
	)

	const auditLogFilterCategories = computed<DropdownFilterBarCategory[]>(() => {
		const categories: DropdownFilterBarCategory[] = [
			{
				key: 'users',
				label: formatMessage(accessMessages.userFilter),
				options: auditLogUserFilterOptions.value,
			},
		]

		if (isAuditLogWorldFilterVisible.value) {
			categories.push({
				key: 'worlds',
				label: formatMessage(accessMessages.instanceFilter),
				options: auditLogWorldFilterOptions.value,
			})
		}

		categories.push({
			key: 'actions',
			label: formatMessage(accessMessages.actionTypeFilter),
			options: auditLogActionFilterOptions.value,
			searchable: true,
			searchPlaceholder: formatMessage(accessMessages.actionTypeFilterSearch),
			submenuClass: 'w-[22rem]',
			previewDropdownMinWidth: '20rem',
		})

		return categories
	})

	const hasActiveAuditLogDateFilter = computed(
		() => !!actionLogDateFilter.value.min_datetime || !!actionLogDateFilter.value.max_datetime,
	)

	const hasActiveAuditLogFilters = computed(
		() =>
			hasActiveAuditLogDateFilter.value ||
			(isAuditLogWorldFilterVisible.value
				? (['users', 'worlds', 'actions'] satisfies AuditLogFilterKey[])
				: (['users', 'actions'] satisfies AuditLogFilterKey[])
			).some((key) => selectedAuditLogFilterValues(key).length > 0),
	)

	watch(
		() => actionLogQuery.error.value,
		(actionLogError) => {
			if (hasShownActionLogLoadError.value || !actionLogError) return

			hasShownActionLogLoadError.value = true
			addNotification({
				type: 'error',
				title: formatMessage(accessMessages.loadFailedTitle),
				text: formatErrorMessage(actionLogError) ?? formatMessage(accessMessages.loadFailedText),
			})
		},
	)

	function selectedAuditLogFilterValues(key: AuditLogFilterKey): string[] {
		const values = auditLogFilters.value[key]
		return values ? [...values] : []
	}

	function selectedAuditLogWorldFilterValues(): Array<string | null> {
		return selectedAuditLogFilterValues('worlds').map((world) =>
			world === SERVER_SCOPED_ACTION_LOG_WORLD_FILTER ? null : world,
		)
	}

	function extractAuditLogUserFilterOptions(
		pages: Archon.Actions.v1.ActionLogResponse[],
	): DropdownFilterBarOption[] {
		const options = new Map<string, DropdownFilterBarOption>()

		for (const page of pages) {
			for (const entry of page.data) {
				if (entry.actor.type === 'support') {
					const userId = entry.actor.user_id ?? null
					const user = userId ? page.users[userId] : undefined
					if (!options.has(SUPPORT_ACTION_LOG_USER_FILTER)) {
						options.set(SUPPORT_ACTION_LOG_USER_FILTER, {
							value: SUPPORT_ACTION_LOG_USER_FILTER,
							label: formatMessage(accessMessages.supportActor),
							searchTerms: [
								SUPPORT_ACTION_LOG_USER_FILTER,
								formatMessage(accessMessages.supportActor),
								userId,
								user?.username,
							].filter(Boolean) as string[],
						})
					}
					continue
				}

				const id = entry.actor.user_id
				const user = page.users[id]
				if (!options.has(id)) {
					options.set(id, {
						value: id,
						label: user?.username ?? id,
						searchTerms: [id, user?.username].filter(Boolean) as string[],
					})
				}
			}
		}

		return [...options.values()].sort(compareFilterOptions)
	}

	function mergeAuditLogUserFilterOptions(
		existingOptions: DropdownFilterBarOption[],
		nextOptions: DropdownFilterBarOption[],
	): DropdownFilterBarOption[] {
		const options = new Map(existingOptions.map((option) => [option.value, option] as const))

		for (const option of nextOptions) {
			options.set(option.value, option)
		}

		return [...options.values()].sort(compareFilterOptions)
	}

	function formatActionLogAction(action: ActionLogFilterActionName): string {
		return formatMessage(actionLogActionMessages[action])
	}

	function loadMoreActionLogEntries() {
		if (
			isActionLogFilterTransitioning.value ||
			actionLogQuery.isPlaceholderData.value ||
			!actionLogQuery.hasNextPage.value ||
			actionLogQuery.isFetchingNextPage.value
		) {
			return
		}
		void actionLogQuery.fetchNextPage()
	}

	function startActionLogFilterTransition() {
		isActionLogFilterTransitioning.value = true

		if (actionLogFilterTransitionTimeout) {
			clearTimeout(actionLogFilterTransitionTimeout)
		}

		actionLogFilterTransitionTimeout = setTimeout(() => {
			isActionLogFilterTransitioning.value = false
			actionLogFilterTransitionTimeout = null
		}, ACTION_LOG_FILTER_OVERLAY_MS)
	}

	function startActionLogSortTransition() {
		isActionLogSortTransitioning.value = true

		if (actionLogSortTransitionTimeout) {
			clearTimeout(actionLogSortTransitionTimeout)
		}

		actionLogSortTransitionTimeout = setTimeout(() => {
			isActionLogSortTransitioning.value = false
			actionLogSortTransitionTimeout = null
		}, 2500)
	}

	function finishActionLogSortTransition() {
		if (actionLogSortTransitionTimeout) {
			clearTimeout(actionLogSortTransitionTimeout)
		}

		actionLogSortTransitionTimeout = setTimeout(() => {
			isActionLogSortTransitioning.value = false
			actionLogSortTransitionTimeout = null
		}, 120)
	}

	function apiActionLogEntryToAuditEntry(
		entry: Archon.Actions.v1.ActionEntry,
		actionLog: Archon.Actions.v1.ActionLogResponse,
		id: string,
	): ServerAuditLogEntry {
		const event = parseAuditEvent(entry, {
			serverId,
			users: actionLog.users,
			addons: actionLog.addons,
			worldById: worldById.value,
			backupById: backupById.value,
			versions: actionLog.versions ?? {},
		})

		return {
			id,
			actor: event.props.actor,
			world: event.props.world,
			event,
			timestamp: entry.timestamp,
		}
	}

	async function invalidateActionLog() {
		await queryClient.invalidateQueries({ queryKey: actionLogBaseQueryKey })
	}

	return {
		auditEntries,
		auditLogFilterCategories,
		auditLogFilters,
		auditLogSortDirection,
		auditLogTimeframeCustomEndDate,
		auditLogTimeframeCustomStartDate,
		auditLogTimeframeLastAmount,
		auditLogTimeframeLastUnit,
		auditLogTimeframeMode,
		auditLogTimeframePreset,
		hasActiveAuditLogFilters,
		hasMoreActionLogEntries,
		invalidateActionLog,
		isActionLogFiltering,
		isActionLogSortTransitioning,
		isLoadingMoreActionLogEntries,
		loadMoreActionLogEntries,
	}
}

function formatErrorMessage(error: unknown): string | undefined {
	return error instanceof Error ? error.message : undefined
}
