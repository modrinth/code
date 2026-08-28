<template>
	<div class="flex flex-col gap-4">
		<div class="flex flex-col justify-between gap-3 lg:flex-row">
			<Input
				v-model="query"
				:icon="SearchIcon"
				type="text"
				autocomplete="off"
				:placeholder="formatMessage(commonMessages.searchPlaceholder)"
				clearable
				wrapper-class="flex-1"
				input-class="h-[40px] w-full"
				@input="goToPage(1)"
			/>

			<div class="flex flex-col flex-wrap justify-end gap-2 sm:flex-row lg:flex-shrink-0">
				<div class="flex flex-col gap-2 sm:flex-row">
					<div class="flex min-w-0 flex-grow gap-2 sm:flex-grow-0">
						<Combobox
							v-model="currentFilterType"
							class="!w-full min-w-0 flex-grow sm:!w-[280px] sm:flex-grow-0 lg:!w-[280px]"
							trigger-type="base"
							trigger-size="lg"
							:options="filterTypes"
							:placeholder="formatMessage(commonMessages.filterByLabel)"
							@select="goToPage(1)"
						>
							<template #selected>
								<span class="flex flex-row gap-2 align-middle font-semibold">
									<ListFilterIcon class="size-5 flex-shrink-0 text-secondary" />
									<span class="truncate text-contrast"
										>{{ currentFilterType }} ({{ totalProjects }})</span
									>
								</span>
							</template>
						</Combobox>
					</div>

					<Combobox
						v-model="currentSortType"
						class="!w-full flex-grow sm:!w-[240px] sm:flex-grow-0"
						trigger-type="base"
						trigger-size="lg"
						:options="sortTypes"
						:placeholder="formatMessage(commonMessages.sortByLabel)"
						@select="goToPage(1)"
					>
						<template #selected>
							<span class="flex flex-row gap-2 align-middle font-semibold">
								<SortAscIcon
									v-if="currentSortType === 'Oldest' || currentSortType === 'Least external deps'"
									class="size-5 flex-shrink-0 text-secondary"
								/>
								<SortDescIcon v-else class="size-5 flex-shrink-0 text-secondary" />
								<span class="truncate text-contrast">{{ currentSortType }}</span>
							</span>
						</template>
					</Combobox>

					<Combobox
						v-model="itemsPerPage"
						class="!w-full flex-grow sm:!w-[160px] sm:flex-grow-0 lg:!w-[140px]"
						trigger-type="base"
						trigger-size="lg"
						:options="itemsPerPageOptions"
						placeholder="Items per page"
						@select="goToPage(1)"
					>
						<template #selected>
							<span class="flex flex-row gap-2 align-middle font-semibold">
								<span class="truncate text-contrast">{{ itemsPerPage }} items</span>
							</span>
						</template>
					</Combobox>
				</div>

				<Button
					type="outlined"
					class="flex !h-[40px] w-full items-center justify-center gap-2 sm:w-auto"
					@click="openModerateByIdsModal"
				>
					<ListFilterIcon class="flex-shrink-0" />
					Moderate by IDs
				</Button>

				<Button
					type="colored"
					color="orange"
					class="flex !h-[40px] w-full items-center justify-center gap-2 sm:w-auto"
					:disabled="pending || paginatedProjects?.length === 0"
					@click="moderateAllInFilter()"
				>
					<ScaleIcon class="flex-shrink-0" />
					<span class="hidden sm:inline">{{ formatMessage(messages.moderate) }}</span>
					<span class="sm:hidden">Moderate</span>
				</Button>
			</div>
		</div>

		<div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
			<div class="flex flex-wrap items-center gap-3">
				<div v-if="totalProjects > 0">
					Showing {{ pageStart }}–{{ pageEnd }} of {{ totalProjects }}
					{{
						currentFilterType === DEFAULT_FILTER_TYPE ? 'projects' : currentFilterType.toLowerCase()
					}}
				</div>
				<div class="flex items-center gap-2 text-sm font-semibold text-secondary">
					<Toggle id="moderation-exclude-technical-review" v-model="excludeTechnicalReview" small />
					<label class="cursor-pointer" for="moderation-exclude-technical-review">
						{{ formatMessage(messages.excludeTechnicalReview) }}
					</label>
				</div>
			</div>
			<Pagination
				v-if="totalPages > 1"
				:page="currentPage"
				:count="totalPages"
				@switch-page="goToPage"
			/>
			<ConfettiExplosion v-if="visible" />
			<QueueSummaryModal
				ref="queueSummaryModal"
				:completed-ids="moderationQueue.currentQueue.completed"
				:skipped-ids="moderationQueue.currentQueue.skipped"
				@review-skipped="reviewSkippedQueue"
			/>
			<ModerateByIdsModal ref="moderateByIdsModal" @apply="startModeratingByIds" />
		</div>

		<div class="flex flex-col gap-3">
			<template v-if="pending">
				<div
					v-for="i in 3"
					:key="`loading-skeleton-${i}`"
					class="flex h-[98px] w-full animate-pulse rounded-2xl bg-surface-3"
				></div>
			</template>
			<EmptyState
				v-else-if="loadError"
				type="no-tasks"
				heading="Failed to load projects"
				:description="loadErrorMessage"
			/>
			<EmptyState
				v-else-if="paginatedProjects.length === 0"
				:type="!!query ? 'no-search-result' : 'no-tasks'"
				:heading="emptyStateHeading"
				:description="emptyStateDescription"
			/>
			<ModerationQueueCard
				v-for="item in paginatedProjects"
				v-else
				:key="item.project.id"
				:queue-entry="item"
				:show-external-dependencies="currentFilterType === MODPACK_FILTER_TYPE"
				@start-from-project="startFromProject"
			/>
		</div>

		<div v-if="totalPages > 1" class="flex justify-end">
			<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
		</div>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CopyIcon,
	ListFilterIcon,
	ScaleIcon,
	SearchIcon,
	SortAscIcon,
	SortDescIcon,
} from '@modrinth/assets'
import { Button } from '@modrinth/ui'
import {
	Combobox,
	type ComboboxOption,
	commonMessages,
	defineMessages,
	EmptyState,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	Pagination,
	Toggle,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { useDebounceFn } from '@vueuse/core'
import ConfettiExplosion from 'vue-confetti-explosion'

import ModerateByIdsModal from '~/components/ui/moderation/ModerateByIdsModal.vue'
import ModerationQueueCard from '~/components/ui/moderation/ModerationQueueCard.vue'
import QueueSummaryModal from '~/components/ui/moderation/QueueSummaryModal.vue'
import { type ModerationProject, toModerationProjects } from '~/helpers/moderation.ts'
import { getProjectTypeForUrlShorthand } from '~/helpers/projects.js'
import { useModerationQueue } from '~/services/moderation/queue.ts'
import { findNextEligibleQueueProject } from '~/services/moderation/queue-eligibility.ts'
import {
	scanProjectsWithValidationIssues,
	type ValidationFilterRequest,
} from '~/services/moderation/validation-filter.ts'

useHead({ title: 'Projects queue - Modrinth' })

const { formatMessage } = useVIntl()
const notificationManager = injectNotificationManager()
const { addNotification } = notificationManager
const moderationQueue = useModerationQueue()
const route = useRoute()
const router = useRouter()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const debugValidationFilter = useDebugLogger('moderation-validation-filter')
const tags = useGeneratedState()

const queueSummaryModal = ref()
const moderateByIdsModal = ref<InstanceType<typeof ModerateByIdsModal>>()

const visible = ref(false)
if (import.meta.client && history && history.state && history.state.confetti) {
	setTimeout(async () => {
		history.state.confetti = false
		visible.value = true
		await nextTick()
		setTimeout(() => {
			visible.value = false
		}, 5000)
	}, 1000)
}

if (import.meta.client && history && history.state && history.state.queueSummary) {
	setTimeout(async () => {
		history.state.queueSummary = false
		await nextTick()
		queueSummaryModal.value?.show()
	}, 1000)
}

const messages = defineMessages({
	moderate: {
		id: 'moderation.moderate',
		defaultMessage: 'Moderate',
	},
	excludeTechnicalReview: {
		id: 'moderation.exclude-technical-review',
		defaultMessage: 'Exclude TR',
	},
})

const query = ref(route.query.q?.toString() || '')
const debouncedFilterQuery = ref(query.value)
const excludeTechnicalReview = ref(false)

const updateDebouncedFilterQuery = useDebounceFn((value: string) => {
	debouncedFilterQuery.value = value
}, 500)

watch(
	query,
	(newQuery) => {
		updateDebouncedFilterQuery(newQuery)
		const currentQuery = { ...route.query }
		if (newQuery) {
			currentQuery.q = newQuery
		} else {
			delete currentQuery.q
		}

		router.replace({
			path: route.path,
			query: currentQuery,
		})
	},
	{ immediate: false },
)

watch(
	() => route.query.q,
	(newQueryParam) => {
		const newValue = newQueryParam?.toString() || ''
		if (query.value !== newValue) {
			query.value = newValue
		}
	},
)

const filterTypes: ComboboxOption<string>[] = [
	{ value: 'All projects', label: 'All projects' },
	{ value: 'Modpacks', label: 'Modpacks' },
	{ value: 'Mods', label: 'Mods' },
	{ value: 'Resource Packs', label: 'Resource Packs' },
	{ value: 'Data Packs', label: 'Data Packs' },
	{ value: 'Plugins', label: 'Plugins' },
	{ value: 'Shaders', label: 'Shaders' },
	{ value: 'Servers', label: 'Servers' },
	{ value: 'Validation errors', label: 'Validation errors' },
	{ value: 'Validation errors + warnings', label: 'Validation errors + warnings' },
	{ value: 'Fucked up', label: 'Fucked up' },
]
const filterTypeValues = filterTypes.map((option) => option.value)
const DEFAULT_FILTER_TYPE = filterTypeValues[0]

const MODPACK_FILTER_TYPE = 'Modpacks'
const VALIDATION_ERROR_FILTER_TYPE = 'Validation errors'
const VALIDATION_ERROR_AND_WARNING_FILTER_TYPE = 'Validation errors + warnings'
const VALIDATION_FILTER_STALE_TIME_MS = 1000 * 60 * 5

const baseSortTypes: ComboboxOption<string>[] = [
	{ value: 'Oldest', label: 'Oldest' },
	{ value: 'Newest', label: 'Newest' },
]
const modpackSortTypes: ComboboxOption<string>[] = [
	{ value: 'Most external deps', label: 'Most external deps' },
	{ value: 'Least external deps', label: 'Least external deps' },
]
const DEFAULT_SORT_TYPE = baseSortTypes[0].value
const modpackSortTypeValues = modpackSortTypes.map((option) => option.value)

const sortTypes = computed(() => {
	if (currentFilterType.value === MODPACK_FILTER_TYPE) {
		return [...baseSortTypes, ...modpackSortTypes]
	}
	return baseSortTypes
})

const itemsPerPageOptions: ComboboxOption<number>[] = [
	{ value: 20, label: '20' },
	{ value: 40, label: '40' },
	{ value: 60, label: '60' },
	{ value: 80, label: '80' },
	{ value: 100, label: '100' },
	{ value: 200, label: '200' },
]
const itemsPerPageValues = itemsPerPageOptions.map((option) => option.value)
const DEFAULT_ITEMS_PER_PAGE = 40

function parseFilterTypeFromQuery(value: LocationQueryValue | LocationQueryValue[]): string {
	const query = queryAsStringOrEmpty(value)
	return filterTypeValues.includes(query) ? query : DEFAULT_FILTER_TYPE
}

function parseSortTypeFromQuery(
	value: LocationQueryValue | LocationQueryValue[],
	filterType: string,
): string {
	const query = queryAsStringOrEmpty(value)
	const validValues = [
		...baseSortTypes.map((option) => option.value),
		...(filterType === MODPACK_FILTER_TYPE ? modpackSortTypeValues : []),
	]
	return validValues.includes(query) ? query : DEFAULT_SORT_TYPE
}

const currentFilterType = ref(parseFilterTypeFromQuery(route.query.filter))
const currentSortType = ref(parseSortTypeFromQuery(route.query.sort, currentFilterType.value))

watch(
	currentFilterType,
	(newFilter) => {
		if (
			newFilter !== MODPACK_FILTER_TYPE &&
			modpackSortTypeValues.includes(currentSortType.value)
		) {
			currentSortType.value = DEFAULT_SORT_TYPE
		}

		const currentQuery = { ...route.query }
		if (newFilter && newFilter !== DEFAULT_FILTER_TYPE) {
			currentQuery.filter = newFilter
		} else {
			delete currentQuery.filter
		}

		router.replace({
			path: route.path,
			query: currentQuery,
		})
	},
	{ immediate: false },
)

watch(
	() => route.query.filter,
	(newFilterParam) => {
		const newValue = parseFilterTypeFromQuery(newFilterParam)
		if (currentFilterType.value !== newValue) {
			currentFilterType.value = newValue
		}
	},
)

watch(
	currentSortType,
	(newSort) => {
		const currentQuery = { ...route.query }
		if (newSort && newSort !== DEFAULT_SORT_TYPE) {
			currentQuery.sort = newSort
		} else {
			delete currentQuery.sort
		}

		router.replace({
			path: route.path,
			query: currentQuery,
		})
	},
	{ immediate: false },
)

watch(
	() => route.query.sort,
	(newSortParam) => {
		const newValue = parseSortTypeFromQuery(newSortParam, currentFilterType.value)
		if (currentSortType.value !== newValue) {
			currentSortType.value = newValue
		}
	},
)

const itemsPerPageCookie = useCookie<number>('moderation-items-per-page', {
	default: () => DEFAULT_ITEMS_PER_PAGE,
	maxAge: 60 * 60 * 24 * 365,
	sameSite: 'lax',
	path: '/',
})

const itemsPerPage = computed({
	get() {
		const value = Number(itemsPerPageCookie.value)
		return itemsPerPageValues.includes(value) ? value : DEFAULT_ITEMS_PER_PAGE
	},
	set(value: number) {
		itemsPerPageCookie.value = value
	},
})

const currentPage = ref(1)

function toApiProjectType(label: string): string | undefined {
	switch (label) {
		case 'Modpacks':
			return 'modpack'
		case 'Mods':
			return 'mod'
		case 'Resource Packs':
			return 'resourcepack'
		case 'Data Packs':
			return 'datapack'
		case 'Plugins':
			return 'plugin'
		case 'Shaders':
			return 'shader'
		case 'Servers':
			return 'minecraft_java_server'
		case 'Fucked up':
			return 'none'
		default:
			return undefined
	}
}

function toApiSort(label: string): Labrinth.Moderation.Internal.ProjectsSort {
	switch (label) {
		case 'Newest':
			return 'newest'
		case 'Most external deps':
			return 'most_external_deps'
		case 'Least external deps':
			return 'least_external_deps'
		default:
			return 'oldest'
	}
}

const moderationProjectsRequest = computed<Labrinth.Moderation.Internal.ProjectsRequest>(() => ({
	count: itemsPerPage.value,
	offset: (currentPage.value - 1) * itemsPerPage.value,
	exclude_technical_review: excludeTechnicalReview.value,
	query: query.value || undefined,
	project_type: toApiProjectType(currentFilterType.value),
	sort: toApiSort(currentSortType.value),
}))

const moderationProjectsQueryKey = computed(
	() => ['moderation-projects', moderationProjectsRequest.value] as const,
)

const isValidationErrorFilter = computed(
	() => currentFilterType.value === VALIDATION_ERROR_FILTER_TYPE,
)
const isValidationErrorAndWarningFilter = computed(
	() => currentFilterType.value === VALIDATION_ERROR_AND_WARNING_FILTER_TYPE,
)
const isValidationFilter = computed(
	() => isValidationErrorFilter.value || isValidationErrorAndWarningFilter.value,
)

const {
	data: standardProjectsResponse,
	isPending: standardProjectsPending,
	isPlaceholderData: standardProjectsPlaceholder,
	error: standardProjectsError,
} = useQuery({
	queryKey: moderationProjectsQueryKey,
	queryFn: ({ queryKey }) => client.labrinth.moderation_internal.getProjects(queryKey[1]),
	placeholderData: (previousData) => previousData,
	enabled: computed(() => !isValidationFilter.value),
})

const validationFilterRequest = computed<ValidationFilterRequest>(() => ({
	exclude_technical_review: excludeTechnicalReview.value,
	query: debouncedFilterQuery.value || undefined,
	sort: toApiSort(currentSortType.value),
}))

const validationProjectsQueryKey = computed(
	() =>
		[
			'moderation-projects',
			'validation',
			isValidationErrorAndWarningFilter.value,
			validationFilterRequest.value,
		] as const,
)

let validationScanNotificationId: string | number | undefined

function showValidationScanCompleteNotification(
	response: Labrinth.Moderation.Internal.ProjectsResponse,
	includeWarnings: boolean,
) {
	if (validationScanNotificationId !== undefined) {
		notificationManager.removeNotification(validationScanNotificationId)
	}

	const projectIds = response.projects.map((project) => project.id)
	const notification = addNotification({
		title: 'Validation scan complete',
		text: `Found ${response.total} projects with validation ${includeWarnings ? 'errors or warnings' : 'errors'}.`,
		type: 'success',
		autoCloseMs: null,
		copyable: false,
		buttons: [
			{
				label: 'Copy all IDs',
				icon: CopyIcon,
				keepOpen: true,
				action: () => navigator.clipboard.writeText(projectIds.join('\n')),
			},
		],
	})
	validationScanNotificationId = notification.id
}

const {
	data: validationProjectsResponse,
	isPending: validationProjectsPending,
	error: validationProjectsError,
} = useQuery({
	queryKey: validationProjectsQueryKey,
	queryFn: async ({ queryKey, signal }) => {
		const response = await scanProjectsWithValidationIssues({
			client,
			request: queryKey[3],
			includeWarnings: queryKey[2],
			tags: tags.value,
			signal,
			log: debugValidationFilter,
		})
		showValidationScanCompleteNotification(response, queryKey[2])
		return response
	},
	enabled: computed(() => import.meta.client && isValidationFilter.value),
	staleTime: VALIDATION_FILTER_STALE_TIME_MS,
	retry: false,
})

watch([isValidationFilter, validationProjectsQueryKey], ([isActive]) => {
	if (!isActive) return
	const cached = queryClient.getQueryData<Labrinth.Moderation.Internal.ProjectsResponse>(
		validationProjectsQueryKey.value,
	)
	if (cached) {
		debugValidationFilter(`Using cached scan result with ${cached.total} matching projects`)
		showValidationScanCompleteNotification(cached, isValidationErrorAndWarningFilter.value)
	}
})

const usesLocalPagination = computed(() => isValidationFilter.value)
const moderationProjectsResponse = computed(() =>
	isValidationFilter.value ? validationProjectsResponse.value : standardProjectsResponse.value,
)
const pending = computed(() =>
	isValidationFilter.value
		? validationProjectsPending.value
		: standardProjectsPending.value || standardProjectsPlaceholder.value,
)
const loadError = computed(() =>
	isValidationFilter.value ? validationProjectsError.value : standardProjectsError.value,
)
const loadErrorMessage = computed(
	() => loadError.value?.message ?? 'An unknown error occurred while loading the moderation queue.',
)
const totalProjects = computed(() => moderationProjectsResponse.value?.total ?? 0)
const totalPages = computed(() => Math.ceil(totalProjects.value / itemsPerPage.value))
const filteredProjects = computed(() =>
	toModerationProjects(moderationProjectsResponse.value?.projects ?? []),
)
const paginatedProjects = computed(() => {
	if (!usesLocalPagination.value) return filteredProjects.value
	const start = (currentPage.value - 1) * itemsPerPage.value
	return filteredProjects.value.slice(start, start + itemsPerPage.value)
})
const pageStart = computed(() =>
	totalProjects.value === 0 ? 0 : (currentPage.value - 1) * itemsPerPage.value + 1,
)
const pageEnd = computed(() =>
	Math.min(
		(currentPage.value - 1) * itemsPerPage.value + paginatedProjects.value.length,
		totalProjects.value,
	),
)
const projectsById = computed(() => {
	const projects = new Map<string, ModerationProject>()
	for (const project of filteredProjects.value) {
		projects.set(project.project.id, project)
	}

	return projects
})

watch(totalPages, (pages) => {
	if (pages === 0 && currentPage.value !== 1) {
		currentPage.value = 1
		return
	}

	if (pages > 0 && currentPage.value > pages) {
		currentPage.value = pages
	}
})

watch(excludeTechnicalReview, () => {
	goToPage(1)
})

const emptyStateHeading = computed(() => {
	if (query.value) {
		return 'Not finding anything...'
	}
	if (currentFilterType.value !== DEFAULT_FILTER_TYPE) {
		return 'All done here!'
	}
	return 'The queue is empty!'
})

const emptyStateDescription = computed(() => {
	if (query.value) {
		return 'Check that your search query is correct!'
	}
	if (currentFilterType.value !== DEFAULT_FILTER_TYPE) {
		return `There are no ${currentFilterType.value.toLowerCase()} in the queue.`
	}
	return 'you will probably never see this but if you do, congrats!!! :D'
})

function goToPage(page: number) {
	currentPage.value = page
}

function openModerateByIdsModal() {
	moderateByIdsModal.value?.show()
}

async function findFirstEligibleProject(): Promise<string | null> {
	const candidateIds = [...moderationQueue.currentQueue.items]
	if (candidateIds.length === 0) return null

	const next = await findNextEligibleQueueProject(client, moderationQueue, candidateIds)

	if (!next) {
		await Promise.all(candidateIds.map((id) => moderationQueue.excludeProject(id)))
		return null
	}

	await Promise.all(next.excluded.map((id) => moderationQueue.excludeProject(id)))
	return next.project
}

function getProjectRouteParam(projectId: string): string {
	return projectsById.value.get(projectId)?.project.slug || projectId
}

function getProjectRouteType(projectId: string): string {
	const projectType = projectsById.value.get(projectId)?.project.project_types[0]
	if (!projectType) return 'project'
	return getProjectTypeForUrlShorthand(projectType, [])
}

async function navigateToModerationProject(projectId: string) {
	await navigateTo({
		name: 'type-project',
		params: {
			type: getProjectRouteType(projectId),
			project: getProjectRouteParam(projectId),
		},
		state: {
			showChecklist: true,
		},
	})
}

async function startModeratingByIds(projectIds: string[]) {
	await moderationQueue.setQueue(projectIds)

	const targetProjectId = await findFirstEligibleProject()

	if (!targetProjectId) {
		addNotification({
			title: 'No projects available',
			text: 'None of the provided projects are awaiting moderation or available to review.',
			type: 'warning',
		})
		return
	}

	await navigateToModerationProject(targetProjectId)
}

async function getFilteredProjectIds(): Promise<string[]> {
	if (usesLocalPagination.value) {
		return filteredProjects.value.map((project) => project.project.id)
	}

	const response = await client.labrinth.moderation_internal.getProjectIds({
		exclude_technical_review: excludeTechnicalReview.value,
		query: query.value || undefined,
		project_type: toApiProjectType(currentFilterType.value),
		sort: toApiSort(currentSortType.value),
	})

	return response.ids
}

async function moderateAllInFilter() {
	const startIndex = (currentPage.value - 1) * itemsPerPage.value
	const projectIds = (await getFilteredProjectIds()).slice(startIndex)
	await moderationQueue.setQueue(projectIds)

	const targetProjectId = await findFirstEligibleProject()

	if (!targetProjectId) {
		addNotification({
			title: 'No projects available',
			text: 'All projects in queue are already moderated or locked by others.',
			type: 'warning',
		})
		return
	}

	await navigateToModerationProject(targetProjectId)
}

async function startFromProject(projectId: string) {
	const allFilteredProjectIds = await getFilteredProjectIds()
	const projectIndex = allFilteredProjectIds.indexOf(projectId)
	const projectIds = projectIndex === -1 ? [projectId] : allFilteredProjectIds.slice(projectIndex)
	await moderationQueue.setQueue(projectIds)

	const targetProjectId = await findFirstEligibleProject()

	if (!targetProjectId) {
		addNotification({
			title: 'No projects available',
			text: 'All projects in queue are already moderated or locked by others.',
			type: 'warning',
		})
		return
	}

	await navigateToModerationProject(targetProjectId)
}

async function reviewSkippedQueue() {
	await moderationQueue.startSkippedReview()

	const targetProjectId = await findFirstEligibleProject()

	if (!targetProjectId) {
		addNotification({
			title: 'No projects available',
			text: 'All previously skipped projects are already moderated or locked by others.',
			type: 'warning',
		})
		return
	}

	await navigateToModerationProject(targetProjectId)
}
</script>
