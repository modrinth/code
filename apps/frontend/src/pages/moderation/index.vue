<template>
	<div class="flex flex-col gap-4">
		<div class="flex flex-col justify-between gap-3 lg:flex-row">
			<StyledInput
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
					<Combobox
						v-model="currentFilterType"
						class="!w-full flex-grow sm:!w-[280px] sm:flex-grow-0 lg:!w-[280px]"
						:options="filterTypes"
						:placeholder="formatMessage(commonMessages.filterByLabel)"
						@select="goToPage(1)"
					>
						<template #selected>
							<span class="flex flex-row gap-2 align-middle font-semibold">
								<ListFilterIcon class="size-5 flex-shrink-0 text-secondary" />
								<span class="truncate text-contrast"
									>{{ currentFilterType }} ({{ filteredProjects.length }})</span
								>
							</span>
						</template>
					</Combobox>

					<Combobox
						v-model="currentSortType"
						class="!w-full flex-grow sm:!w-[150px] sm:flex-grow-0 lg:!w-[150px]"
						:options="sortTypes"
						:placeholder="formatMessage(commonMessages.sortByLabel)"
						@select="goToPage(1)"
					>
						<template #selected>
							<span class="flex flex-row gap-2 align-middle font-semibold">
								<SortAscIcon
									v-if="currentSortType === 'Oldest'"
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

				<ButtonStyled color="orange">
					<button
						class="flex !h-[40px] w-full items-center justify-center gap-2 sm:w-auto"
						:disabled="paginatedProjects?.length === 0"
						@click="moderateAllInFilter()"
					>
						<ScaleIcon class="flex-shrink-0" />
						<span class="hidden sm:inline">{{ formatMessage(messages.moderate) }}</span>
						<span class="sm:hidden">Moderate</span>
					</button>
				</ButtonStyled>
			</div>
		</div>

		<div v-if="totalPages > 1" class="flex items-center justify-between">
			<div>
				Showing {{ itemsPerPage * (currentPage - 1) + 1 }}–{{
					itemsPerPage * (currentPage - 1) + Math.min(itemsPerPage, paginatedProjects.length)
				}}
				of {{ filteredProjects.length }}
				{{
					currentFilterType === DEFAULT_FILTER_TYPE ? 'projects' : currentFilterType.toLowerCase()
				}}
			</div>
			<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
			<ConfettiExplosion v-if="visible" />
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
				@start-from-project="startFromProject"
			/>
		</div>

		<div v-if="totalPages > 1" class="flex justify-end">
			<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
		</div>
	</div>
</template>
<script setup lang="ts">
import { ListFilterIcon, ScaleIcon, SearchIcon, SortAscIcon, SortDescIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	Combobox,
	type ComboboxOption,
	commonMessages,
	defineMessages,
	EmptyState,
	injectNotificationManager,
	Pagination,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import Fuse from 'fuse.js'
import ConfettiExplosion from 'vue-confetti-explosion'

import ModerationQueueCard from '~/components/ui/moderation/ModerationQueueCard.vue'
import {
	type ModerationProject,
	type ProjectWithOwnership,
	toModerationProjects,
} from '~/helpers/moderation.ts'
import { useModerationQueue } from '~/services/moderation-queue.ts'

useHead({ title: 'Projects queue - Modrinth' })

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const moderationQueue = useModerationQueue()
const route = useRoute()
const router = useRouter()

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

const messages = defineMessages({
	moderate: {
		id: 'moderation.moderate',
		defaultMessage: 'Moderate',
	},
})

const { data: allProjects, pending } = await useLazyAsyncData('moderation-projects', async () => {
	const startTime = performance.now()
	let currentOffset = 0
	const PROJECT_ENDPOINT_COUNT = 350
	const allProjects: ModerationProject[] = []

	let projects: ProjectWithOwnership[] = []
	do {
		projects = (await useBaseFetch(
			`moderation/projects?count=${PROJECT_ENDPOINT_COUNT}&offset=${currentOffset}`,
			{ internal: true },
		)) as ProjectWithOwnership[]

		if (projects.length === 0) break

		allProjects.push(...toModerationProjects(projects))
		currentOffset += projects.length
	} while (projects.length === PROJECT_ENDPOINT_COUNT)

	const duration = performance.now() - startTime

	console.debug(
		`Projects fetched and processed in ${duration.toFixed(2)}ms (${(duration / 1000).toFixed(2)}s)`,
	)

	return allProjects
})

const query = ref(route.query.q?.toString() || '')

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

const sortTypes: ComboboxOption<string>[] = [
	{ value: 'Oldest', label: 'Oldest' },
	{ value: 'Newest', label: 'Newest' },
]
const sortTypeValues = sortTypes.map((option) => option.value)
const DEFAULT_SORT_TYPE = sortTypeValues[0]

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

function parseSortTypeFromQuery(value: LocationQueryValue | LocationQueryValue[]): string {
	const query = queryAsStringOrEmpty(value)
	return sortTypeValues.includes(query) ? query : DEFAULT_SORT_TYPE
}

const currentFilterType = ref(parseFilterTypeFromQuery(route.query.filter))
const currentSortType = ref(parseSortTypeFromQuery(route.query.sort))

watch(
	currentFilterType,
	(newFilter) => {
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
		const newValue = parseSortTypeFromQuery(newSortParam)
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
const totalPages = computed(() =>
	Math.ceil((filteredProjects.value?.length || 0) / itemsPerPage.value),
)

const fuse = computed(() => {
	if (!allProjects.value || allProjects.value.length === 0) return null
	return new Fuse(allProjects.value, {
		keys: [
			{
				name: 'project.title',
				weight: 3,
			},
			{
				name: 'project.slug',
				weight: 2,
			},
			{
				name: 'project.description',
				weight: 2,
			},
			{
				name: 'project.project_type',
				weight: 1,
			},
			'ownership.name',
		],
		includeScore: true,
		threshold: 0.4,
	})
})

const searchResults = computed(() => {
	if (!query.value || !fuse.value) return null
	return fuse.value.search(query.value).map((result) => result.item)
})

const baseFiltered = computed(() => {
	if (!allProjects.value) return []
	return query.value && searchResults.value ? searchResults.value : [...allProjects.value]
})

const typeFiltered = computed(() => {
	if (currentFilterType.value === 'All projects') {
		return baseFiltered.value
	} else if (currentFilterType.value === 'Fucked up') {
		return baseFiltered.value.filter((queueItem) => queueItem.project.project_types.length === 0)
	}

	const filterMap: Record<string, string> = {
		Modpacks: 'modpack',
		Mods: 'mod',
		'Resource Packs': 'resourcepack',
		'Data Packs': 'datapack',
		Plugins: 'plugin',
		Shaders: 'shader',
		Servers: 'minecraft_java_server',
	}
	const projectType = filterMap[currentFilterType.value]
	if (!projectType) return baseFiltered.value

	return baseFiltered.value.filter(
		(queueItem) =>
			(queueItem.project.project_types.length > 0 &&
				queueItem.project.project_types[0] === projectType) ||
			(projectType === 'minecraft_java_server' &&
				queueItem.project.project_types.includes('minecraft_java_server')),
	)
})

const filteredProjects = computed(() => {
	const filtered = [...typeFiltered.value]

	if (currentSortType.value === 'Oldest') {
		filtered.sort((a, b) => {
			const dateA = new Date(a.project.queued || a.project.published || 0).getTime()
			const dateB = new Date(b.project.queued || b.project.published || 0).getTime()
			return dateA - dateB
		})
	} else {
		filtered.sort((a, b) => {
			const dateA = new Date(a.project.queued || a.project.published || 0).getTime()
			const dateB = new Date(b.project.queued || b.project.published || 0).getTime()
			return dateB - dateA
		})
	}

	return filtered
})

const paginatedProjects = computed(() => {
	if (!filteredProjects.value) return []
	const start = (currentPage.value - 1) * itemsPerPage.value
	const end = start + itemsPerPage.value
	return filteredProjects.value.slice(start, end)
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

async function findFirstUnlockedProject(): Promise<ModerationProject | null> {
	let skippedCount = 0

	while (moderationQueue.hasItems) {
		const currentId = moderationQueue.getCurrentProjectId()
		if (!currentId) return null

		const project = filteredProjects.value.find((p) => p.project.id === currentId)
		if (!project) {
			await moderationQueue.completeCurrentProject(currentId, 'skipped')
			continue
		}

		try {
			const lockStatus = await moderationQueue.checkLock(currentId)

			if (!lockStatus.locked || lockStatus.expired) {
				if (skippedCount > 0) {
					addNotification({
						title: 'Skipped locked projects',
						text: `Skipped ${skippedCount} project(s) being moderated by others.`,
						type: 'info',
					})
				}
				return project
			}

			// Project is locked, skip it
			await moderationQueue.completeCurrentProject(currentId, 'skipped')
			skippedCount++
		} catch {
			return project
		}
	}

	return null
}

async function moderateAllInFilter() {
	// Start from the current page - get projects from current page onwards
	const startIndex = (currentPage.value - 1) * itemsPerPage.value
	const projectsFromCurrentPage = filteredProjects.value.slice(startIndex)
	const projectIds = projectsFromCurrentPage.map((queueItem) => queueItem.project.id)
	await moderationQueue.setQueue(projectIds)

	// Find first unlocked project
	const targetProject = await findFirstUnlockedProject()

	if (!targetProject) {
		addNotification({
			title: 'All projects locked',
			text: 'All projects in queue are currently being moderated by others.',
			type: 'warning',
		})
		return
	}

	navigateTo({
		name: 'type-id',
		params: {
			type: 'project',
			id: targetProject.project.slug,
		},
		state: {
			showChecklist: true,
		},
	})
}

async function startFromProject(projectId: string) {
	// Find the index of the clicked project in the filtered list
	const projectIndex = filteredProjects.value.findIndex((p) => p.project.id === projectId)
	if (projectIndex === -1) {
		// Project not found in filtered list, just moderate it alone
		await moderationQueue.setSingleProject(projectId)
	} else {
		// Start queue from this project onwards
		const projectsFromHere = filteredProjects.value.slice(projectIndex)
		const projectIds = projectsFromHere.map((queueItem) => queueItem.project.id)
		await moderationQueue.setQueue(projectIds)
	}

	// Find first unlocked project
	const targetProject = await findFirstUnlockedProject()

	if (!targetProject) {
		addNotification({
			title: 'All projects locked',
			text: 'All projects in queue are currently being moderated by others.',
			type: 'warning',
		})
		return
	}

	navigateTo({
		name: 'type-id',
		params: {
			type: 'project',
			id: targetProject.project.slug,
		},
		state: {
			showChecklist: true,
		},
	})
}
</script>
