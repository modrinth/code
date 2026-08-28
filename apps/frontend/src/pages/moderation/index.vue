<template>
	<div class="flex flex-col gap-4">
		<ModerationQueueToolbar
			v-model="query"
			:page="currentPage"
			:total-pages="totalPages"
			@search="goToPage(1)"
			@switch-page="goToPage"
		>
			<template #actions>
				<Combobox
					v-model="currentFilterType"
					class="!w-full flex-grow sm:!w-[280px] sm:flex-grow-0 lg:!w-[280px]"
					trigger-type="base"
					trigger-size="lg"
					:options="filterTypes"
					:placeholder="formatMessage(commonMessages.filterByLabel)"
					@select="goToPage(1)"
				>
					<template #selected>
						<span class="flex flex-row gap-2 align-middle font-semibold">
							<ListFilterIcon class="size-5 flex-shrink-0 text-secondary" />
							<ModerationFilterCount
								:label="currentFilterType"
								:count="totalProjects"
								:loading="pending"
							/>
						</span>
					</template>
				</Combobox>

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

				<Button
					type="colored"
					color="orange"
					size="lg"
					class="w-full sm:w-auto"
					:disabled="pending || paginatedProjects?.length === 0"
					@click="moderateAllInFilter()"
				>
					<ScaleIcon />
					<span class="hidden sm:inline">{{ formatMessage(messages.moderate) }}</span>
					<span class="sm:hidden">Moderate</span>
				</Button>
			</template>
			<template #meta>
				<div v-if="totalProjects > 0">
					Showing {{ formatNumber(pageStart) }}–{{ formatNumber(pageEnd) }} of
					{{ formatNumber(totalProjects) }}
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
			</template>
		</ModerationQueueToolbar>

		<ConfettiExplosion v-if="visible" />
		<QueueSummaryModal
			ref="queueSummaryModal"
			:completed-ids="moderationQueue.currentQueue.completed"
			:skipped-ids="moderationQueue.currentQueue.skipped"
			@review-skipped="reviewSkippedQueue"
		/>

		<ModerationQueueSkeleton v-if="pending" />
		<EmptyState
			v-else-if="paginatedProjects.length === 0"
			:type="!!query ? 'no-search-result' : 'no-tasks'"
			:heading="emptyStateHeading"
			:description="emptyStateDescription"
		/>
		<div v-else class="flex flex-col gap-3">
			<ModerationQueueCard
				v-for="item in paginatedProjects"
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
import { ListFilterIcon, ScaleIcon, SortAscIcon, SortDescIcon } from '@modrinth/assets'
import { Button } from '@modrinth/ui'
import {
	Combobox,
	type ComboboxOption,
	commonMessages,
	defineMessages,
	EmptyState,
	injectModrinthClient,
	injectNotificationManager,
	Pagination,
	Toggle,
	useFormatNumber,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import ConfettiExplosion from 'vue-confetti-explosion'

import ModerationFilterCount from '~/components/ui/moderation/ModerationFilterCount.vue'
import ModerationQueueCard from '~/components/ui/moderation/ModerationQueueCard.vue'
import ModerationQueueSkeleton from '~/components/ui/moderation/ModerationQueueSkeleton.vue'
import ModerationQueueToolbar from '~/components/ui/moderation/ModerationQueueToolbar.vue'
import QueueSummaryModal from '~/components/ui/moderation/QueueSummaryModal.vue'
import { type ModerationProject, toModerationProjects } from '~/helpers/moderation.ts'
import { getProjectTypeForUrlShorthand } from '~/helpers/projects.js'
import { useModerationQueue } from '~/services/moderation/queue.ts'
import { findNextEligibleQueueProject } from '~/services/moderation/queue-eligibility.ts'

useHead({ title: 'Projects queue - Modrinth' })

const { formatMessage } = useVIntl()
const formatNumber = useFormatNumber()
const { addNotification } = injectNotificationManager()
const moderationQueue = useModerationQueue()
const route = useRoute()
const router = useRouter()
const client = injectModrinthClient()

const queueSummaryModal = ref()

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
const excludeTechnicalReview = ref(false)

watch(
	query,
	(newQuery) => {
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
	{ value: 'Fucked up', label: 'Fucked up' },
]
const filterTypeValues = filterTypes.map((option) => option.value)
const DEFAULT_FILTER_TYPE = filterTypeValues[0]

const MODPACK_FILTER_TYPE = 'Modpacks'

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

const {
	data: moderationProjectsResponse,
	isPending: moderationProjectsPending,
	isPlaceholderData: moderationProjectsPlaceholder,
} = useQuery({
	queryKey: moderationProjectsQueryKey,
	queryFn: ({ queryKey }) => client.labrinth.moderation_internal.getProjects(queryKey[1]),
	placeholderData: (previousData) => previousData,
})

const pending = computed(
	() => moderationProjectsPending.value || moderationProjectsPlaceholder.value,
)
const totalProjects = computed(() => moderationProjectsResponse.value?.total ?? 0)
const totalPages = computed(() => Math.ceil(totalProjects.value / itemsPerPage.value))
const filteredProjects = computed(() =>
	toModerationProjects(moderationProjectsResponse.value?.projects ?? []),
)
const paginatedProjects = computed(() => filteredProjects.value)
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

async function getFilteredProjectIds(): Promise<string[]> {
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
