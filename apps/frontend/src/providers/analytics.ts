import type { Labrinth } from '@modrinth/api-client'
import { createContext, injectModrinthClient, type ProjectPageContext } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import type { ComputedRef, Ref } from 'vue'

import type { OrganizationContext } from './organization-context'

export type AnalyticsDashboardStat = 'views' | 'downloads' | 'revenue' | 'playtime'

export interface AnalyticsDashboardProject {
	id: string
	name: string
}

export interface AnalyticsDashboardTotals {
	views: number
	downloads: number
	revenue: number
	playtime: number
}

export interface AnalyticsDashboardPercentChanges {
	views: number
	downloads: number
	revenue: number
	playtime: number
}

export interface AnalyticsDashboardContextValue {
	projects: ComputedRef<AnalyticsDashboardProject[]>
	selectedProjectIds: Ref<string[]>
	fetchRequest: Ref<Labrinth.Analytics.v3.FetchRequest | null>
	timeSlices: Ref<Labrinth.Analytics.v3.TimeSlice[]>
	previousTimeSlices: Ref<Labrinth.Analytics.v3.TimeSlice[]>
	isLoading: ComputedRef<boolean>
	isRefetching: ComputedRef<boolean>
	activeStat: Ref<AnalyticsDashboardStat>
	currentTotals: ComputedRef<AnalyticsDashboardTotals>
	previousTotals: ComputedRef<AnalyticsDashboardTotals>
	percentChanges: ComputedRef<AnalyticsDashboardPercentChanges>
	setFetchRequest: (fetchRequest: Labrinth.Analytics.v3.FetchRequest) => void
	setActiveStat: (stat: AnalyticsDashboardStat) => void
}

export type CreateAnalyticsDashboardContextOptions = {
	auth: Ref<{ user?: { id?: string } | null }>
	projectPageContext?: ProjectPageContext | null
	organizationContext?: OrganizationContext | null
}

export const [injectAnalyticsDashboardContext, provideAnalyticsDashboardContext] =
	createContext<AnalyticsDashboardContextValue>('AnalyticsDashboard')

function buildPreviousFetchRequest(
	fetchRequest: Labrinth.Analytics.v3.FetchRequest | null,
): Labrinth.Analytics.v3.FetchRequest | null {
	if (!fetchRequest) {
		return null
	}

	const startTimestamp = new Date(fetchRequest.time_range.start).getTime()
	const endTimestamp = new Date(fetchRequest.time_range.end).getTime()
	const duration = endTimestamp - startTimestamp

	if (!Number.isFinite(duration) || duration <= 0) {
		return null
	}

	const previousEnd = new Date(startTimestamp)
	const previousStart = new Date(startTimestamp - duration)

	return {
		time_range: {
			start: previousStart.toISOString(),
			end: previousEnd.toISOString(),
			resolution: fetchRequest.time_range.resolution,
		},
		return_metrics: fetchRequest.return_metrics,
	}
}

function getPercentChange(currentValue: number, previousValue: number): number {
	if (previousValue === 0) {
		if (currentValue === 0) {
			return 0
		}
		return 100
	}

	return ((currentValue - previousValue) / previousValue) * 100
}

function computeTotals(
	timeSlices: Labrinth.Analytics.v3.TimeSlice[],
	selectedProjectIds: Set<string>,
): AnalyticsDashboardTotals {
	const totals: AnalyticsDashboardTotals = {
		views: 0,
		downloads: 0,
		revenue: 0,
		playtime: 0,
	}

	for (const timeSlice of timeSlices) {
		for (const dataPoint of timeSlice) {
			if (!('source_project' in dataPoint)) {
				continue
			}

			if (selectedProjectIds.size > 0 && !selectedProjectIds.has(dataPoint.source_project)) {
				continue
			}

			switch (dataPoint.metric_kind) {
				case 'views':
					totals.views += dataPoint.views
					break
				case 'downloads':
					totals.downloads += dataPoint.downloads
					break
				case 'playtime':
					totals.playtime += dataPoint.seconds
					break
				case 'revenue': {
					const value = Number.parseFloat(dataPoint.revenue)
					totals.revenue += Number.isFinite(value) ? value : 0
					break
				}
			}
		}
	}

	return totals
}

export function createAnalyticsDashboardContext(
	options: CreateAnalyticsDashboardContextOptions,
): AnalyticsDashboardContextValue {
	const client = injectModrinthClient()
	const activeStat = ref<AnalyticsDashboardStat>('views')
	const selectedProjectIds = ref<string[]>([])
	const fetchRequest = ref<Labrinth.Analytics.v3.FetchRequest | null>(null)

	const hasProjectContext = computed(() => Boolean(options.projectPageContext))
	const hasOrganizationContext = computed(
		() => !hasProjectContext.value && Boolean(options.organizationContext),
	)

	const { data: userProjects } = useQuery({
		queryKey: computed(() => ['analytics', 'dashboard', options.auth.value?.user?.id, 'projects']),
		queryFn: () => client.labrinth.users_v2.getProjects(options.auth.value.user?.id ?? ''),
		enabled: computed(
			() =>
				Boolean(options.auth.value.user?.id) &&
				!hasProjectContext.value &&
				!hasOrganizationContext.value,
		),
		placeholderData: [],
	})

	const projects = computed<AnalyticsDashboardProject[]>(() => {
		if (hasProjectContext.value && options.projectPageContext) {
			const project = options.projectPageContext.projectV2.value
			return project ? [{ id: project.id, name: project.title }] : []
		}

		if (hasOrganizationContext.value && options.organizationContext?.projects.value) {
			return options.organizationContext.projects.value.map((project) => ({
				id: project.id,
				name: project.name,
			}))
		}

		return (userProjects.value ?? []).map((project) => ({
			id: project.id,
			name: project.title,
		}))
	})

	watch(
		projects,
		(nextProjects) => {
			if (nextProjects.length === 0) {
				selectedProjectIds.value = []
				return
			}

			const availableProjectIds = new Set(nextProjects.map((project) => project.id))
			const retainedSelection = selectedProjectIds.value.filter((id) => availableProjectIds.has(id))

			selectedProjectIds.value =
				retainedSelection.length > 0 ? retainedSelection : nextProjects.map((project) => project.id)
		},
		{ immediate: true },
	)

	const { data: currentTimeSliceData, isPending: currentTimeSlicePending, isFetching: currentFetching } =
		useQuery({
			queryKey: computed(() => ['analytics', 'dashboard', 'current', fetchRequest.value]),
			queryFn: () => client.labrinth.analytics_v3.fetch(fetchRequest.value as Labrinth.Analytics.v3.FetchRequest),
			enabled: computed(() => fetchRequest.value !== null),
			placeholderData: [],
		})

	const previousFetchRequest = computed(() => buildPreviousFetchRequest(fetchRequest.value))

	const {
		data: previousTimeSliceData,
		isPending: previousTimeSlicePending,
		isFetching: previousFetching,
	} = useQuery({
		queryKey: computed(() => ['analytics', 'dashboard', 'previous', previousFetchRequest.value]),
		queryFn: () =>
			client.labrinth.analytics_v3.fetch(
				previousFetchRequest.value as Labrinth.Analytics.v3.FetchRequest,
			),
		enabled: computed(() => previousFetchRequest.value !== null),
		placeholderData: [],
	})

	const timeSlices = ref<Labrinth.Analytics.v3.TimeSlice[]>([])
	const previousTimeSlices = ref<Labrinth.Analytics.v3.TimeSlice[]>([])

	watch(
		currentTimeSliceData,
		(nextTimeSlices) => {
			timeSlices.value = nextTimeSlices ?? []
		},
		{ immediate: true },
	)

	watch(
		previousTimeSliceData,
		(nextTimeSlices) => {
			previousTimeSlices.value = nextTimeSlices ?? []
		},
		{ immediate: true },
	)

	const selectedProjectIdSet = computed(() => new Set(selectedProjectIds.value))

	const currentTotals = computed<AnalyticsDashboardTotals>(() =>
		computeTotals(timeSlices.value, selectedProjectIdSet.value),
	)
	const previousTotals = computed<AnalyticsDashboardTotals>(() =>
		computeTotals(previousTimeSlices.value, selectedProjectIdSet.value),
	)

	const percentChanges = computed<AnalyticsDashboardPercentChanges>(() => ({
		views: getPercentChange(currentTotals.value.views, previousTotals.value.views),
		downloads: getPercentChange(currentTotals.value.downloads, previousTotals.value.downloads),
		revenue: getPercentChange(currentTotals.value.revenue, previousTotals.value.revenue),
		playtime: getPercentChange(currentTotals.value.playtime, previousTotals.value.playtime),
	}))

	const isLoading = computed(() => currentTimeSlicePending.value || previousTimeSlicePending.value)
	const isRefetching = computed(() => currentFetching.value || previousFetching.value)

	function setFetchRequest(nextFetchRequest: Labrinth.Analytics.v3.FetchRequest) {
		fetchRequest.value = nextFetchRequest
	}

	function setActiveStat(nextStat: AnalyticsDashboardStat) {
		activeStat.value = nextStat
	}

	return {
		projects,
		selectedProjectIds,
		fetchRequest,
		timeSlices,
		previousTimeSlices,
		isLoading,
		isRefetching,
		activeStat,
		currentTotals,
		previousTotals,
		percentChanges,
		setFetchRequest,
		setActiveStat,
	}
}
